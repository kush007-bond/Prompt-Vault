use crate::AppState;
use crate::commands::types::{Tag, CreateTagRequest, UpdateTagRequest};
use tauri::State;
use rusqlite::params;
use log::{info, error};
use uuid::Uuid;

#[tauri::command]
pub fn get_all_tags(state: State<AppState>) -> Result<Vec<Tag>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = db.conn.prepare(
        "SELECT id, name, color FROM tags ORDER BY name ASC"
    ).map_err(|e| e.to_string())?;

    let tags = stmt.query_map([], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?;

    tags.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_tag(state: State<AppState>, request: CreateTagRequest) -> Result<Tag, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();

    info!("Creating tag: {} - {}", id, request.name);

    db.conn.execute(
        "INSERT INTO tags (id, name, color) VALUES (?, ?, ?)",
        params![id, request.name, request.color],
    ).map_err(|e| { error!("Failed to create tag: {}", e); e.to_string() })?;

    info!("Created tag: {}", id);

    db.conn.query_row(
        "SELECT id, name, color FROM tags WHERE id = ?",
        params![id],
        |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
            })
        }
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_tag(state: State<AppState>, request: UpdateTagRequest) -> Result<Tag, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    db.conn.execute(
        "UPDATE tags SET name = ?, color = ? WHERE id = ?",
        params![request.name, request.color, request.id],
    ).map_err(|e| e.to_string())?;

    info!("Updated tag: {}", request.id);

    db.conn.query_row(
        "SELECT id, name, color FROM tags WHERE id = ?",
        params![request.id],
        |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
            })
        }
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_tag(state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    db.conn.execute("DELETE FROM tags WHERE id = ?", params![id])
        .map_err(|e| e.to_string())?;

    info!("Deleted tag: {}", id);
    Ok(())
}

#[tauri::command]
pub fn add_tag_to_prompt(state: State<AppState>, prompt_id: String, tag_id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    db.conn.execute(
        "INSERT OR IGNORE INTO prompt_tags (prompt_id, tag_id) VALUES (?, ?)",
        params![prompt_id, tag_id],
    ).map_err(|e| e.to_string())?;

    info!("Added tag {} to prompt {}", tag_id, prompt_id);
    Ok(())
}

#[tauri::command]
pub fn remove_tag_from_prompt(state: State<AppState>, prompt_id: String, tag_id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    db.conn.execute(
        "DELETE FROM prompt_tags WHERE prompt_id = ? AND tag_id = ?",
        params![prompt_id, tag_id],
    ).map_err(|e| e.to_string())?;

    info!("Removed tag {} from prompt {}", tag_id, prompt_id);
    Ok(())
}

#[tauri::command]
pub fn get_prompt_tags(state: State<AppState>, prompt_id: String) -> Result<Vec<Tag>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = db.conn.prepare(
        "SELECT t.id, t.name, t.color FROM tags t
         JOIN prompt_tags pt ON t.id = pt.tag_id
         WHERE pt.prompt_id = ?
         ORDER BY t.name ASC"
    ).map_err(|e| e.to_string())?;

    let tags = stmt.query_map(params![prompt_id], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?;

    tags.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}