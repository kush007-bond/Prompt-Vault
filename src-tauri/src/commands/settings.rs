use crate::AppState;
use crate::commands::types::{Setting, GetSettingRequest, SetSettingRequest};
use tauri::State;
use rusqlite::params;
use log::info;

#[tauri::command]
pub fn get_setting(state: State<AppState>, key: String) -> Result<Option<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let result = db.conn.query_row(
        "SELECT value FROM settings WHERE key = ?",
        params![key],
        |row| row.get::<_, String>(0),
    );

    match result {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn set_setting(state: State<AppState>, request: SetSettingRequest) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    db.conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
        params![request.key, request.value],
    ).map_err(|e| e.to_string())?;

    info!("Set setting: {} = {}", request.key, request.value);
    Ok(())
}

#[tauri::command]
pub fn get_all_settings(state: State<AppState>) -> Result<Vec<Setting>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = db.conn.prepare(
        "SELECT key, value FROM settings ORDER BY key ASC"
    ).map_err(|e| e.to_string())?;

    let settings = stmt.query_map([], |row| {
        Ok(Setting {
            key: row.get(0)?,
            value: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?;

    settings.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}