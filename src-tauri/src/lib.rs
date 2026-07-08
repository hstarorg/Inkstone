mod assets;
mod docs;
mod vault;

use serde_json::{json, Value};
use tauri::Manager;

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

#[tauri::command]
fn vault_create(app: tauri::AppHandle, path: String) -> Result<vault::VaultInfo, String> {
    let info = vault::create(&path)?;
    remember_vault(&app, &info.path);
    allow_asset_dir(&app, &info.path);
    Ok(info)
}

#[tauri::command]
fn vault_open(app: tauri::AppHandle, path: String) -> Result<vault::VaultInfo, String> {
    let info = vault::open(&path)?;
    remember_vault(&app, &info.path);
    allow_asset_dir(&app, &info.path);
    Ok(info)
}

#[tauri::command]
fn asset_save(vault: String, data: String, ext: String) -> Result<String, String> {
    assets::save(&vault, &data, &ext)
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
fn doc_list(vault: String) -> Result<docs::DocList, String> {
    docs::list(&vault)
}

#[tauri::command]
fn doc_create(vault: String) -> Result<Value, String> {
    docs::create(&vault)
}

#[tauri::command]
fn doc_read(vault: String, id: String) -> Result<Value, String> {
    docs::read(&vault, &id)
}

#[tauri::command]
fn doc_write(vault: String, id: String, content: Value) -> Result<docs::DocMeta, String> {
    docs::write_content(&vault, &id, content)
}

#[tauri::command]
fn doc_trash(vault: String, id: String) -> Result<(), String> {
    docs::trash(&vault, &id)
}

#[tauri::command]
fn doc_restore(vault: String, id: String) -> Result<(), String> {
    docs::restore(&vault, &id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            vault_create,
            vault_open,
            vault_recent,
            doc_list,
            doc_create,
            doc_read,
            doc_write,
            doc_trash,
            doc_restore,
            asset_save,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
