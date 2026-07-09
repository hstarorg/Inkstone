use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use sha2::Sha256;

pub const MK_LEN: usize = 32;
pub const DEK_LEN: usize = 32;
pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 24;
pub const RECOVERY_CODE_LEN: usize = 20;
pub const FILE_TOKEN_LEN: usize = 16;

#[derive(Clone, Copy)]
pub struct Argon2Params {
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

impl Argon2Params {
    pub const DEFAULT: Self = Self {
        memory_kib: 65536,
        iterations: 3,
        parallelism: 4,
    };
}

pub struct Envelope {
    pub nonce: [u8; NONCE_LEN],
    pub ciphertext: Vec<u8>,
}

pub fn random_bytes<const N: usize>() -> [u8; N] {
    let mut buf = [0u8; N];
    getrandom::fill(&mut buf).expect("OS RNG must be available");
    buf
}

/// A random identifier for encrypted-vault documents/assets: unlike a ULID,
/// carries no embedded timestamp, so it is safe to use as both the on-disk
/// filename and the document's own `id` without leaking creation time.
pub fn random_token_hex() -> String {
    let bytes = random_bytes::<FILE_TOKEN_LEN>();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn derive_kek_from_password(
    password: &str,
    salt: &[u8; SALT_LEN],
    params: &Argon2Params,
) -> Result<[u8; 32], String> {
    let argon2_params = Params::new(
        params.memory_kib,
        params.iterations,
        params.parallelism,
        Some(32),
    )
    .map_err(|error| format!("invalid Argon2 parameters: {error}"))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon2_params);
    let mut out = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut out)
        .map_err(|error| format!("key derivation failed: {error}"))?;
    Ok(out)
}

pub fn derive_kek_from_recovery(recovery_bytes: &[u8], salt: &[u8; SALT_LEN]) -> [u8; 32] {
    hkdf_expand(recovery_bytes, salt, b"inkstone-recovery-kek-v1")
}

pub fn derive_index_key(mk: &[u8; MK_LEN], vault_id: &str) -> [u8; 32] {
    hkdf_expand(mk, vault_id.as_bytes(), b"inkstone-index-key-v1")
}

fn hkdf_expand(ikm: &[u8], salt: &[u8], info: &[u8]) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(Some(salt), ikm);
    let mut out = [0u8; 32];
    hk.expand(info, &mut out)
        .expect("32 bytes is a valid HKDF-SHA256 output length");
    out
}

/// Encrypts `plaintext` under `key` with a freshly generated random nonce.
pub fn seal(key: &[u8; 32], plaintext: &[u8]) -> Envelope {
    let nonce = random_bytes::<NONCE_LEN>();
    let cipher = XChaCha20Poly1305::new(key.as_slice().try_into().expect("32-byte key"));
    let xnonce: &XNonce = nonce.as_slice().try_into().expect("24-byte nonce");
    let ciphertext = cipher
        .encrypt(xnonce, plaintext)
        .expect("encryption with a valid key and nonce cannot fail");
    Envelope { nonce, ciphertext }
}

/// Decrypts a ciphertext produced by [`seal`]. Fails if `key` is wrong or
/// `ciphertext` was tampered with / corrupted (AEAD tag mismatch).
pub fn open(key: &[u8; 32], nonce: &[u8; NONCE_LEN], ciphertext: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = XChaCha20Poly1305::new(key.as_slice().try_into().expect("32-byte key"));
    let xnonce: &XNonce = nonce.as_slice().try_into().expect("24-byte nonce");
    cipher
        .decrypt(xnonce, ciphertext)
        .map_err(|_| "decryption failed: wrong key or corrupted data".to_string())
}

const FILE_MAGIC: &[u8; 4] = b"INKD";
const FILE_FORMAT_VERSION: u8 = 1;
const DEK_WRAP_CIPHERTEXT_LEN: usize = DEK_LEN + 16; // + Poly1305 tag
const FILE_HEADER_LEN: usize = 4 + 1 + NONCE_LEN + DEK_WRAP_CIPHERTEXT_LEN + NONCE_LEN;

