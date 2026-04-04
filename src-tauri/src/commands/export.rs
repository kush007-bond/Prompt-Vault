use crate::AppState;
use crate::commands::types::*;
use tauri::State;
use rusqlite::params;
use std::fs;
use std::path::Path;
use log::info;

#[tauri::command]
pub fn export_to_json(state: State<AppState>) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    // Export collections
    let mut stmt = db.conn.prepare(
        "SELECT id, name, parent_id, color, icon, sort_order FROM collections"
    ).map_err(|e| e.to_string())?;
    
    let collections: Vec<ExportedCollection> = stmt.query_map([], |row| {
        Ok(ExportedCollection {
            id: row.get(0)?,
            name: row.get(1)?,
            parent_id: row.get(2)?,
            color: row.get(3)?,
            icon: row.get(4)?,
            sort_order: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
    
    // Export tags
    let mut stmt = db.conn.prepare("SELECT id, name, color FROM tags").map_err(|e| e.to_string())?;
    let tags: Vec<ExportedTag> = stmt.query_map([], |row| {
        Ok(ExportedTag {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
    
    // Export prompts with tags
    let mut stmt = db.conn.prepare(
        "SELECT id, title, body, model_target, collection_id, is_pinned, is_archived, use_count, created_at, updated_at FROM prompts"
    ).map_err(|e| e.to_string())?;
    
    let mut prompts: Vec<ExportedPrompt> = Vec::new();
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
    
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        // id(0), title(1), body(2), model_target(3), collection_id(4),
        // is_pinned(5), is_archived(6), use_count(7), created_at(8), updated_at(9)
        let id: String = row.get(0).map_err(|e| e.to_string())?;

        // Get tags for this prompt
        let mut tag_stmt = db.conn.prepare(
            "SELECT t.name FROM tags t JOIN prompt_tags pt ON t.id = pt.tag_id WHERE pt.prompt_id = ?"
        ).map_err(|e| e.to_string())?;

        let tag_names: Vec<String> = tag_stmt.query_map(params![id], |r| r.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        prompts.push(ExportedPrompt {
            id,
            title: row.get(1).map_err(|e| e.to_string())?,
            body: row.get(2).map_err(|e| e.to_string())?,
            model_target: row.get(3).map_err(|e| e.to_string())?,
            collection_id: row.get(4).map_err(|e| e.to_string())?,
            is_pinned: row.get::<_, i32>(5).map_err(|e| e.to_string())? != 0,
            is_archived: row.get::<_, i32>(6).map_err(|e| e.to_string())? != 0,
            use_count: row.get(7).map_err(|e| e.to_string())?,
            tags: tag_names,
            created_at: row.get(8).map_err(|e| e.to_string())?,
            updated_at: row.get(9).map_err(|e| e.to_string())?,
        });
    }
    
    let export_data = ExportData {
        version: "1.0".to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        prompts,
        collections,
        tags,
    };
    
    serde_json::to_string_pretty(&export_data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_to_markdown(state: State<AppState>, output_path: String) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    let output = Path::new(&output_path);
    fs::create_dir_all(output).map_err(|e| e.to_string())?;
    
    // Get all prompts
    let mut stmt = db.conn.prepare(
        "SELECT title, body FROM prompts WHERE is_archived = 0"
    ).map_err(|e| e.to_string())?;

    let mut prompts_written = 0;
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;

    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let title: String = row.get(0).map_err(|e| e.to_string())?;
        let body: String = row.get(1).map_err(|e| e.to_string())?;
        
        // Sanitize filename
        let safe_title = title.chars()
            .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' })
            .collect::<String>();
        
        let file_path = output.join(format!("{}.md", safe_title));
        
        let content = format!("# {}\n\n{}", title, body);
        fs::write(&file_path, content).map_err(|e| e.to_string())?;
        
        prompts_written += 1;
    }
    
    info!("Exported {} prompts to markdown", prompts_written);
    
    Ok(format!("Exported {} prompts to {}", prompts_written, output_path))
}

#[tauri::command]
pub fn import_from_json(state: State<AppState>, request: ImportRequest) -> Result<ImportResult, String> {
    let export_data: ExportData = serde_json::from_str(&request.json_content)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    let mut prompts_imported = 0;
    let mut collections_imported = 0;
    let mut tags_imported = 0;
    
    // Import collections first
    for collection in export_data.collections {
        db.conn.execute(
            "INSERT OR IGNORE INTO collections (id, name, parent_id, color, icon, sort_order)
             VALUES (?, ?, ?, ?, ?, ?)",
            params![collection.id, collection.name, collection.parent_id, collection.color, collection.icon, collection.sort_order],
        ).map_err(|e| e.to_string())?;
        collections_imported += 1;
    }
    
    // Import tags
    for tag in export_data.tags {
        db.conn.execute(
            "INSERT OR IGNORE INTO tags (id, name, color) VALUES (?, ?, ?)",
            params![tag.id, tag.name, tag.color],
        ).map_err(|e| e.to_string())?;
        
        tags_imported += 1;
    }
    
    // Import prompts
    for prompt in export_data.prompts {
        let now = chrono::Utc::now().to_rfc3339();
        
        db.conn.execute(
            "INSERT OR REPLACE INTO prompts (id, title, body, model_target, collection_id, is_pinned, is_archived, use_count, sort_order, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                prompt.id, prompt.title, prompt.body, prompt.model_target, prompt.collection_id,
                prompt.is_pinned as i32, prompt.is_archived as i32, prompt.use_count, 0.0,
                prompt.created_at, now
            ],
        ).map_err(|e| e.to_string())?;
        
        // Add tags
        for tag_name in prompt.tags {
            // Get or create tag
            let tag_id: Result<String, _> = db.conn.query_row(
                "SELECT id FROM tags WHERE name = ?",
                params![tag_name],
                |row| row.get(0),
            );
            
            if let Ok(tid) = tag_id {
                db.conn.execute(
                    "INSERT OR IGNORE INTO prompt_tags (prompt_id, tag_id) VALUES (?, ?)",
                    params![prompt.id, tid],
                ).map_err(|e| e.to_string())?;
            }
        }
        
        prompts_imported += 1;
    }
    
    info!("Imported {} prompts, {} collections, {} tags", prompts_imported, collections_imported, tags_imported);
    
    Ok(ImportResult {
        success: true,
        prompts_imported,
        collections_imported,
        tags_imported,
        message: format!("Successfully imported {} prompts, {} collections, and {} tags", prompts_imported, collections_imported, tags_imported),
    })
}