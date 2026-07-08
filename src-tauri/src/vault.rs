use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

pub const VAULT_FORMAT_VERSION: u64 = 1;

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
