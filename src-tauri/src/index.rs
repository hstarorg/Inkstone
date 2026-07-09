use crate::crypto::{self, MK_LEN};
use crate::docs;
use rusqlite::Connection;
use serde::Serialize;
use serde_json::Value;
use std::path::Path;

const HIGHLIGHT_OPEN: char = '\u{1}';
const HIGHLIGHT_CLOSE: char = '\u{2}';
const SNIPPET_CONTEXT_CHARS: usize = 30;
const MAX_HITS: usize = 20;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub id: String,
    pub title: String,
    pub snippet: String,
}

fn open(vault: &str, key: Option<&[u8; MK_LEN]>, vault_id: &str) -> Result<Connection, String> {
    let dir = Path::new(vault).join(".inkstone");
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let conn = Connection::open(dir.join("index.db"))
        .map_err(|error| format!("failed to open index: {error}"))?;
    if let Some(mk) = key {
        let index_key = crypto::derive_index_key(mk, vault_id);
        let hex: String = index_key.iter().map(|b| format!("{b:02x}")).collect();
        // SQLCipher's raw-key syntax (`x'...'`) must not be escaped like a
        // regular string literal, so this is issued directly rather than via
        // a bound parameter. `hex` is our own hex encoding, not user input.
        conn.execute_batch(&format!("PRAGMA key = \"x'{hex}'\";"))
            .map_err(|error| format!("failed to unlock index: {error}"))?;
    }
    conn.execute_batch(
        "CREATE VIRTUAL TABLE IF NOT EXISTS docs_fts
         USING fts5(id UNINDEXED, title, body, tokenize='trigram');",
    )
    .map_err(|error| format!("failed to init index schema: {error}"))?;
    Ok(conn)
}

fn body_text(content: &Value) -> String {
    let mut body = String::new();
    if let Some(blocks) = content.get("content").and_then(Value::as_array) {
        for block in blocks {
            let mut text = String::new();
            docs::collect_text(block, &mut text);
            if !text.is_empty() {
                body.push_str(&text);
                body.push('\n');
            }
        }
    }
    body
}

pub fn index_doc(
    vault: &str,
    id: &str,
    title: &str,
    content: &Value,
    key: Option<&[u8; MK_LEN]>,
    vault_id: &str,
) -> Result<(), String> {
    let conn = open(vault, key, vault_id)?;
    upsert(&conn, id, title, &body_text(content))
}

