use crate::AppState;
use crate::commands::types::{Collection, CreateCollectionRequest, UpdateCollectionRequest};
use tauri::State;
use rusqlite::params;
use log::{info, error};
use uuid::Uuid;

#[tauri::command]
pub fn get_all_collections(state: State<AppState>) -> Result<Vec<Collection>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = db.conn.prepare(
        "SELECT id, name, parent_id, color, icon, is_smart, smart_filter, sort_order, created_at 
         FROM collections ORDER BY sort_order ASC, name ASC"
    ).map_err(|e| e.to_string())?;

    let collections = stmt.query_map([], |row| {
        Ok(Collection {
            id: row.get(0)?,
            name: row.get(1)?,
            parent_id: row.get(2)?,
            color: row.get(3)?,
            icon: row.get(4)?,
            is_smart: row.get::<_, i32>(5)? != 0,
            smart_filter: row.get(6)?,
            sort_order: row.get(7)?,
            created_at: row.get(8)?,
        })
    }).map_err(|e| e.to_string())?;

    collections.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_collection(state: State<AppState>, request: CreateCollectionRequest) -> Result<Collection, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    info!("Creating collection: {} - {}", id, request.name);

    db.conn.execute(
        "INSERT INTO collections (id, name, parent_id, color, icon, created_at, sort_order)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
        params![id, request.name, request.parent_id, request.color, request.icon, now, 0.0],
    ).map_err(|e| { error!("Failed to create collection: {}", e); e.to_string() })?;

    info!("Created collection: {}", id);

    db.conn.query_row(
        "SELECT id, name, parent_id, color, icon, is_smart, smart_filter, sort_order, created_at 
         FROM collections WHERE id = ?",
        params![id],
        |row| {
            Ok(Collection {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                color: row.get(3)?,
                icon: row.get(4)?,
                is_smart: row.get::<_, i32>(5)? != 0,
                smart_filter: row.get(6)?,
                sort_order: row.get(7)?,
                created_at: row.get(8)?,
            })
        }
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_collection(state: State<AppState>, request: UpdateCollectionRequest) -> Result<Collection, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    db.conn.execute(
        "UPDATE collections SET name = ?, parent_id = ?, color = ?, icon = ? WHERE id = ?",
        params![request.name, request.parent_id, request.color, request.icon, request.id],
    ).map_err(|e| e.to_string())?;

    info!("Updated collection: {}", request.id);

    db.conn.query_row(
        "SELECT id, name, parent_id, color, icon, is_smart, smart_filter, sort_order, created_at 
         FROM collections WHERE id = ?",
        params![request.id],
        |row| {
            Ok(Collection {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                color: row.get(3)?,
                icon: row.get(4)?,
                is_smart: row.get::<_, i32>(5)? != 0,
                smart_filter: row.get(6)?,
                sort_order: row.get(7)?,
                created_at: row.get(8)?,
            })
        }
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_collection(state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    db.conn.execute("DELETE FROM collections WHERE id = ?", params![id])
        .map_err(|e| e.to_string())?;

    info!("Deleted collection: {}", id);
    Ok(())
}