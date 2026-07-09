use crate::crypto::{self, MK_LEN};
use crate::vault::write_atomic;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

fn sanitize_ext(ext: &str) -> String {
    let ext: String = ext
        .to_lowercase()
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .take(8)
        .collect();
    if ext.is_empty() {
        "bin".to_string()
    } else {
        ext
    }
}

/// Saves an asset. The filename is always content-addressed by the
/// *plaintext* bytes' hash (dedup is preserved even for encrypted vaults —
/// see `docs/FORMAT.md`'s accepted tradeoff on this narrow metadata
/// exception). When `key` is set, the file's contents are encrypted; the
/// filename gains a `.enc` suffix so it's not mistaken for a viewable image.
pub fn save(
    vault: &str,
    data_base64: &str,
    ext: &str,
    key: Option<&[u8; MK_LEN]>,
) -> Result<String, String> {
    let bytes = STANDARD
        .decode(data_base64)
        .map_err(|error| format!("invalid base64 payload: {error}"))?;

    let digest = Sha256::digest(&bytes);
    let hex: String = digest[..16].iter().map(|b| format!("{b:02x}")).collect();
    let ext = sanitize_ext(ext);

    let dir = Path::new(vault).join("assets");
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;

    let name = match key {
        None => format!("{hex}.{ext}"),
        Some(_) => format!("{hex}.{ext}.enc"),
    };
    let path = dir.join(&name);
    if !path.exists() {
        let out = match key {
            None => bytes,
            Some(mk) => crypto::encode_encrypted_file(mk, &bytes),
        };
        write_atomic(&path, &out)?;
    }
    Ok(name)
}

/// Reads back an asset's plaintext bytes, decrypting if `key` is set.
pub fn read(vault: &str, name: &str, key: Option<&[u8; MK_LEN]>) -> Result<Vec<u8>, String> {
    let path = Path::new(vault).join("assets").join(name);
    let bytes = fs::read(&path).map_err(|error| format!("failed to read asset: {error}"))?;
    match key {
        None => Ok(bytes),
        Some(mk) => crypto::decode_encrypted_file(mk, &bytes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault;

    fn test_vault() -> (tempfile::TempDir, String) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        vault::create(&path).unwrap();
        (dir, path)
    }

    fn test_encrypted_vault() -> (tempfile::TempDir, String, [u8; MK_LEN]) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        vault::create_encrypted(&path, "correct horse battery staple").unwrap();
        let mk = vault::unlock_with_password(&path, "correct horse battery staple").unwrap();
        (dir, path, mk)
    }

    #[test]
    fn save_is_content_addressed_and_deduplicates() {
        let (_dir, vault) = test_vault();
        let data = STANDARD.encode(b"picture bytes");
        let first = save(&vault, &data, "png", None).unwrap();
        let second = save(&vault, &data, "png", None).unwrap();
        assert_eq!(first, second);
        assert!(first.ends_with(".png"));
        assert_eq!(first.len(), 32 + 1 + 3);

        let entries = fs::read_dir(Path::new(&vault).join("assets")).unwrap();
        assert_eq!(entries.count(), 1);
    }

    #[test]
    fn extension_is_sanitized() {
        let (_dir, vault) = test_vault();
        let data = STANDARD.encode(b"x");
        let name = save(&vault, &data, "../EVIL/../png", None).unwrap();
        assert!(name.ends_with(".evilpng"));
        let empty = save(&vault, &data, "!!!", None).unwrap();
        assert!(empty.ends_with(".bin"));
    }

    #[test]
    fn rejects_bad_base64() {
        let (_dir, vault) = test_vault();
        assert!(save(&vault, "not base64!!!", "png", None).is_err());
    }

    #[test]
    fn read_roundtrips_plaintext_asset() {
        let (_dir, vault) = test_vault();
        let data = STANDARD.encode(b"picture bytes");
        let name = save(&vault, &data, "png", None).unwrap();
        assert_eq!(read(&vault, &name, None).unwrap(), b"picture bytes");
    }

    #[test]
    fn encrypted_save_dedupes_by_plaintext_hash_and_has_no_leakage() {
        let (_dir, vault, mk) = test_encrypted_vault();
        let data = STANDARD.encode(b"picture bytes");
        let first = save(&vault, &data, "png", Some(&mk)).unwrap();
        let second = save(&vault, &data, "png", Some(&mk)).unwrap();
        assert_eq!(first, second);
        assert!(first.ends_with(".png.enc"));

        let raw = fs::read(Path::new(&vault).join("assets").join(&first)).unwrap();
        assert!(!raw.windows(b"picture".len()).any(|w| w == b"picture"));
    }

    #[test]
    fn encrypted_read_roundtrip() {
        let (_dir, vault, mk) = test_encrypted_vault();
        let data = STANDARD.encode(b"picture bytes");
        let name = save(&vault, &data, "png", Some(&mk)).unwrap();
        assert_eq!(read(&vault, &name, Some(&mk)).unwrap(), b"picture bytes");
    }

    #[test]
    fn encrypted_read_rejects_wrong_key() {
        let (_dir, vault, mk) = test_encrypted_vault();
        let data = STANDARD.encode(b"picture bytes");
        let name = save(&vault, &data, "png", Some(&mk)).unwrap();
        let wrong = crypto::random_bytes::<MK_LEN>();
        assert!(read(&vault, &name, Some(&wrong)).is_err());
    }
}