/// Encodes `plaintext` as a self-contained encrypted file: a per-file random
/// DEK, wrapped under `mk`, followed by `plaintext` encrypted under that DEK.
/// Used for both encrypted documents and encrypted assets — see
/// `docs/FORMAT.md`'s "加密文档文件" section for the exact byte layout.
pub fn encode_encrypted_file(mk: &[u8; MK_LEN], plaintext: &[u8]) -> Vec<u8> {
    let dek = random_bytes::<DEK_LEN>();
    let dek_envelope = seal(mk, &dek);
    let content_envelope = seal(&dek, plaintext);

    let mut bytes = Vec::with_capacity(FILE_HEADER_LEN + content_envelope.ciphertext.len());
    bytes.extend_from_slice(FILE_MAGIC);
    bytes.push(FILE_FORMAT_VERSION);
    bytes.extend_from_slice(&dek_envelope.nonce);
    bytes.extend_from_slice(&dek_envelope.ciphertext);
    bytes.extend_from_slice(&content_envelope.nonce);
    bytes.extend_from_slice(&content_envelope.ciphertext);
    bytes
}

/// Decodes a file produced by [`encode_encrypted_file`].
pub fn decode_encrypted_file(mk: &[u8; MK_LEN], bytes: &[u8]) -> Result<Vec<u8>, String> {
    if bytes.len() < FILE_HEADER_LEN {
        return Err("corrupted encrypted file: too short".to_string());
    }
    if &bytes[0..4] != FILE_MAGIC {
        return Err("corrupted encrypted file: bad magic".to_string());
    }
    let format_version = bytes[4];
    if format_version != FILE_FORMAT_VERSION {
        return Err(format!(
            "encrypted file requires a newer version of Inkstone (formatVersion {format_version})"
        ));
    }
    let dek_nonce: [u8; NONCE_LEN] = bytes[5..5 + NONCE_LEN].try_into().unwrap();
    let dek_ciphertext_end = 5 + NONCE_LEN + DEK_WRAP_CIPHERTEXT_LEN;
    let dek_ciphertext = &bytes[5 + NONCE_LEN..dek_ciphertext_end];
    let content_nonce: [u8; NONCE_LEN] = bytes[dek_ciphertext_end..dek_ciphertext_end + NONCE_LEN]
        .try_into()
        .unwrap();
    let content_ciphertext = &bytes[dek_ciphertext_end + NONCE_LEN..];

    let dek_bytes = open(mk, &dek_nonce, dek_ciphertext)?;
    let dek: [u8; DEK_LEN] = dek_bytes
        .try_into()
        .map_err(|_| "corrupted encrypted file: invalid DEK length".to_string())?;
    open(&dek, &content_nonce, content_ciphertext)
}

fn crockford_encoding() -> data_encoding::Encoding {
    let mut spec = data_encoding::Specification::new();
    spec.symbols.push_str("0123456789ABCDEFGHJKMNPQRSTVWXYZ");
    spec.encoding()
        .expect("hardcoded Crockford alphabet is valid")
}

/// Generates a new recovery code: `RECOVERY_CODE_LEN` random bytes, formatted
/// as dash-grouped Crockford Base32 for legible manual transcription.
pub fn generate_recovery_code() -> (String, [u8; RECOVERY_CODE_LEN]) {
    let bytes = random_bytes::<RECOVERY_CODE_LEN>();
    (format_recovery_code(&bytes), bytes)
}

pub fn format_recovery_code(bytes: &[u8; RECOVERY_CODE_LEN]) -> String {
    let encoded = crockford_encoding().encode(bytes);
    encoded
        .as_bytes()
        .chunks(4)
        .map(|chunk| std::str::from_utf8(chunk).expect("ASCII"))
        .collect::<Vec<_>>()
        .join("-")
}

