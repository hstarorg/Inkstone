use crate::crypto::{self, Argon2Params, MK_LEN, SALT_LEN};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

pub const VAULT_FORMAT_VERSION: u64 = 1;
const KEYS_FORMAT_VERSION: u64 = 1;

const VAULT_MANIFEST: &str = "inkstone.json";
const SUB_DIRS: [&str; 4] = ["docs", "assets", ".trash", ".inkstone"];

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VaultInfo {
    pub path: String,
    pub vault_id: String,
    pub format_version: u64,
    pub encryption: String,
}

pub fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("RFC 3339 formatting of current time cannot fail")
}

pub fn new_id() -> String {
    ulid::Ulid::new().to_string().to_lowercase()
}

pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("invalid path: {}", path.display()))?;
    let tmp_path = path.with_file_name(format!("{file_name}.tmp-{}", new_id()));

    let result = (|| -> std::io::Result<()> {
        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&tmp_path, path)
    })();

    if result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }
    result.map_err(|error| format!("failed to write {}: {error}", path.display()))
}

pub fn read_json(path: &Path) -> Result<Value, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid JSON in {}: {error}", path.display()))
}

pub fn format_version_of(value: &Value, path: &Path) -> Result<u64, String> {
    value
        .get("formatVersion")
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("missing formatVersion in {}", path.display()))
}

fn sweep_tmp_files(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        if name.to_string_lossy().contains(".tmp-") {
            let _ = fs::remove_file(entry.path());
        }
    }
}

fn ensure_layout(root: &Path) -> Result<(), String> {
    for sub in SUB_DIRS {
        fs::create_dir_all(root.join(sub))
            .map_err(|error| format!("failed to create {sub}/: {error}"))?;
    }
    Ok(())
}

fn info_from_manifest(root: &Path, manifest: &Value) -> Result<VaultInfo, String> {
    let manifest_path = root.join(VAULT_MANIFEST);
    let format_version = format_version_of(manifest, &manifest_path)?;
    if format_version > VAULT_FORMAT_VERSION {
        return Err(format!(
            "vault requires a newer version of Inkstone (vault formatVersion {format_version}, supported {VAULT_FORMAT_VERSION})"
        ));
    }
    Ok(VaultInfo {
        path: root.to_string_lossy().into_owned(),
        vault_id: manifest
            .get("vaultId")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        format_version,
        encryption: manifest
            .get("encryption")
            .and_then(Value::as_str)
            .unwrap_or("none")
            .to_string(),
    })
}

pub fn create(path: &str) -> Result<VaultInfo, String> {
    let root = PathBuf::from(path);
    let manifest_path = root.join(VAULT_MANIFEST);
    if manifest_path.exists() {
        return Err(format!("{} is already a vault", root.display()));
    }
    fs::create_dir_all(&root)
        .map_err(|error| format!("failed to create vault directory: {error}"))?;

    let manifest = json!({
        "formatVersion": VAULT_FORMAT_VERSION,
        "vaultId": new_id(),
        "createdAt": now_rfc3339(),
        "encryption": "none",
    });
    write_atomic(
        &manifest_path,
        serde_json::to_string_pretty(&manifest)
            .map_err(|error| error.to_string())?
            .as_bytes(),
    )?;
    ensure_layout(&root)?;
    info_from_manifest(&root, &manifest)
}

fn envelope_to_json(envelope: &crypto::Envelope) -> Value {
    json!({
        "nonce": BASE64.encode(envelope.nonce),
        "ciphertext": BASE64.encode(&envelope.ciphertext),
    })
}

fn envelope_from_json(value: &Value, what: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
    let nonce = value
        .get("nonce")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing {what} nonce"))?;
    let ciphertext = value
        .get("ciphertext")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing {what} ciphertext"))?;
    let nonce = BASE64
        .decode(nonce)
        .map_err(|error| format!("invalid {what} nonce: {error}"))?;
    let ciphertext = BASE64
        .decode(ciphertext)
        .map_err(|error| format!("invalid {what} ciphertext: {error}"))?;
    Ok((nonce, ciphertext))
}

