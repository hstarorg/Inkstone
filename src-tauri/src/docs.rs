use crate::crypto::{self, MK_LEN};
use crate::vault::{format_version_of, new_id, now_rfc3339, read_json, write_atomic};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub const DOC_FORMAT_VERSION: u64 = 1;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocMeta {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocList {
    pub docs: Vec<DocMeta>,
    pub warnings: Vec<String>,
}

fn docs_dir(vault: &str) -> PathBuf {
    Path::new(vault).join("docs")
}

fn trash_dir(vault: &str) -> PathBuf {
    Path::new(vault).join(".trash")
}

fn extension(key: Option<&[u8; MK_LEN]>) -> &'static str {
    if key.is_some() {
        "enc"
    } else {
        "json"
    }
}

fn doc_path(vault: &str, id: &str, key: Option<&[u8; MK_LEN]>) -> PathBuf {
    docs_dir(vault).join(format!("{id}.{}", extension(key)))
}

pub(crate) fn collect_text(node: &Value, out: &mut String) {
    if node.get("type").and_then(Value::as_str) == Some("text") {
        if let Some(text) = node.get("text").and_then(Value::as_str) {
            out.push_str(text);
        }
        return;
    }
    if let Some(children) = node.get("content").and_then(Value::as_array) {
        for child in children {
            collect_text(child, out);
        }
    }
}

fn derive_title(doc: &Value) -> String {
    let first_node = doc
        .get("content")
        .and_then(|content| content.get("content"))
        .and_then(Value::as_array)
        .and_then(|nodes| nodes.first());
    let mut title = String::new();
    if let Some(node) = first_node {
        collect_text(node, &mut title);
    }
    let title = title.trim();
    if title.is_empty() {
        "Untitled".to_string()
    } else {
        title.to_string()
    }
}

fn string_field(doc: &Value, key: &str) -> String {
    doc.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

pub(crate) fn meta_of(doc: &Value) -> DocMeta {
    DocMeta {
        id: string_field(doc, "id"),
        title: derive_title(doc),
        created_at: string_field(doc, "createdAt"),
        updated_at: string_field(doc, "updatedAt"),
    }
}

fn read_doc_bytes(path: &Path, key: Option<&[u8; MK_LEN]>) -> Result<Value, String> {
    match key {
        None => read_json(path),
        Some(mk) => {
            let bytes = fs::read(path)
                .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
            let plaintext = crypto::decode_encrypted_file(mk, &bytes)
                .map_err(|error| format!("{}: {error}", path.display()))?;
            serde_json::from_slice(&plaintext)
                .map_err(|error| format!("invalid JSON in {}: {error}", path.display()))
        }
    }
}

fn load_validated_at(path: &Path, id: &str, key: Option<&[u8; MK_LEN]>) -> Result<Value, String> {
    let doc = read_doc_bytes(path, key)?;
    let format_version = format_version_of(&doc, path)?;
    if format_version > DOC_FORMAT_VERSION {
        return Err(format!(
            "document {id} requires a newer version of Inkstone (formatVersion {format_version})"
        ));
    }
    if doc.get("id").and_then(Value::as_str) != Some(id) {
        return Err(format!("document {id}: id does not match file name"));
    }
    Ok(doc)
}

fn load_validated(vault: &str, id: &str, key: Option<&[u8; MK_LEN]>) -> Result<Value, String> {
    load_validated_at(&doc_path(vault, id, key), id, key)
}

fn list_dir(dir: &Path, key: Option<&[u8; MK_LEN]>) -> Result<DocList, String> {
    let entries =
        fs::read_dir(dir).map_err(|error| format!("failed to read {}: {error}", dir.display()))?;

    let suffix = format!(".{}", extension(key));
    let mut docs = Vec::new();
    let mut warnings = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.ends_with(&suffix) || name.contains(".tmp-") {
            continue;
        }
        let id = name.trim_end_matches(&suffix).to_string();
        match load_validated_at(&path, &id, key) {
            Ok(doc) => docs.push(meta_of(&doc)),
            Err(error) => warnings.push(format!("{}: {error}", path.display())),
        }
    }
    docs.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(DocList { docs, warnings })
}

pub fn list(vault: &str, key: Option<&[u8; MK_LEN]>) -> Result<DocList, String> {
    list_dir(&docs_dir(vault), key)
}

/// Lists soft-deleted documents in `.trash/`, so the UI can offer a way to
/// undo a deletion (see FORMAT.md's soft-delete section).
pub fn list_trash(vault: &str, key: Option<&[u8; MK_LEN]>) -> Result<DocList, String> {
    list_dir(&trash_dir(vault), key)
}

pub fn create(vault: &str, key: Option<&[u8; MK_LEN]>) -> Result<Value, String> {
    let id = if key.is_some() {
        crypto::random_token_hex()
    } else {
        new_id()
    };
    let now = now_rfc3339();
    let doc = json!({
        "formatVersion": DOC_FORMAT_VERSION,
        "id": id,
        "createdAt": now,
        "updatedAt": now,
        "tags": [],
        "content": { "type": "doc", "content": [{ "type": "paragraph" }] },
    });
    write_doc_file(vault, &id, &doc, key)?;
    Ok(doc)
}