/// Parses a user-typed recovery code back into raw bytes. Tolerant of
/// lowercase input, surrounding whitespace, and dashes in any position;
/// applies the standard Crockford confusable substitutions (O→0, I/L→1).
pub fn parse_recovery_code(input: &str) -> Result<[u8; RECOVERY_CODE_LEN], String> {
    let normalized: String = input
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .map(|c| match c.to_ascii_uppercase() {
            'O' => '0',
            'I' | 'L' => '1',
            upper => upper,
        })
        .collect();
    let bytes = crockford_encoding()
        .decode(normalized.as_bytes())
        .map_err(|_| "invalid recovery code".to_string())?;
    bytes
        .try_into()
        .map_err(|_| "invalid recovery code length".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Fast, insecure Argon2 parameters for unit tests only — production
    // paths always go through Argon2Params::DEFAULT.
    const TEST_ARGON2_PARAMS: Argon2Params = Argon2Params {
        memory_kib: 8,
        iterations: 1,
        parallelism: 1,
    };

    #[test]
    fn seal_open_roundtrip() {
        let key = random_bytes::<32>();
        let envelope = seal(&key, b"hello inkstone");
        let plaintext = open(&key, &envelope.nonce, &envelope.ciphertext).unwrap();
        assert_eq!(plaintext, b"hello inkstone");
    }

    #[test]
    fn open_rejects_wrong_key() {
        let key = random_bytes::<32>();
        let other_key = random_bytes::<32>();
        let envelope = seal(&key, b"secret");
        assert!(open(&other_key, &envelope.nonce, &envelope.ciphertext).is_err());
    }

    #[test]
    fn open_rejects_tampered_ciphertext() {
        let key = random_bytes::<32>();
        let mut envelope = seal(&key, b"secret");
        let last = envelope.ciphertext.len() - 1;
        envelope.ciphertext[last] ^= 0xFF;
        assert!(open(&key, &envelope.nonce, &envelope.ciphertext).is_err());
    }

    #[test]
    fn nonces_are_not_reused() {
        let key = random_bytes::<32>();
        let a = seal(&key, b"one");
        let b = seal(&key, b"two");
        assert_ne!(a.nonce, b.nonce);
    }

    #[test]
    fn password_kek_is_deterministic_for_same_inputs() {
        let salt = random_bytes::<SALT_LEN>();
        let params = TEST_ARGON2_PARAMS;
        let a = derive_kek_from_password("correct horse", &salt, &params).unwrap();
        let b = derive_kek_from_password("correct horse", &salt, &params).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn password_kek_differs_for_different_passwords() {
        let salt = random_bytes::<SALT_LEN>();
        let params = TEST_ARGON2_PARAMS;
        let a = derive_kek_from_password("password one", &salt, &params).unwrap();
        let b = derive_kek_from_password("password two", &salt, &params).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn recovery_kek_matches_between_generation_and_reentry() {
        let salt = random_bytes::<SALT_LEN>();
        let (code_str, code_bytes) = generate_recovery_code();
        let reparsed = parse_recovery_code(&code_str).unwrap();
        assert_eq!(code_bytes, reparsed);

        let a = derive_kek_from_recovery(&code_bytes, &salt);
        let b = derive_kek_from_recovery(&reparsed, &salt);
        assert_eq!(a, b);
    }

    #[test]
    fn recovery_code_parse_is_case_and_whitespace_tolerant() {
        let (code_str, code_bytes) = generate_recovery_code();
        let messy = format!("  {} \n", code_str.to_lowercase());
        assert_eq!(parse_recovery_code(&messy).unwrap(), code_bytes);
    }

    #[test]
    fn recovery_code_parse_rejects_garbage() {
        assert!(parse_recovery_code("not-a-real-code").is_err());
    }

    #[test]
    fn encrypted_file_roundtrip() {
        let mk = random_bytes::<MK_LEN>();
        let encoded = encode_encrypted_file(&mk, b"top secret document body");
        let decoded = decode_encrypted_file(&mk, &encoded).unwrap();
        assert_eq!(decoded, b"top secret document body");
    }

    #[test]
    fn encrypted_file_rejects_wrong_mk() {
        let mk = random_bytes::<MK_LEN>();
        let other_mk = random_bytes::<MK_LEN>();
        let encoded = encode_encrypted_file(&mk, b"secret");
        assert!(decode_encrypted_file(&other_mk, &encoded).is_err());
    }

    #[test]
    fn encrypted_file_rejects_truncated_input() {
        let mk = random_bytes::<MK_LEN>();
        let encoded = encode_encrypted_file(&mk, b"secret");
        assert!(decode_encrypted_file(&mk, &encoded[..10]).is_err());
    }

    #[test]
    fn encrypted_file_rejects_bad_magic() {
        let mk = random_bytes::<MK_LEN>();
        let mut encoded = encode_encrypted_file(&mk, b"secret");
        encoded[0] = b'X';
        assert!(decode_encrypted_file(&mk, &encoded).is_err());
    }

    #[test]
    fn encrypted_file_each_dek_is_independent() {
        let mk = random_bytes::<MK_LEN>();
        let a = encode_encrypted_file(&mk, b"same plaintext");
        let b = encode_encrypted_file(&mk, b"same plaintext");
        // Independent random DEK + nonces per file, even for identical plaintext.
        assert_ne!(a, b);
    }

    #[test]
    fn index_key_differs_per_vault_id() {
        let mk = random_bytes::<MK_LEN>();
        let a = derive_index_key(&mk, "vault-a");
        let b = derive_index_key(&mk, "vault-b");
        assert_ne!(a, b);
    }
}
