use crate::AppState;
use crate::commands::types::*;
use tauri::State;
use rusqlite::params;
use std::fs;
use std::path::Path;
use log::info;
use uuid::Uuid;
use serde_json::json;

const PROMPTVAULT_START: &str = "<!-- promptvault:start -->";
const PROMPTVAULT_END: &str = "<!-- promptvault:end -->";

fn format_claude_code_content(content: &str) -> String {
    format!(
        "{}\n\n{}\n{}",
        PROMPTVAULT_START, content, PROMPTVAULT_END
    )
}

fn backup_path_for(path: &Path) -> std::path::PathBuf {
    // Append .promptvault.bak to the full filename rather than replacing the extension
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("backup");
    path.with_file_name(format!("{}.promptvault.bak", filename))
}

fn inject_with_delimiter(path: &Path, content: &str) -> Result<String, String> {
    // Create backup
    if path.exists() {
        let backup_path = backup_path_for(path);
        fs::copy(path, &backup_path).map_err(|e| e.to_string())?;
    }
    
    let existing_content = if path.exists() {
        fs::read_to_string(path).unwrap_or_default()
    } else {
        String::new()
    };
    
    // Check if existing content has promptvault section
    let new_content = if existing_content.contains(PROMPTVAULT_START) && existing_content.contains(PROMPTVAULT_END) {
        // Replace existing section
        let start_idx = existing_content.find(PROMPTVAULT_START).unwrap();
        let end_idx = existing_content.find(PROMPTVAULT_END).unwrap() + PROMPTVAULT_END.len();
        
        let mut result = existing_content[..start_idx].to_string();
        result.push_str(&format_claude_code_content(content));
        result.push_str(&existing_content[end_idx..]);
        result
    } else {
        // Prepend new section
        format!("{}\n\n{}", format_claude_code_content(content), existing_content)
    };
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    
    fs::write(path, &new_content).map_err(|e| e.to_string())?;
    
    Ok(new_content)
}

#[tauri::command]
pub fn inject_to_claude_code(state: State<AppState>, request: InjectRequest) -> Result<InjectionResult, String> {
    let project_path = Path::new(&request.project_path);
    let apply_globally = request.apply_globally.unwrap_or(false);
    
    let target_path = if apply_globally {
        // Global: ~/.claude/CLAUDE.md
        let home = dirs::home_dir().ok_or("Cannot find home directory")?;
        home.join(".claude").join("CLAUDE.md")
    } else {
        // Project-level
        project_path.join("CLAUDE.md")
    };
    
    inject_with_delimiter(&target_path, &request.content)?;
    
    // Log injection
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    
    db.conn.execute(
        "INSERT INTO injection_log (id, prompt_id, cli_target, project_path, injected_content, injected_at)
         VALUES (?, ?, ?, ?, ?, ?)",
        params![id, request.prompt_id, "claude_code", request.project_path, request.content, now],
    ).map_err(|e| e.to_string())?;
    
    info!("Injected prompt to Claude Code: {:?}", target_path);
    
    Ok(InjectionResult {
        success: true,
        target_path: target_path.to_string_lossy().to_string(),
        message: format!("Successfully injected to {}", target_path.display()),
    })
}

#[tauri::command]
pub fn inject_to_cursor(state: State<AppState>, request: InjectRequest) -> Result<InjectionResult, String> {
    let project_path = Path::new(&request.project_path);
    let target_path = project_path.join(".cursorrules");
    
    inject_with_delimiter(&target_path, &request.content)?;
    
    // Log injection
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    
    db.conn.execute(
        "INSERT INTO injection_log (id, prompt_id, cli_target, project_path, injected_content, injected_at)
         VALUES (?, ?, ?, ?, ?, ?)",
        params![id, request.prompt_id, "cursor", request.project_path, request.content, now],
    ).map_err(|e| e.to_string())?;
    
    info!("Injected prompt to Cursor: {:?}", target_path);
    
    Ok(InjectionResult {
        success: true,
        target_path: target_path.to_string_lossy().to_string(),
        message: format!("Successfully injected to {}", target_path.display()),
    })
}

#[tauri::command]
pub fn inject_to_continue(state: State<AppState>, request: InjectRequest) -> Result<InjectionResult, String> {
    let config_path = dirs::home_dir()
        .ok_or("Cannot find home directory")?
        .join(".continue")
        .join("config.json");
    
    // Read existing config or create new
    let config_content = if config_path.exists() {
        fs::read_to_string(&config_path).unwrap_or_default()
    } else {
        r#"{"models": []}"#.to_string()
    };
    
    // Parse and update config
    let mut config: serde_json::Value = serde_json::from_str(&config_content).unwrap_or(json!({"models": []}));
    
    if let Some(models) = config["models"].as_array_mut() {
        // Find or create a model config with the prompt
        let model_obj = models.iter_mut().find(|m| m["model"].as_str() == Some("Claude"));
        
        if let Some(m) = model_obj {
            m["system_message"] = serde_json::Value::String(request.content.clone());
        } else {
            models.push(json!({
                "model": "Claude",
                "system_message": request.content
            }));
        }
    }
    
    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    
    // Create backup
    if config_path.exists() {
        let backup_path = backup_path_for(&config_path);
        let _ = fs::copy(&config_path, &backup_path);
    }
    
    fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap())
        .map_err(|e| e.to_string())?;
    
    // Log injection
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    
    db.conn.execute(
        "INSERT INTO injection_log (id, prompt_id, cli_target, project_path, injected_content, injected_at)
         VALUES (?, ?, ?, ?, ?, ?)",
        params![id, request.prompt_id, "continue", request.project_path, request.content, now],
    ).map_err(|e| e.to_string())?;
    
    info!("Injected prompt to Continue.dev: {:?}", config_path);
    
    Ok(InjectionResult {
        success: true,
        target_path: config_path.to_string_lossy().to_string(),
        message: format!("Successfully injected to {}", config_path.display()),
    })
}

