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

fn doc_path(vault: &str, id: &str) -> PathBuf {
    docs_dir(vault).join(format!("{id}.json"))
}

fn collect_text(node: &Value, out: &mut String) {
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

fn meta_of(doc: &Value) -> DocMeta {
    DocMeta {
        id: string_field(doc, "id"),
        title: derive_title(doc),
        created_at: string_field(doc, "createdAt"),
        updated_at: string_field(doc, "updatedAt"),
    }
}

fn load_validated(vault: &str, id: &str) -> Result<Value, String> {
    let path = doc_path(vault, id);
    let doc = read_json(&path)?;
    let format_version = format_version_of(&doc, &path)?;
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

pub fn list(vault: &str) -> Result<DocList, String> {
    let dir = docs_dir(vault);
    let entries =
        fs::read_dir(&dir).map_err(|error| format!("failed to read {}: {error}", dir.display()))?;

    let mut docs = Vec::new();
    let mut warnings = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".json") || name.contains(".tmp-") {
            continue;
        }
        let id = name.trim_end_matches(".json").to_string();
        match load_validated(vault, &id) {
            Ok(doc) => docs.push(meta_of(&doc)),
            Err(error) => warnings.push(format!("{}: {error}", path.display())),
        }
    }
    docs.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(DocList { docs, warnings })
}

pub fn create(vault: &str) -> Result<Value, String> {
    let id = new_id();
    let now = now_rfc3339();
    let doc = json!({
        "formatVersion": DOC_FORMAT_VERSION,
        "id": id,
        "createdAt": now,
        "updatedAt": now,
        "tags": [],
        "content": { "type": "doc", "content": [{ "type": "paragraph" }] },
    });
    write_doc_file(vault, &id, &doc)?;
    Ok(doc)
}

pub fn read(vault: &str, id: &str) -> Result<Value, String> {
    load_validated(vault, id)
}

pub fn write_content(vault: &str, id: &str, content: Value) -> Result<DocMeta, String> {
    let mut doc = load_validated(vault, id)?;
    doc["content"] = content;
    doc["updatedAt"] = json!(now_rfc3339());
    write_doc_file(vault, id, &doc)?;
    Ok(meta_of(&doc))
}

pub fn trash(vault: &str, id: &str) -> Result<(), String> {
    let from = doc_path(vault, id);
    let to = trash_dir(vault).join(format!("{id}.json"));
    fs::rename(&from, &to).map_err(|error| format!("failed to trash document {id}: {error}"))
}

pub fn restore(vault: &str, id: &str) -> Result<(), String> {
    let from = trash_dir(vault).join(format!("{id}.json"));
    let to = doc_path(vault, id);
    if to.exists() {
        return Err(format!("document {id} already exists in docs/"));
    }
    fs::rename(&from, &to).map_err(|error| format!("failed to restore document {id}: {error}"))
}

fn write_doc_file(vault: &str, id: &str, doc: &Value) -> Result<(), String> {
    let bytes = serde_json::to_string_pretty(doc)
        .map_err(|error| error.to_string())?
        .into_bytes();
    write_atomic(&doc_path(vault, id), &bytes)
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
    fn create_list_read_roundtrip() {
        let (_dir, vault) = test_vault();
        let doc = create(&vault).unwrap();
        let id = doc["id"].as_str().unwrap();

        let listed = list(&vault).unwrap();
        assert_eq!(listed.docs.len(), 1);
        assert_eq!(listed.docs[0].id, id);
        assert_eq!(listed.docs[0].title, "Untitled");
        assert!(listed.warnings.is_empty());

        let loaded = read(&vault, id).unwrap();
        assert_eq!(loaded, doc);
    }

    #[test]
    fn write_content_updates_title_and_preserves_unknown_fields() {
        let (_dir, vault) = test_vault();
        let doc = create(&vault).unwrap();
        let id = doc["id"].as_str().unwrap().to_string();

        let path = doc_path(&vault, &id);
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
        let meta = write_content(&vault, &id, content).unwrap();
        assert_eq!(meta.title, "Hello Inkstone");

        let reloaded = read(&vault, &id).unwrap();
        assert_eq!(reloaded["futureField"]["kept"], json!(true));
        assert_ne!(reloaded["updatedAt"].as_str().unwrap(), "",);
    }

    #[test]
    fn corrupted_doc_is_reported_not_crashing() {
        let (_dir, vault) = test_vault();
        create(&vault).unwrap();
        fs::write(docs_dir(&vault).join("broken.json"), b"{ not json").unwrap();

        let listed = list(&vault).unwrap();
        assert_eq!(listed.docs.len(), 1);
        assert_eq!(listed.warnings.len(), 1);
    }

    #[test]
    fn id_mismatch_is_rejected() {
        let (_dir, vault) = test_vault();
        let doc = create(&vault).unwrap();
        let id = doc["id"].as_str().unwrap();
        let renamed = docs_dir(&vault).join("zzzzzzzzzzzzzzzzzzzzzzzzzz.json");
        fs::rename(doc_path(&vault, id), &renamed).unwrap();

        let listed = list(&vault).unwrap();
        assert!(listed.docs.is_empty());
        assert_eq!(listed.warnings.len(), 1);
    }

    #[test]
    fn trash_and_restore() {
        let (_dir, vault) = test_vault();
        let doc = create(&vault).unwrap();
        let id = doc["id"].as_str().unwrap();

        trash(&vault, id).unwrap();
        assert!(list(&vault).unwrap().docs.is_empty());
        assert!(trash_dir(&vault).join(format!("{id}.json")).exists());

        restore(&vault, id).unwrap();
        assert_eq!(list(&vault).unwrap().docs.len(), 1);
    }

    #[test]
    fn newer_doc_format_version_is_rejected() {
        let (_dir, vault) = test_vault();
        let doc = create(&vault).unwrap();
        let id = doc["id"].as_str().unwrap().to_string();
        let path = doc_path(&vault, &id);
        let mut raw = read_json(&path).unwrap();
        raw["formatVersion"] = json!(999);
        fs::write(&path, raw.to_string()).unwrap();

        assert!(read(&vault, &id).is_err());
    }
}
