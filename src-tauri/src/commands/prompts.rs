use crate::AppState;
use crate::commands::types::{Prompt, CreatePromptRequest, UpdatePromptRequest, SearchRequest, PromptVersion};
use tauri::State;
use rusqlite::params;
use log::{info, error};
use uuid::Uuid;

#[tauri::command]
pub fn get_all_prompts(state: State<AppState>) -> Result<Vec<Prompt>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = db.conn.prepare(
        "SELECT id, title, body, model_target, collection_id, is_pinned, is_archived, 
         use_count, sort_order, created_at, updated_at 
         FROM prompts WHERE is_archived = 0 ORDER BY is_pinned DESC, sort_order ASC, updated_at DESC"
    ).map_err(|e| e.to_string())?;

    let prompts = stmt.query_map([], |row| {
        Ok(Prompt {
            id: row.get(0)?,
            title: row.get(1)?,
            body: row.get(2)?,
            model_target: row.get(3)?,
            collection_id: row.get(4)?,
            is_pinned: row.get::<_, i32>(5)? != 0,
            is_archived: row.get::<_, i32>(6)? != 0,
            use_count: row.get(7)?,
            sort_order: row.get(8)?,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
        })
    }).map_err(|e| e.to_string())?;

    prompts.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_prompt(state: State<AppState>, id: String) -> Result<Prompt, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    db.conn.query_row(
        "SELECT id, title, body, model_target, collection_id, is_pinned, is_archived, 
         use_count, sort_order, created_at, updated_at 
         FROM prompts WHERE id = ?",
        params![id],
        |row| {
            Ok(Prompt {
                id: row.get(0)?,
                title: row.get(1)?,
                body: row.get(2)?,
                model_target: row.get(3)?,
                collection_id: row.get(4)?,
                is_pinned: row.get::<_, i32>(5)? != 0,
                is_archived: row.get::<_, i32>(6)? != 0,
                use_count: row.get(7)?,
                sort_order: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        }
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_prompt(state: State<AppState>, request: CreatePromptRequest) -> Result<Prompt, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    info!("Creating prompt: {} - {}", id, request.title);

    db.conn.execute(
        "INSERT INTO prompts (id, title, body, model_target, collection_id, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
        params![id, request.title, request.body, request.model_target, request.collection_id, now, now],
    ).map_err(|e| { error!("Failed to create prompt: {}", e); e.to_string() })?;

    info!("Created prompt: {}", id);

    // Create initial version
    db.conn.execute(
        "INSERT INTO prompt_versions (prompt_id, title_snapshot, body_snapshot, changed_at)
         VALUES (?, ?, ?, ?)",
        params![id, request.title, request.body, now],
    ).map_err(|e| e.to_string())?;

    // Return the created prompt directly
    Ok(Prompt {
        id,
        title: request.title,
        body: request.body,
        model_target: request.model_target,
        collection_id: request.collection_id,
        is_pinned: false,
        is_archived: false,
        use_count: 0,
        sort_order: 0.0,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_prompt(state: State<AppState>, request: UpdatePromptRequest) -> Result<Prompt, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();

    // Get current state for version history
    let current: Prompt = db.conn.query_row(
        "SELECT id, title, body, model_target, collection_id, is_pinned, is_archived, 
         use_count, sort_order, created_at, updated_at FROM prompts WHERE id = ?",
        params![request.id],
        |row| {
            Ok(Prompt {
                id: row.get(0)?,
                title: row.get(1)?,
                body: row.get(2)?,
                model_target: row.get(3)?,
                collection_id: row.get(4)?,
                is_pinned: row.get::<_, i32>(5)? != 0,
                is_archived: row.get::<_, i32>(6)? != 0,
                use_count: row.get(7)?,
                sort_order: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        }
    ).map_err(|e| e.to_string())?;

    db.conn.execute(
        "UPDATE prompts SET title = ?, body = ?, model_target = ?, collection_id = ?, 
         is_pinned = ?, is_archived = ?, updated_at = ? WHERE id = ?",
        params![
            request.title, request.body, request.model_target, request.collection_id,
            request.is_pinned.unwrap_or(current.is_pinned) as i32,
            request.is_archived.unwrap_or(current.is_archived) as i32,
            now, request.id
        ],
    ).map_err(|e| e.to_string())?;

    // Save version if content changed
    if current.title != request.title || current.body != request.body {
        db.conn.execute(
            "INSERT INTO prompt_versions (prompt_id, title_snapshot, body_snapshot, changed_at)
             VALUES (?, ?, ?, ?)",
            params![request.id, request.title, request.body, now],
        ).map_err(|e| e.to_string())?;
    }

    info!("Updated prompt: {}", request.id);
    
    // Return the updated prompt directly
    Ok(Prompt {
        id: request.id.clone(),
        title: request.title,
        body: request.body,
        model_target: request.model_target,
        collection_id: request.collection_id,
        is_pinned: request.is_pinned.unwrap_or(current.is_pinned),
        is_archived: request.is_archived.unwrap_or(current.is_archived),
        use_count: current.use_count,
        sort_order: current.sort_order,
        created_at: current.created_at,
        updated_at: now,
    })
}

#[tauri::command]
pub fn delete_prompt(state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    db.conn.execute("DELETE FROM prompts WHERE id = ?", params![id])
        .map_err(|e| e.to_string())?;

    info!("Deleted prompt: {}", id);
    Ok(())
}

#[tauri::command]
pub fn search_prompts(state: State<AppState>, request: SearchRequest) -> Result<Vec<Prompt>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let limit = request.limit.unwrap_or(50);
    let offset = request.offset.unwrap_or(0);

    // Build query with FTS5
    let mut query = String::from(
        "SELECT p.id, p.title, p.body, p.model_target, p.collection_id, p.is_pinned, 
         p.is_archived, p.use_count, p.sort_order, p.created_at, p.updated_at 
         FROM prompts p"
    );

    let mut conditions = vec!["p.is_archived = 0".to_string()];
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if !request.query.is_empty() {
        query.push_str(" JOIN prompts_fts ON p.rowid = prompts_fts.rowid");
        conditions.push(format!("prompts_fts MATCH ?"));
        params_vec.push(Box::new(format!("{}*", request.query)));
    }

    if let Some(ref col_id) = request.collection_id {
        conditions.push("p.collection_id = ?".to_string());
        params_vec.push(Box::new(col_id.clone()) as Box<dyn rusqlite::ToSql>);
    }

    if let Some(pinned) = request.is_pinned {
        conditions.push(format!("p.is_pinned = {}", pinned as i32));
    }

    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }

    query.push_str(" ORDER BY p.is_pinned DESC, p.sort_order ASC LIMIT ? OFFSET ?");
    params_vec.push(Box::new(limit));
    params_vec.push(Box::new(offset));

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

    let mut stmt = db.conn.prepare(&query).map_err(|e| e.to_string())?;
    let prompts = stmt.query_map(params_refs.as_slice(), |row| {
        Ok(Prompt {
            id: row.get(0)?,
            title: row.get(1)?,
            body: row.get(2)?,
            model_target: row.get(3)?,
            collection_id: row.get(4)?,
            is_pinned: row.get::<_, i32>(5)? != 0,
            is_archived: row.get::<_, i32>(6)? != 0,
            use_count: row.get(7)?,
            sort_order: row.get(8)?,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
        })
    }).map_err(|e| e.to_string())?;

    prompts.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_prompt_versions(state: State<AppState>, prompt_id: String) -> Result<Vec<PromptVersion>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = db.conn.prepare(
        "SELECT id, prompt_id, title_snapshot, body_snapshot, changed_at, change_note 
         FROM prompt_versions WHERE prompt_id = ? ORDER BY changed_at DESC"
    ).map_err(|e| e.to_string())?;

    let versions = stmt.query_map(params![prompt_id], |row| {
        Ok(PromptVersion {
            id: row.get(0)?,
            prompt_id: row.get(1)?,
            title_snapshot: row.get(2)?,
            body_snapshot: row.get(3)?,
            changed_at: row.get(4)?,
            change_note: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;

    versions.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_prompt_version(state: State<AppState>, version_id: String) -> Result<Prompt, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    // Get the version to restore
    let version: PromptVersion = db.conn.query_row(
        "SELECT id, prompt_id, title_snapshot, body_snapshot, changed_at, change_note 
         FROM prompt_versions WHERE id = ?",
        params![version_id],
        |row| {
            Ok(PromptVersion {
                id: row.get(0)?,
                prompt_id: row.get(1)?,
                title_snapshot: row.get(2)?,
                body_snapshot: row.get(3)?,
                changed_at: row.get(4)?,
                change_note: row.get(5)?,
            })
        }
    ).map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();

    // Create a new version with current content first
    let current: Prompt = db.conn.query_row(
        "SELECT id, title, body, model_target, collection_id, is_pinned, is_archived, 
         use_count, sort_order, created_at, updated_at FROM prompts WHERE id = ?",
        params![version.prompt_id],
        |row| {
            Ok(Prompt {
                id: row.get(0)?,
                title: row.get(1)?,
                body: row.get(2)?,
                model_target: row.get(3)?,
                collection_id: row.get(4)?,
                is_pinned: row.get::<_, i32>(5)? != 0,
                is_archived: row.get::<_, i32>(6)? != 0,
                use_count: row.get(7)?,
                sort_order: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        }
    ).map_err(|e| e.to_string())?;

    db.conn.execute(
        "INSERT INTO prompt_versions (prompt_id, title_snapshot, body_snapshot, changed_at)
         VALUES (?, ?, ?, ?)",
        params![version.prompt_id, current.title, current.body, now],
    ).map_err(|e| e.to_string())?;

    // Restore the selected version
    db.conn.execute(
        "UPDATE prompts SET title = ?, body = ?, updated_at = ? WHERE id = ?",
        params![version.title_snapshot, version.body_snapshot, now, version.prompt_id],
    ).map_err(|e| e.to_string())?;

    info!("Restored prompt version: {} for prompt: {}", version_id, version.prompt_id);
    
    // Return the restored prompt directly
    Ok(Prompt {
        id: version.prompt_id,
        title: version.title_snapshot,
        body: version.body_snapshot,
        model_target: current.model_target,
        collection_id: current.collection_id,
        is_pinned: current.is_pinned,
        is_archived: current.is_archived,
        use_count: current.use_count,
        sort_order: current.sort_order,
        created_at: current.created_at,
        updated_at: now,
    })
}

#[tauri::command]
pub fn duplicate_prompt(state: State<AppState>, id: String) -> Result<Prompt, String> {
    let current = get_prompt(state.clone(), id)?;
    
    create_prompt(state, CreatePromptRequest {
        title: format!("{} (Copy)", current.title),
        body: current.body,
        model_target: current.model_target,
        collection_id: current.collection_id,
    })
}

#[tauri::command]
pub fn toggle_pin_prompt(state: State<AppState>, id: String) -> Result<Prompt, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let current: Prompt = db.conn.query_row(
        "SELECT id, title, body, model_target, collection_id, is_pinned, is_archived, 
         use_count, sort_order, created_at, updated_at FROM prompts WHERE id = ?",
        params![id],
        |row| {
            Ok(Prompt {
                id: row.get(0)?,
                title: row.get(1)?,
                body: row.get(2)?,
                model_target: row.get(3)?,
                collection_id: row.get(4)?,
                is_pinned: row.get::<_, i32>(5)? != 0,
                is_archived: row.get::<_, i32>(6)? != 0,
                use_count: row.get(7)?,
                sort_order: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        }
    ).map_err(|e| e.to_string())?;
    
    let new_pinned = !current.is_pinned;
    let now = chrono::Utc::now().to_rfc3339();

    db.conn.execute(
        "UPDATE prompts SET is_pinned = ?, updated_at = ? WHERE id = ?",
        params![new_pinned as i32, now, id],
    ).map_err(|e| e.to_string())?;

    info!("Toggled pin for prompt: {} to {}", id, new_pinned);
    
    // Return the updated prompt directly
    Ok(Prompt {
        id,
        title: current.title,
        body: current.body,
        model_target: current.model_target,
        collection_id: current.collection_id,
        is_pinned: new_pinned,
        is_archived: current.is_archived,
        use_count: current.use_count,
        sort_order: current.sort_order,
        created_at: current.created_at,
        updated_at: now,
    })
}