#[tauri::command]
pub fn inject_to_aider(state: State<AppState>, request: InjectRequest) -> Result<InjectionResult, String> {
    let project_path = Path::new(&request.project_path);
    let target_path = project_path.join(".aider.conf.yml");
    
    // Read existing config or create new
    let config_content = if target_path.exists() {
        fs::read_to_string(&target_path).unwrap_or_default()
    } else {
        String::new()
    };
    
    // Simple YAML update - append or update system prompt
    let new_content = if config_content.contains("system-prompt:") {
        // Replace existing
        let lines: Vec<&str> = config_content.lines().collect();
        let mut result = String::new();
        let mut in_system_prompt = false;
        
        for line in lines {
            if line.trim().starts_with("system-prompt:") {
                in_system_prompt = true;
                result.push_str(&format!("system-prompt: |\n  {}\n", request.content.replace('\n', "\n  ")));
            } else if in_system_prompt && (line.starts_with(' ') || line.starts_with('\t')) {
                // Skip indented continuation lines of the old system-prompt block scalar.
                // Any non-indented line signals the start of a new top-level key.
                continue;
            } else {
                in_system_prompt = false;
                result.push_str(line);
                result.push('\n');
            }
        }
        result
    } else {
        // Append
        format!("{}\nsystem-prompt: |\n  {}", config_content, request.content.replace("\n", "\n  "))
    };
    
    // Ensure parent directory exists
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    
    // Create backup
    if target_path.exists() {
        let backup_path = backup_path_for(&target_path);
        let _ = fs::copy(&target_path, &backup_path);
    }
    
    fs::write(&target_path, new_content).map_err(|e| e.to_string())?;
    
    // Log injection
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    
    db.conn.execute(
        "INSERT INTO injection_log (id, prompt_id, cli_target, project_path, injected_content, injected_at)
         VALUES (?, ?, ?, ?, ?, ?)",
        params![id, request.prompt_id, "aider", request.project_path, request.content, now],
    ).map_err(|e| e.to_string())?;
    
    info!("Injected prompt to Aider: {:?}", target_path);
    
    Ok(InjectionResult {
        success: true,
        target_path: target_path.to_string_lossy().to_string(),
        message: format!("Successfully injected to {}", target_path.display()),
    })
}

#[tauri::command]
pub fn preview_injection(target: String, project_path: String, content: String, apply_globally: Option<bool>) -> Result<InjectionPreview, String> {
    let path = Path::new(&project_path);
    
    let target_path = match target.as_str() {
        "claude_code" => {
            if apply_globally.unwrap_or(false) {
                dirs::home_dir()
                    .ok_or("Cannot find home directory")?
                    .join(".claude")
                    .join("CLAUDE.md")
            } else {
                path.join("CLAUDE.md")
            }
        }
        "cursor" => path.join(".cursorrules"),
        "continue" => dirs::home_dir()
            .ok_or("Cannot find home directory")?
            .join(".continue")
            .join("config.json"),
        "aider" => path.join(".aider.conf.yml"),
        _ => return Err(format!("Unknown target: {}", target)),
    };
    
    let will_overwrite = target_path.exists();
    let content_preview = format_claude_code_content(&content);
    
    Ok(InjectionPreview {
        target_path: target_path.to_string_lossy().to_string(),
        content_preview,
        will_overwrite,
    })
}

#[tauri::command]
pub fn get_injection_history(state: State<AppState>, limit: Option<i32>) -> Result<Vec<InjectionLog>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let limit = limit.unwrap_or(50);
    
    let mut stmt = db.conn.prepare(
        "SELECT id, prompt_id, cli_target, project_path, injected_content, injected_at 
         FROM injection_log ORDER BY injected_at DESC LIMIT ?"
    ).map_err(|e| e.to_string())?;

    let logs = stmt.query_map(params![limit], |row| {
        Ok(InjectionLog {
            id: row.get(0)?,
            prompt_id: row.get(1)?,
            cli_target: row.get(2)?,
            project_path: row.get(3)?,
            injected_content: row.get(4)?,
            injected_at: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;

    logs.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

// Helper module for dirs
mod dirs {
    use std::path::PathBuf;
    
    pub fn home_dir() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            std::env::var("USERPROFILE").ok().map(PathBuf::from)
        }
        #[cfg(not(target_os = "windows"))]
        {
            std::env::var("HOME").ok().map(PathBuf::from)
        }
    }
}