fn upsert(conn: &Connection, id: &str, title: &str, body: &str) -> Result<(), String> {
    conn.execute("DELETE FROM docs_fts WHERE id = ?1", [id])
        .map_err(|error| error.to_string())?;
    conn.execute(
        "INSERT INTO docs_fts (id, title, body) VALUES (?1, ?2, ?3)",
        [id, title, body],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn remove_doc(
    vault: &str,
    id: &str,
    key: Option<&[u8; MK_LEN]>,
    vault_id: &str,
) -> Result<(), String> {
    let conn = open(vault, key, vault_id)?;
    conn.execute("DELETE FROM docs_fts WHERE id = ?1", [id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn rebuild(vault: &str, key: Option<&[u8; MK_LEN]>, vault_id: &str) -> Result<(), String> {
    let conn = open(vault, key, vault_id)?;
    conn.execute("DELETE FROM docs_fts", [])
        .map_err(|error| error.to_string())?;
    for meta in docs::list(vault, key)?.docs {
        let doc = docs::read(vault, &meta.id, key)?;
        let body = doc.get("content").map(body_text).unwrap_or_default();
        upsert(&conn, &meta.id, &meta.title, &body)?;
    }
    Ok(())
}

fn escape_like(query: &str) -> String {
    query
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn like_snippet(body: &str, query: &str) -> String {
    let lower_body = body.to_lowercase();
    let lower_query = query.to_lowercase();
    let Some(byte_start) = lower_body.find(&lower_query) else {
        return body.chars().take(SNIPPET_CONTEXT_CHARS * 2).collect();
    };
    let prefix: String = body[..byte_start]
        .chars()
        .rev()
        .take(SNIPPET_CONTEXT_CHARS)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    let matched: String = body[byte_start..]
        .chars()
        .take(lower_query.chars().count())
        .collect();
    let suffix: String = body[byte_start + matched.len()..]
        .chars()
        .take(SNIPPET_CONTEXT_CHARS)
        .collect();
    format!("{prefix}{HIGHLIGHT_OPEN}{matched}{HIGHLIGHT_CLOSE}{suffix}")
}

pub fn search(
    vault: &str,
    query: &str,
    key: Option<&[u8; MK_LEN]>,
    vault_id: &str,
) -> Result<Vec<SearchHit>, String> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    let conn = open(vault, key, vault_id)?;

    if query.chars().count() >= 3 {
        let fts_query = format!("\"{}\"", query.replace('"', "\"\""));
        let mut stmt = conn
            .prepare(
                "SELECT id, title, snippet(docs_fts, 2, char(1), char(2), '…', 12)
                 FROM docs_fts WHERE docs_fts MATCH ?1 ORDER BY rank LIMIT ?2",
            )
            .map_err(|error| error.to_string())?;
        let hits = stmt
            .query_map((&fts_query, MAX_HITS), |row| {
                Ok(SearchHit {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    snippet: row.get(2)?,
                })
            })
            .map_err(|error| error.to_string())?
            .filter_map(Result::ok)
            .collect();
        return Ok(hits);
    }

    let pattern = format!("%{}%", escape_like(query));
    let mut stmt = conn
        .prepare(
            "SELECT id, title, body FROM docs_fts
             WHERE title LIKE ?1 ESCAPE '\\' OR body LIKE ?1 ESCAPE '\\' LIMIT ?2",
        )
        .map_err(|error| error.to_string())?;
    let hits = stmt
        .query_map((&pattern, MAX_HITS), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
        .map(|(id, title, body)| SearchHit {
            snippet: like_snippet(&body, query),
            id,
            title,
        })
        .collect();
    Ok(hits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault;
    use serde_json::json;

    fn heading_doc(text: &str) -> Value {
        json!({
            "type": "doc",
            "content": [
                { "type": "heading", "attrs": { "level": 1 },
                  "content": [{ "type": "text", "text": text }] },
                { "type": "paragraph",
                  "content": [{ "type": "text", "text": "shared body words 中文正文内容" }] },
            ],
        })
    }

    fn test_vault_with_doc(title: &str) -> (tempfile::TempDir, String, String) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        vault::create(&path).unwrap();
        let doc = docs::create(&path, None).unwrap();
        let id = doc["id"].as_str().unwrap().to_string();
        docs::write_content(&path, &id, heading_doc(title), None).unwrap();
        (dir, path, id)
    }

    fn test_encrypted_vault_with_doc(
        title: &str,
    ) -> (tempfile::TempDir, String, String, [u8; MK_LEN], String) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let (info, _recovery) = vault::create_encrypted(&path, "correct horse").unwrap();
        let mk = vault::unlock_with_password(&path, "correct horse").unwrap();
        let doc = docs::create(&path, Some(&mk)).unwrap();
        let id = doc["id"].as_str().unwrap().to_string();
        docs::write_content(&path, &id, heading_doc(title), Some(&mk)).unwrap();
        (dir, path, id, mk, info.vault_id)
    }

    #[test]
    fn rebuild_and_fts_search() {
        let (_dir, vault, id) = test_vault_with_doc("Cloudflare deployment notes");
        rebuild(&vault, None, "").unwrap();

        let hits = search(&vault, "deployment", None, "").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, id);
        assert!(hits[0].snippet.contains('\u{1}'));
    }

    #[test]
    fn chinese_trigram_and_short_query() {
        let (_dir, vault, id) = test_vault_with_doc("数据库高可用研究");
        rebuild(&vault, None, "").unwrap();

        let long = search(&vault, "高可用研究", None, "").unwrap();
        assert_eq!(long.len(), 1);
        assert_eq!(long[0].id, id);

        let short = search(&vault, "正文", None, "").unwrap();
        assert_eq!(short.len(), 1);
        assert!(short[0].snippet.contains('\u{1}'));
    }

    #[test]
    fn incremental_index_and_remove() {
        let (_dir, vault, id) = test_vault_with_doc("first version");
        rebuild(&vault, None, "").unwrap();

        let updated = heading_doc("second edition");
        docs::write_content(&vault, &id, updated.clone(), None).unwrap();
        index_doc(&vault, &id, "second edition", &updated, None, "").unwrap();

        assert!(search(&vault, "first version", None, "")
            .unwrap()
            .is_empty());
        assert_eq!(search(&vault, "second edition", None, "").unwrap().len(), 1);

        remove_doc(&vault, &id, None, "").unwrap();
        assert!(search(&vault, "second edition", None, "")
            .unwrap()
            .is_empty());
    }

    #[test]
    fn empty_query_returns_nothing() {
        let (_dir, vault, _id) = test_vault_with_doc("anything");
        rebuild(&vault, None, "").unwrap();
        assert!(search(&vault, "  ", None, "").unwrap().is_empty());
    }

    #[test]
    fn encrypted_rebuild_and_search() {
        let (_dir, vault, id, mk, vault_id) =
            test_encrypted_vault_with_doc("Cloudflare deployment notes");
        rebuild(&vault, Some(&mk), &vault_id).unwrap();

        let hits = search(&vault, "deployment", Some(&mk), &vault_id).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, id);
    }

    #[test]
    fn encrypted_index_has_no_plaintext_leakage_on_disk() {
        let (_dir, vault, _id, mk, vault_id) =
            test_encrypted_vault_with_doc("a very secret unique title");
        rebuild(&vault, Some(&mk), &vault_id).unwrap();

        let raw = std::fs::read(Path::new(&vault).join(".inkstone").join("index.db")).unwrap();
        assert!(!raw.windows(6).any(|w| w == b"secret"));
    }

    #[test]
    fn encrypted_search_fails_without_key() {
        let (_dir, vault, _id, mk, vault_id) = test_encrypted_vault_with_doc("secret");
        rebuild(&vault, Some(&mk), &vault_id).unwrap();
        assert!(search(&vault, "secret", None, &vault_id).is_err());
    }
}
