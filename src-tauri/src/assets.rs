use crate::vault::write_atomic;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub fn save(vault: &str, data_base64: &str, ext: &str) -> Result<String, String> {
    let bytes = STANDARD
        .decode(data_base64)
        .map_err(|error| format!("invalid base64 payload: {error}"))?;

    let digest = Sha256::digest(&bytes);
    let hex: String = digest[..16].iter().map(|b| format!("{b:02x}")).collect();

    let ext: String = ext
        .to_lowercase()
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .take(8)
        .collect();
    let ext = if ext.is_empty() {
        "bin".to_string()
    } else {
        ext
    };

    let dir = Path::new(vault).join("assets");
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;

    let name = format!("{hex}.{ext}");
    let path = dir.join(&name);
    if !path.exists() {
        write_atomic(&path, &bytes)?;
    }
    Ok(name)
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

    #[test]
    fn save_is_content_addressed_and_deduplicates() {
        let (_dir, vault) = test_vault();
        let data = STANDARD.encode(b"picture bytes");
        let first = save(&vault, &data, "png").unwrap();
        let second = save(&vault, &data, "png").unwrap();
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
        let name = save(&vault, &data, "../EVIL/../png").unwrap();
        assert!(name.ends_with(".evilpng"));
        let empty = save(&vault, &data, "!!!").unwrap();
        assert!(empty.ends_with(".bin"));
    }

    #[test]
    fn rejects_bad_base64() {
        let (_dir, vault) = test_vault();
        assert!(save(&vault, "not base64!!!", "png").is_err());
    }
}
