mod assets;
mod crypto;
mod docs;
mod index;
mod session;
mod vault;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::Serialize;
use serde_json::{json, Value};
use tauri::Manager;

struct VaultContext {
    key: Option<[u8; crypto::MK_LEN]>,
    vault_id: String,
}

fn resolve_context(sessions: &session::VaultSessions, vault: &str) -> Result<VaultContext, String> {
    let manifest_path = std::path::Path::new(vault).join("inkstone.json");
    let manifest = vault::read_json(&manifest_path)?;
    let encryption = manifest
        .get("encryption")
        .and_then(Value::as_str)
        .unwrap_or("none");
    let vault_id = manifest
        .get("vaultId")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let key = if encryption == "none" {
        None
    } else {
        Some(
            sessions
                .get(vault)
                .ok_or_else(|| "vault is locked".to_string())?,
        )
    };
    Ok(VaultContext { key, vault_id })
}

fn allow_asset_dir(app: &tauri::AppHandle, vault: &str) {
    let assets_dir = std::path::Path::new(vault).join("assets");
    let _ = app.asset_protocol_scope().allow_directory(assets_dir, true);
}

fn state_file(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("failed to resolve app config dir: {error}"))?;
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir.join("state.json"))
}

fn remember_vault(app: &tauri::AppHandle, path: &str) {
    if let Ok(state_path) = state_file(app) {
        let state = json!({ "recentVault": path });
        let _ = vault::write_atomic(&state_path, state.to_string().as_bytes());
    }
}