fn open_wrapped_key(value: &Value, kek: &[u8; 32], what: &str) -> Result<[u8; MK_LEN], String> {
    let (nonce, ciphertext) = envelope_from_json(value, what)?;
    let nonce: [u8; crypto::NONCE_LEN] = nonce
        .try_into()
        .map_err(|_| format!("invalid {what} nonce length"))?;
    let mk = crypto::open(kek, &nonce, &ciphertext)?;
    mk.try_into()
        .map_err(|_| format!("invalid {what}: wrong key length after decryption"))
}

/// Creates a new vault protected by a master password. Returns the vault
/// info plus a one-time recovery code the caller MUST show the user — it is
/// never stored or derivable again once this call returns.
pub fn create_encrypted(path: &str, password: &str) -> Result<(VaultInfo, String), String> {
    let root = PathBuf::from(path);
    let manifest_path = root.join(VAULT_MANIFEST);
    if manifest_path.exists() {
        return Err(format!("{} is already a vault", root.display()));
    }
    fs::create_dir_all(&root)
        .map_err(|error| format!("failed to create vault directory: {error}"))?;

    let mk = crypto::random_bytes::<MK_LEN>();

    let password_salt = crypto::random_bytes::<SALT_LEN>();
    let password_params = Argon2Params::DEFAULT;
    let kek_password =
        crypto::derive_kek_from_password(password, &password_salt, &password_params)?;
    let password_envelope = crypto::seal(&kek_password, &mk);

    let (recovery_code, recovery_bytes) = crypto::generate_recovery_code();
    let recovery_salt = crypto::random_bytes::<SALT_LEN>();
    let kek_recovery = crypto::derive_kek_from_recovery(&recovery_bytes, &recovery_salt);
    let recovery_envelope = crypto::seal(&kek_recovery, &mk);

    let manifest = json!({
        "formatVersion": VAULT_FORMAT_VERSION,
        "vaultId": new_id(),
        "createdAt": now_rfc3339(),
        "encryption": "v1",
        "keys": {
            "formatVersion": KEYS_FORMAT_VERSION,
            "password": {
                "kdf": "argon2id",
                "kdfParams": {
                    "memoryKib": password_params.memory_kib,
                    "iterations": password_params.iterations,
                    "parallelism": password_params.parallelism,
                },
                "salt": BASE64.encode(password_salt),
                "wrappedKey": envelope_to_json(&password_envelope),
            },
            "recovery": {
                "kdf": "hkdf-sha256",
                "salt": BASE64.encode(recovery_salt),
                "wrappedKey": envelope_to_json(&recovery_envelope),
            },
        },
    });
    write_atomic(
        &manifest_path,
        serde_json::to_string_pretty(&manifest)
            .map_err(|error| error.to_string())?
            .as_bytes(),
    )?;
    ensure_layout(&root)?;
    let info = info_from_manifest(&root, &manifest)?;
    Ok((info, recovery_code))
}

enum UnlockPath {
    Password,
    Recovery,
}

