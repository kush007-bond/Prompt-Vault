use rusqlite::{Connection, Result, params};
use std::path::PathBuf;
use log::{info, error};
use once_cell::sync::Lazy;
use std::fs;

pub static DB_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let app_data = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."));
    let app_dir = app_data.join("com.promptvault.app");
    
    if let Err(e) = fs::create_dir_all(&app_dir) {
        error!("Failed to create app directory: {} - {}", app_dir.display(), e);
    }
    
    let db_path = app_dir.join("promptvault.db");
    info!("Database path: {:?}", db_path);
    db_path
});

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        info!("Opening database at {:?}", DB_PATH.as_path());
        
        let conn = Connection::open(DB_PATH.as_path())
            .map_err(|e| {
                error!("Failed to open database: {}", e);
                e
            })?;
        
        info!("Database opened successfully");
        
        // Configure SQLite settings
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;
             PRAGMA synchronous = NORMAL;
             PRAGMA cache_size = -32000;
             PRAGMA temp_store = MEMORY;
             PRAGMA mmap_size = 268435456;"
        ).map_err(|e| {
            error!("Failed to configure SQLite PRAGMAs: {}", e);
            e
        })?;

        info!("SQLite PRAGMAs configured");

        // Run migrations
        Self::run_migrations(&conn)?;

        info!("Database initialized successfully");
        
        Ok(Database { conn })
    }

    fn run_migrations(conn: &Connection) -> Result<()> {
        // Create migrations table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        )?;

        // Get current version
        let current_version: i64 = conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )?;

        info!("Current database schema version: {}", current_version);

        // Apply migrations (additive only)
        if current_version < 1 {
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS collections (
                    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
                    name TEXT NOT NULL,
                    parent_id TEXT REFERENCES collections(id) ON DELETE CASCADE,
                    color TEXT,
                    icon TEXT,
                    is_smart INTEGER NOT NULL DEFAULT 0,
                    smart_filter TEXT,
                    sort_order REAL NOT NULL DEFAULT 0,
                    created_at TEXT NOT NULL DEFAULT (datetime('now'))
                );

                CREATE TABLE IF NOT EXISTS tags (
                    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
                    name TEXT NOT NULL UNIQUE COLLATE NOCASE,
                    color TEXT
                );

                CREATE TABLE IF NOT EXISTS prompts (
                    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
                    title TEXT NOT NULL,
                    body TEXT NOT NULL,
                    model_target TEXT,
                    collection_id TEXT REFERENCES collections(id) ON DELETE SET NULL,
                    is_pinned INTEGER NOT NULL DEFAULT 0,
                    is_archived INTEGER NOT NULL DEFAULT 0,
                    use_count INTEGER NOT NULL DEFAULT 0,
                    sort_order REAL NOT NULL DEFAULT 0,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
                );

                CREATE TABLE IF NOT EXISTS prompt_tags (
                    prompt_id TEXT NOT NULL REFERENCES prompts(id) ON DELETE CASCADE,
                    tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
                    PRIMARY KEY (prompt_id, tag_id)
                );

                CREATE TABLE IF NOT EXISTS prompt_versions (
                    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
                    prompt_id TEXT NOT NULL REFERENCES prompts(id) ON DELETE CASCADE,
                    title_snapshot TEXT NOT NULL,
                    body_snapshot TEXT NOT NULL,
                    changed_at TEXT NOT NULL DEFAULT (datetime('now')),
                    change_note TEXT
                );

                CREATE TABLE IF NOT EXISTS variables (
                    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
                    prompt_id TEXT NOT NULL REFERENCES prompts(id) ON DELETE CASCADE,
                    name TEXT NOT NULL,
                    display_label TEXT,
                    type TEXT NOT NULL DEFAULT 'text',
                    default_value TEXT,
                    options TEXT,
                    required INTEGER NOT NULL DEFAULT 1,
                    sort_order INTEGER NOT NULL DEFAULT 0
                );

                CREATE TABLE IF NOT EXISTS response_history (
                    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
                    prompt_id TEXT NOT NULL REFERENCES prompts(id) ON DELETE CASCADE,
                    provider TEXT NOT NULL,
                    model TEXT NOT NULL,
                    input_snapshot TEXT NOT NULL,
                    response TEXT NOT NULL,
                    tokens_input INTEGER,
                    tokens_output INTEGER,
                    duration_ms INTEGER,
                    ran_at TEXT NOT NULL DEFAULT (datetime('now'))
                );

                CREATE TABLE IF NOT EXISTS injection_log (
                    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
                    prompt_id TEXT REFERENCES prompts(id) ON DELETE SET NULL,
                    cli_target TEXT NOT NULL,
                    project_path TEXT NOT NULL,
                    injected_content TEXT NOT NULL,
                    injected_at TEXT NOT NULL DEFAULT (datetime('now'))
                );

                CREATE TABLE IF NOT EXISTS settings (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                );"
            )?;

            conn.execute(
                "INSERT INTO schema_migrations (version) VALUES (1)",
                [],
            )?;
            info!("Applied migration 1: initial schema");
        }

        if current_version < 2 {
            conn.execute(
                "CREATE VIRTUAL TABLE IF NOT EXISTS prompts_fts USING fts5(
                    title, body,
                    content='prompts',
                    content_rowid='rowid',
                    tokenize='unicode61 remove_diacritics 2'
                )",
                [],
            )?;

            // Create triggers to keep FTS in sync
            conn.execute_batch(
                "CREATE TRIGGER IF NOT EXISTS prompts_ai AFTER INSERT ON prompts BEGIN
                    INSERT INTO prompts_fts(rowid, title, body) VALUES (new.rowid, new.title, new.body);
                END;

                CREATE TRIGGER IF NOT EXISTS prompts_ad AFTER DELETE ON prompts BEGIN
                    INSERT INTO prompts_fts(prompts_fts, rowid, title, body) VALUES('delete', old.rowid, old.title, old.body);
                END;

                CREATE TRIGGER IF NOT EXISTS prompts_au AFTER UPDATE ON prompts BEGIN
                    INSERT INTO prompts_fts(prompts_fts, rowid, title, body) VALUES('delete', old.rowid, old.title, old.body);
                    INSERT INTO prompts_fts(rowid, title, body) VALUES (new.rowid, new.title, new.body);
                END;"
            )?;

            conn.execute(
                "INSERT INTO schema_migrations (version) VALUES (2)",
                [],
            )?;
            info!("Applied migration 2: FTS5 search index");
        }

        // Insert default settings if not exist
        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES ('theme', 'system')",
            [],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES ('editor_font_size', '14')",
            [],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES ('default_provider', 'openai')",
            [],
        )?;

        Ok(())
    }
}

// Helper for dirs crate
mod dirs {
    use std::path::PathBuf;

    pub fn data_local_dir() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            std::env::var("LOCALAPPDATA").ok().map(PathBuf::from)
        }
        #[cfg(target_os = "macos")]
        {
            std::env::var("HOME").ok().map(|h| PathBuf::from(h).join("Library/Application Support"))
        }
        #[cfg(target_os = "linux")]
        {
            std::env::var("XDG_DATA_HOME")
                .ok()
                .map(PathBuf::from)
                .or_else(|| std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".local/share")))
        }
    }
}