pub fn read(vault: &str, id: &str, key: Option<&[u8; MK_LEN]>) -> Result<Value, String> {
    load_validated(vault, id, key)
}

pub fn write_content(
    vault: &str,
    id: &str,
    content: Value,
    key: Option<&[u8; MK_LEN]>,
) -> Result<DocMeta, String> {
    let mut doc = load_validated(vault, id, key)?;
    doc["content"] = content;
    doc["updatedAt"] = json!(now_rfc3339());
    write_doc_file(vault, id, &doc, key)?;
    Ok(meta_of(&doc))
}

pub fn trash(vault: &str, id: &str, key: Option<&[u8; MK_LEN]>) -> Result<(), String> {
    let from = doc_path(vault, id, key);
    let to = trash_dir(vault).join(format!("{id}.{}", extension(key)));
    fs::rename(&from, &to).map_err(|error| format!("failed to trash document {id}: {error}"))
}

pub fn restore(vault: &str, id: &str, key: Option<&[u8; MK_LEN]>) -> Result<(), String> {
    let from = trash_dir(vault).join(format!("{id}.{}", extension(key)));
    let to = doc_path(vault, id, key);
    if to.exists() {
        return Err(format!("document {id} already exists in docs/"));
    }
    fs::rename(&from, &to).map_err(|error| format!("failed to restore document {id}: {error}"))
}