fn unlock_with(path: &str, secret: &str, which: UnlockPath) -> Result<[u8; MK_LEN], String> {
    let root = PathBuf::from(path);
    let manifest_path = root.join(VAULT_MANIFEST);
    let manifest = read_json(&manifest_path)?;
    if manifest.get("encryption").and_then(Value::as_str) != Some("v1") {
        return Err(format!("{} is not an encrypted vault", root.display()));
    }
    let keys = manifest
        .get("keys")
        .ok_or_else(|| "vault manifest is missing its keys section".to_string())?;

    match which {
        UnlockPath::Password => {
            let entry = keys
                .get("password")
                .ok_or_else(|| "vault manifest is missing the password key entry".to_string())?;
            let salt: [u8; SALT_LEN] = BASE64
                .decode(
                    entry
                        .get("salt")
                        .and_then(Value::as_str)
                        .ok_or_else(|| "missing password salt".to_string())?,
                )
                .map_err(|error| format!("invalid password salt: {error}"))?
                .try_into()
                .map_err(|_| "invalid password salt length".to_string())?;
            let params = entry
                .get("kdfParams")
                .ok_or_else(|| "missing password KDF params".to_string())?;
            let argon2_params = Argon2Params {
                memory_kib: params
                    .get("memoryKib")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| "missing memoryKib".to_string())?
                    as u32,
                iterations: params
                    .get("iterations")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| "missing iterations".to_string())?
                    as u32,
                parallelism: params
                    .get("parallelism")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| "missing parallelism".to_string())?
                    as u32,
            };
            let kek = crypto::derive_kek_from_password(secret, &salt, &argon2_params)?;
            let wrapped = entry
                .get("wrappedKey")
                .ok_or_else(|| "missing password wrappedKey".to_string())?;
            open_wrapped_key(wrapped, &kek, "password-wrapped key")
        }
        UnlockPath::Recovery => {
            let entry = keys
                .get("recovery")
                .ok_or_else(|| "vault manifest is missing the recovery key entry".to_string())?;
            let salt: [u8; SALT_LEN] = BASE64
                .decode(
                    entry
                        .get("salt")
                        .and_then(Value::as_str)
                        .ok_or_else(|| "missing recovery salt".to_string())?,
                )
                .map_err(|error| format!("invalid recovery salt: {error}"))?
                .try_into()
                .map_err(|_| "invalid recovery salt length".to_string())?;
            let recovery_bytes = crypto::parse_recovery_code(secret)
                .map_err(|_| "invalid recovery code".to_string())?;
            let kek = crypto::derive_kek_from_recovery(&recovery_bytes, &salt);
            let wrapped = entry
                .get("wrappedKey")
                .ok_or_else(|| "missing recovery wrappedKey".to_string())?;
            open_wrapped_key(wrapped, &kek, "recovery-wrapped key")
        }
    }
    .map_err(|_| "incorrect password or recovery code".to_string())
}

/// Unlocks an encrypted vault with the master password, returning the
/// decrypted master key. The caller is responsible for holding it (e.g. in
/// session state) and zeroizing it on lock.
pub fn unlock_with_password(path: &str, password: &str) -> Result<[u8; MK_LEN], String> {
    unlock_with(path, password, UnlockPath::Password)
}

/// Unlocks an encrypted vault with the one-time recovery code.
pub fn unlock_with_recovery_code(path: &str, recovery_code: &str) -> Result<[u8; MK_LEN], String> {
    unlock_with(path, recovery_code, UnlockPath::Recovery)
}

/// Re-wraps the master key under a new password (e.g. changing the master
/// password). Does not touch the recovery-code envelope or any document.
pub fn change_password(path: &str, mk: &[u8; MK_LEN], new_password: &str) -> Result<(), String> {
    let root = PathBuf::from(path);
    let manifest_path = root.join(VAULT_MANIFEST);
    let mut manifest = read_json(&manifest_path)?;

    let salt = crypto::random_bytes::<SALT_LEN>();
    let params = Argon2Params::DEFAULT;
    let kek = crypto::derive_kek_from_password(new_password, &salt, &params)?;
    let envelope = crypto::seal(&kek, mk);

    manifest["keys"]["password"] = json!({
        "kdf": "argon2id",
        "kdfParams": {
            "memoryKib": params.memory_kib,
            "iterations": params.iterations,
            "parallelism": params.parallelism,
        },
        "salt": BASE64.encode(salt),
        "wrappedKey": envelope_to_json(&envelope),
    });
    write_atomic(
        &manifest_path,
        serde_json::to_string_pretty(&manifest)
            .map_err(|error| error.to_string())?
            .as_bytes(),
    )
}

