// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod commands;
mod keychain;
mod adapters;

use std::sync::Mutex;
use log::{info, error, LevelFilter};
use env_logger::Builder;
use std::io::Write;

pub struct AppState {
    pub db: Mutex<db::Database>,
}

fn main() {
    // Initialize logging - include debug level for detailed database logs
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] {} - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Debug)  // Changed to Debug for more detailed logs
        .init();

    info!("Starting PromptVault application");
    info!("Log level set to DEBUG");

    // Initialize database
    let database = match db::Database::new() {
        Ok(db) => {
            info!("Database initialized successfully");
            db
        }
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            panic!("Database initialization failed: {}", e);
        }
    };

    info!("Creating app state...");
    let app_state = AppState {
        db: Mutex::new(database),
    };
    info!("App state created");

    info!("Building Tauri application...");
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::prompts::get_all_prompts,
            commands::prompts::get_prompt,
            commands::prompts::create_prompt,
            commands::prompts::update_prompt,
            commands::prompts::delete_prompt,
            commands::prompts::search_prompts,
            commands::prompts::get_prompt_versions,
            commands::prompts::restore_prompt_version,
            commands::prompts::duplicate_prompt,
            commands::prompts::toggle_pin_prompt,
            commands::collections::get_all_collections,
            commands::collections::create_collection,
            commands::collections::update_collection,
            commands::collections::delete_collection,
            commands::tags::get_all_tags,
            commands::tags::create_tag,
            commands::tags::update_tag,
            commands::tags::delete_tag,
            commands::tags::add_tag_to_prompt,
            commands::tags::remove_tag_from_prompt,
            commands::tags::get_prompt_tags,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::get_all_settings,
            commands::ai::run_prompt,
            commands::ai::run_prompt_streaming,
            commands::ai::run_conversation,
            commands::ai::stream_prompt,
            commands::ai::list_models,
            commands::ai::health_check,
            commands::ai::store_api_key,
            commands::ai::get_api_key_status,
            commands::cli::inject_to_claude_code,
            commands::cli::inject_to_cursor,
            commands::cli::inject_to_continue,
            commands::cli::inject_to_aider,
            commands::cli::preview_injection,
            commands::cli::get_injection_history,
            commands::export::export_to_json,
            commands::export::export_to_markdown,
            commands::export::import_from_json,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