fn write_doc_file(
    vault: &str,
    id: &str,
    doc: &Value,
    key: Option<&[u8; MK_LEN]>,
) -> Result<(), String> {
    let json_bytes = serde_json::to_vec(doc).map_err(|error| error.to_string())?;
    let bytes = match key {
        None => json_bytes,
        Some(mk) => crypto::encode_encrypted_file(mk, &json_bytes),
    };
    write_atomic(&doc_path(vault, id, key), &bytes)
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
    fn create_list_read_roundtrip() {
        let (_dir, vault) = test_vault();
        let doc = create(&vault, None).unwrap();
        let id = doc["id"].as_str().unwrap();

        let listed = list(&vault, None).unwrap();
        assert_eq!(listed.docs.len(), 1);
        assert_eq!(listed.docs[0].id, id);
        assert_eq!(listed.docs[0].title, "Untitled");
        assert!(listed.warnings.is_empty());

        let loaded = read(&vault, id, None).unwrap();
        assert_eq!(loaded, doc);
    }

    #[test]
    fn write_content_updates_title_and_preserves_unknown_fields() {
        let (_dir, vault) = test_vault();
        let doc = create(&vault, None).unwrap();
        let id = doc["id"].as_str().unwrap().to_string();

        let path = doc_path(&vault, &id, None);
        let mut raw = read_json(&path).unwrap();
        raw["futureField"] = json!({ "kept": true });
        fs::write(&path, raw.to_string()).unwrap();

        let content = json!({
            "type": "doc",
            "content": [{
                "type": "heading",
                "attrs": { "level": 1 },
                "content": [{ "type": "text", "text": "Hello Inkstone" }],
            }],
        });
        let meta = write_content(&vault, &id, content, None).unwrap();
        assert_eq!(meta.title, "Hello Inkstone");

        let reloaded = read(&vault, &id, None).unwrap();
        assert_eq!(reloaded["futureField"]["kept"], json!(true));
        assert_ne!(reloaded["updatedAt"].as_str().unwrap(), "",);
    }

    #[test]
    fn corrupted_doc_is_reported_not_crashing() {
        let (_dir, vault) = test_vault();
        create(&vault, None).unwrap();
        fs::write(docs_dir(&vault).join("broken.json"), b"{ not json").unwrap();

        let listed = list(&vault, None).unwrap();
        assert_eq!(listed.docs.len(), 1);
        assert_eq!(listed.warnings.len(), 1);
    }

    #[test]
    fn id_mismatch_is_rejected() {
        let (_dir, vault) = test_vault();
        let doc = create(&vault, None).unwrap();
        let id = doc["id"].as_str().unwrap();
        let renamed = docs_dir(&vault).join("zzzzzzzzzzzzzzzzzzzzzzzzzz.json");
        fs::rename(doc_path(&vault, id, None), &renamed).unwrap();

        let listed = list(&vault, None).unwrap();
        assert!(listed.docs.is_empty());
        assert_eq!(listed.warnings.len(), 1);
    }

    #[test]
    fn trash_and_restore() {
        let (_dir, vault) = test_vault();
        let doc = create(&vault, None).unwrap();
        let id = doc["id"].as_str().unwrap();

        trash(&vault, id, None).unwrap();
        assert!(list(&vault, None).unwrap().docs.is_empty());
        assert!(trash_dir(&vault).join(format!("{id}.json")).exists());

        restore(&vault, id, None).unwrap();
        assert_eq!(list(&vault, None).unwrap().docs.len(), 1);
    }

    #[test]
    fn list_trash_shows_trashed_docs_and_restore_clears_it() {
        let (_dir, vault) = test_vault();
        let doc = create(&vault, None).unwrap();
        let id = doc["id"].as_str().unwrap();

        assert!(list_trash(&vault, None).unwrap().docs.is_empty());

        trash(&vault, id, None).unwrap();
        let trashed = list_trash(&vault, None).unwrap();
        assert_eq!(trashed.docs.len(), 1);
        assert_eq!(trashed.docs[0].id, id);

        restore(&vault, id, None).unwrap();
        assert!(list_trash(&vault, None).unwrap().docs.is_empty());
    }

    #[test]
    fn encrypted_list_trash_shows_trashed_docs() {
        let (_dir, vault, mk) = test_encrypted_vault();
        let doc = create(&vault, Some(&mk)).unwrap();
        let id = doc["id"].as_str().unwrap();

        trash(&vault, id, Some(&mk)).unwrap();
        let trashed = list_trash(&vault, Some(&mk)).unwrap();
        assert_eq!(trashed.docs.len(), 1);
        assert_eq!(trashed.docs[0].id, id);
    }

    #[test]
    fn newer_doc_format_version_is_rejected() {
        let (_dir, vault) = test_vault();
        let doc = create(&vault, None).unwrap();
        let id = doc["id"].as_str().unwrap().to_string();
        let path = doc_path(&vault, &id, None);
        let mut raw = read_json(&path).unwrap();
        raw["formatVersion"] = json!(999);
        fs::write(&path, raw.to_string()).unwrap();

        assert!(read(&vault, &id, None).is_err());
    }

    #[test]
    fn encrypted_create_list_read_roundtrip() {
        let (_dir, vault, mk) = test_encrypted_vault();
        let doc = create(&vault, Some(&mk)).unwrap();
        let id = doc["id"].as_str().unwrap();

        let listed = list(&vault, Some(&mk)).unwrap();
        assert_eq!(listed.docs.len(), 1);
        assert_eq!(listed.docs[0].id, id);
        assert!(listed.warnings.is_empty());

        let loaded = read(&vault, id, Some(&mk)).unwrap();
        assert_eq!(loaded, doc);
    }

    #[test]
    fn encrypted_doc_id_is_not_a_ulid_and_matches_filename() {
        let (_dir, vault, mk) = test_encrypted_vault();
        let doc = create(&vault, Some(&mk)).unwrap();
        let id = doc["id"].as_str().unwrap();
        assert_eq!(id.len(), 32); // 16 random bytes, hex-encoded
        assert!(doc_path(&vault, id, Some(&mk)).exists());
    }

    #[test]
    fn encrypted_doc_file_has_no_plaintext_leakage() {
        let (_dir, vault, mk) = test_encrypted_vault();
        let doc = create(&vault, Some(&mk)).unwrap();
        let id = doc["id"].as_str().unwrap();

        write_content(
            &vault,
            id,
            json!({
                "type": "doc",
                "content": [{
                    "type": "heading",
                    "attrs": { "level": 1 },
                    "content": [{ "type": "text", "text": "a very secret title" }],
                }],
            }),
            Some(&mk),
        )
        .unwrap();

        let raw = fs::read(doc_path(&vault, id, Some(&mk))).unwrap();
        assert!(!raw.windows(6).any(|w| w == b"secret"));
        assert!(!raw
            .windows(2)
            .any(|w| w == id.as_bytes().get(0..2).unwrap()));
    }

    #[test]
    fn encrypted_read_rejects_wrong_key() {
        let (_dir, vault, mk) = test_encrypted_vault();
        let doc = create(&vault, Some(&mk)).unwrap();
        let id = doc["id"].as_str().unwrap();

        let wrong_key = crypto::random_bytes::<MK_LEN>();
        assert!(read(&vault, id, Some(&wrong_key)).is_err());
    }

    #[test]
    fn encrypted_corrupted_doc_is_reported_not_crashing() {
        let (_dir, vault, mk) = test_encrypted_vault();
        create(&vault, Some(&mk)).unwrap();
        fs::write(docs_dir(&vault).join("broken.enc"), b"not a valid envelope").unwrap();

        let listed = list(&vault, Some(&mk)).unwrap();
        assert_eq!(listed.docs.len(), 1);
        assert_eq!(listed.warnings.len(), 1);
    }

    #[test]
    fn encrypted_trash_and_restore() {
        let (_dir, vault, mk) = test_encrypted_vault();
        let doc = create(&vault, Some(&mk)).unwrap();
        let id = doc["id"].as_str().unwrap();

        trash(&vault, id, Some(&mk)).unwrap();
        assert!(list(&vault, Some(&mk)).unwrap().docs.is_empty());

        restore(&vault, id, Some(&mk)).unwrap();
        assert_eq!(list(&vault, Some(&mk)).unwrap().docs.len(), 1);
    }
}