pub fn open(path: &str) -> Result<VaultInfo, String> {
    let root = PathBuf::from(path);
    let manifest_path = root.join(VAULT_MANIFEST);
    if !manifest_path.exists() {
        return Err(format!("{} is not an Inkstone vault", root.display()));
    }
    let manifest = read_json(&manifest_path)?;
    let info = info_from_manifest(&root, &manifest)?;

    ensure_layout(&root)?;
    sweep_tmp_files(&root);
    sweep_tmp_files(&root.join("docs"));
    sweep_tmp_files(&root.join(".trash"));

    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_encrypted_then_unlock_with_password() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let (info, _recovery) = create_encrypted(&path, "correct horse battery staple").unwrap();
        assert_eq!(info.encryption, "v1");

        let mk = unlock_with_password(&path, "correct horse battery staple").unwrap();
        assert_eq!(mk.len(), MK_LEN);
    }

    #[test]
    fn unlock_with_password_rejects_wrong_password() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        create_encrypted(&path, "correct horse battery staple").unwrap();
        assert!(unlock_with_password(&path, "wrong password").is_err());
    }

    #[test]
    fn create_encrypted_recovery_code_unlocks_to_same_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let (_info, recovery) = create_encrypted(&path, "correct horse battery staple").unwrap();

        let via_password = unlock_with_password(&path, "correct horse battery staple").unwrap();
        let via_recovery = unlock_with_recovery_code(&path, &recovery).unwrap();
        assert_eq!(via_password, via_recovery);
    }

    #[test]
    fn unlock_with_recovery_code_rejects_wrong_code() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        create_encrypted(&path, "correct horse battery staple").unwrap();
        assert!(
            unlock_with_recovery_code(&path, "0000-0000-0000-0000-0000-0000-0000-0000").is_err()
        );
    }

    #[test]
    fn change_password_rewraps_without_changing_master_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let (_info, recovery) = create_encrypted(&path, "old password").unwrap();
        let mk = unlock_with_password(&path, "old password").unwrap();

        change_password(&path, &mk, "new password").unwrap();

        assert!(unlock_with_password(&path, "old password").is_err());
        let mk_after = unlock_with_password(&path, "new password").unwrap();
        assert_eq!(mk, mk_after);

        // Recovery path is untouched by a password change.
        let via_recovery = unlock_with_recovery_code(&path, &recovery).unwrap();
        assert_eq!(mk, via_recovery);
    }

    #[test]
    fn unlock_rejects_plaintext_vault() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        create(&path).unwrap();
        assert!(unlock_with_password(&path, "anything").is_err());
    }

    #[test]
    fn create_then_open_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("MyVault");
        let created = create(path.to_str().unwrap()).unwrap();
        let opened = open(path.to_str().unwrap()).unwrap();
        assert_eq!(created.vault_id, opened.vault_id);
        assert_eq!(opened.format_version, VAULT_FORMAT_VERSION);
        assert!(path.join("docs").is_dir());
        assert!(path.join(".trash").is_dir());
    }

    #[test]
    fn create_rejects_existing_vault() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        create(&path).unwrap();
        assert!(create(&path).is_err());
    }

    #[test]
    fn open_rejects_non_vault() {
        let dir = tempfile::tempdir().unwrap();
        assert!(open(dir.path().to_str().unwrap()).is_err());
    }

    #[test]
    fn open_rejects_newer_format_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        create(&path).unwrap();
        let manifest_path = dir.path().join(VAULT_MANIFEST);
        let mut manifest = read_json(&manifest_path).unwrap();
        manifest["formatVersion"] = json!(999);
        fs::write(&manifest_path, manifest.to_string()).unwrap();
        assert!(open(&path).is_err());
    }

    #[test]
    fn open_sweeps_orphan_tmp_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        create(&path).unwrap();
        let orphan = dir.path().join("docs").join("abc.json.tmp-xyz");
        fs::write(&orphan, b"partial").unwrap();
        open(&path).unwrap();
        assert!(!orphan.exists());
    }

    #[test]
    fn write_atomic_replaces_content() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("file.json");
        write_atomic(&target, b"one").unwrap();
        write_atomic(&target, b"two").unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "two");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}