fn rebuild_index_best_effort(vault: &str, key: Option<&[u8; crypto::MK_LEN]>, vault_id: &str) {
    if let Err(error) = index::rebuild(vault, key, vault_id) {
        eprintln!("index rebuild failed: {error}");
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct VaultCreateResult {
    info: vault::VaultInfo,
    recovery_code: Option<String>,
}

#[tauri::command]
fn vault_create(
    app: tauri::AppHandle,
    sessions: tauri::State<session::VaultSessions>,
    path: String,
    password: Option<String>,
) -> Result<VaultCreateResult, String> {
    let (info, recovery_code) = match password {
        Some(password) => {
            let (info, recovery_code) = vault::create_encrypted(&path, &password)?;
            let mk = vault::unlock_with_password(&path, &password)?;
            sessions.unlock(&info.path, mk);
            (info, Some(recovery_code))
        }
        None => (vault::create(&path)?, None),
    };
    remember_vault(&app, &info.path);
    allow_asset_dir(&app, &info.path);
    Ok(VaultCreateResult {
        info,
        recovery_code,
    })
}

#[tauri::command]
fn vault_open(app: tauri::AppHandle, path: String) -> Result<vault::VaultInfo, String> {
    let info = vault::open(&path)?;
    remember_vault(&app, &info.path);
    allow_asset_dir(&app, &info.path);
    if info.encryption == "none" {
        rebuild_index_best_effort(&info.path, None, &info.vault_id);
    }
    Ok(info)
}

#[tauri::command]
fn vault_unlock(
    app: tauri::AppHandle,
    sessions: tauri::State<session::VaultSessions>,
    path: String,
    password: String,
) -> Result<vault::VaultInfo, String> {
    let mk = vault::unlock_with_password(&path, &password)?;
    sessions.unlock(&path, mk);
    let info = vault::open(&path)?;
    remember_vault(&app, &info.path);
    allow_asset_dir(&app, &info.path);
    rebuild_index_best_effort(&info.path, Some(&mk), &info.vault_id);
    Ok(info)
}

#[tauri::command]
fn vault_unlock_with_recovery_code(
    app: tauri::AppHandle,
    sessions: tauri::State<session::VaultSessions>,
    path: String,
    recovery_code: String,
) -> Result<vault::VaultInfo, String> {
    let mk = vault::unlock_with_recovery_code(&path, &recovery_code)?;
    sessions.unlock(&path, mk);
    let info = vault::open(&path)?;
    remember_vault(&app, &info.path);
    allow_asset_dir(&app, &info.path);
    rebuild_index_best_effort(&info.path, Some(&mk), &info.vault_id);
    Ok(info)
}

#[tauri::command]
fn vault_lock(sessions: tauri::State<session::VaultSessions>, path: String) {
    sessions.lock_vault(&path);
}

#[tauri::command]
fn vault_is_unlocked(sessions: tauri::State<session::VaultSessions>, path: String) -> bool {
    sessions.is_unlocked(&path)
}

#[tauri::command]
fn vault_change_password(
    sessions: tauri::State<session::VaultSessions>,
    path: String,
    new_password: String,
) -> Result<(), String> {
    let mk = sessions
        .get(&path)
        .ok_or_else(|| "vault is locked".to_string())?;
    vault::change_password(&path, &mk, &new_password)
}

#[tauri::command]
fn vault_recent(app: tauri::AppHandle) -> Option<String> {
    let state_path = state_file(&app).ok()?;
    let state = vault::read_json(&state_path).ok()?;
    state
        .get("recentVault")
        .and_then(Value::as_str)
        .map(str::to_string)
}

#[tauri::command]
fn doc_list(
    sessions: tauri::State<session::VaultSessions>,
    vault: String,
) -> Result<docs::DocList, String> {
    let ctx = resolve_context(&sessions, &vault)?;
    docs::list(&vault, ctx.key.as_ref())
}

#[tauri::command]
fn doc_list_trash(
    sessions: tauri::State<session::VaultSessions>,
    vault: String,
) -> Result<docs::DocList, String> {
    let ctx = resolve_context(&sessions, &vault)?;
    docs::list_trash(&vault, ctx.key.as_ref())
}

#[tauri::command]
fn doc_create(
    sessions: tauri::State<session::VaultSessions>,
    vault: String,
) -> Result<Value, String> {
    let ctx = resolve_context(&sessions, &vault)?;
    let doc = docs::create(&vault, ctx.key.as_ref())?;
    if let (Some(id), Some(content)) = (doc.get("id").and_then(Value::as_str), doc.get("content")) {
        let _ = index::index_doc(
            &vault,
            id,
            "Untitled",
            content,
            ctx.key.as_ref(),
            &ctx.vault_id,
        );
    }
    Ok(doc)
}

#[tauri::command]
fn doc_read(
    sessions: tauri::State<session::VaultSessions>,
    vault: String,
    id: String,
) -> Result<Value, String> {
    let ctx = resolve_context(&sessions, &vault)?;
    docs::read(&vault, &id, ctx.key.as_ref())
}

#[tauri::command]
fn doc_write(
    sessions: tauri::State<session::VaultSessions>,
    vault: String,
    id: String,
    content: Value,
) -> Result<docs::DocMeta, String> {
    let ctx = resolve_context(&sessions, &vault)?;
    let meta = docs::write_content(&vault, &id, content.clone(), ctx.key.as_ref())?;
    let _ = index::index_doc(
        &vault,
        &id,
        &meta.title,
        &content,
        ctx.key.as_ref(),
        &ctx.vault_id,
    );
    Ok(meta)
}

#[tauri::command]
fn doc_trash(
    sessions: tauri::State<session::VaultSessions>,
    vault: String,
    id: String,
) -> Result<(), String> {
    let ctx = resolve_context(&sessions, &vault)?;
    docs::trash(&vault, &id, ctx.key.as_ref())?;
    let _ = index::remove_doc(&vault, &id, ctx.key.as_ref(), &ctx.vault_id);
    Ok(())
}

#[tauri::command]
fn doc_restore(
    sessions: tauri::State<session::VaultSessions>,
    vault: String,
    id: String,
) -> Result<(), String> {
    let ctx = resolve_context(&sessions, &vault)?;
    docs::restore(&vault, &id, ctx.key.as_ref())?;
    if let Ok(doc) = docs::read(&vault, &id, ctx.key.as_ref()) {
        if let Some(content) = doc.get("content") {
            let title = docs::meta_of(&doc).title;
            let _ = index::index_doc(
                &vault,
                &id,
                &title,
                content,
                ctx.key.as_ref(),
                &ctx.vault_id,
            );
        }
    }
    Ok(())
}

#[tauri::command]
fn asset_save(
    sessions: tauri::State<session::VaultSessions>,
    vault: String,
    data: String,
    ext: String,
) -> Result<String, String> {
    let ctx = resolve_context(&sessions, &vault)?;
    assets::save(&vault, &data, &ext, ctx.key.as_ref())
}

#[tauri::command]
fn asset_read(
    sessions: tauri::State<session::VaultSessions>,
    vault: String,
    name: String,
) -> Result<String, String> {
    let ctx = resolve_context(&sessions, &vault)?;
    let bytes = assets::read(&vault, &name, ctx.key.as_ref())?;
    Ok(BASE64.encode(bytes))
}

#[tauri::command]
fn search_docs(
    sessions: tauri::State<session::VaultSessions>,
    vault: String,
    query: String,
) -> Result<Vec<index::SearchHit>, String> {
    let ctx = resolve_context(&sessions, &vault)?;
    index::search(&vault, &query, ctx.key.as_ref(), &ctx.vault_id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(session::VaultSessions::default())
        .invoke_handler(tauri::generate_handler![
            vault_create,
            vault_open,
            vault_unlock,
            vault_unlock_with_recovery_code,
            vault_lock,
            vault_is_unlocked,
            vault_change_password,
            vault_recent,
            doc_list,
            doc_list_trash,
            doc_create,
            doc_read,
            doc_write,
            doc_trash,
            doc_restore,
            asset_save,
            asset_read,
            search_docs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
