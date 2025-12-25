// src/database.rs
// Core database struct with connection management only

use rusqlite::{Connection, Result};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Database {
    pub(crate) app_conn: Arc<Mutex<Connection>>,
    pub(crate) knowledge_conn: Arc<Mutex<Connection>>,
    pub(crate) analytics_conn: Arc<Mutex<Connection>>,
}

// Manual Debug implementation since Mutex<Connection> doesn't implement Debug
impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Database")
            .field("app_conn", &"Arc<Mutex<Connection>>")
            .field("knowledge_conn", &"Arc<Mutex<Connection>>")
            .field("analytics_conn", &"Arc<Mutex<Connection>>")
            .finish()
    }
}

impl Database {
    pub fn new_implied_all_schemas() -> Result<Self> {
        let db_dir = std::path::PathBuf::from("./db");
        if !db_dir.exists() {
            std::fs::create_dir_all(&db_dir).map_err(|e| {
                rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
                    Some(format!("Failed to create db directory: {}", e))
                )
            })?;
        }

        let app_db_path = db_dir.join("survon.db");
        let knowledge_db_path = db_dir.join("knowledge.db");
        let analytics_db_path = db_dir.join("analytics.db");

        Self::new(&app_db_path, &knowledge_db_path, &analytics_db_path)
    }

    pub fn new(app_db_path: &Path, knowledge_db_path: &Path, analytics_db_path: &Path) -> Result<Self> {
        let app_conn = Connection::open(app_db_path)?;
        let knowledge_conn = Connection::open(knowledge_db_path)?;
        let analytics_conn = Connection::open(analytics_db_path)?;

        let db = Database {
            app_conn: Arc::new(Mutex::new(app_conn)),
            knowledge_conn: Arc::new(Mutex::new(knowledge_conn)),
            analytics_conn: Arc::new(Mutex::new(analytics_conn)),
        };

        // Initialize all module schemas
        db.init_all_schemas()?;

        Ok(db)
    }

    fn init_all_schemas(&self) -> Result<()> {
        // Core tables that don't belong to any specific module
        self.init_core_tables()?;

        // Module-specific initialization
        use crate::modules::llm::database::LlmDatabase;
        use crate::modules::overseer::database::OverseerDatabase;
        use crate::modules::side_quest::database::SideQuestDatabase;

        self.init_llm_schema()?;
        self.init_overseer_schema()?;
        self.init_side_quest_schema()?;

        Ok(())
    }

    fn init_core_tables(&self) -> Result<()> {
        let conn = self.app_conn.lock().unwrap();

        // Message bus logging
        conn.execute(
            "CREATE TABLE IF NOT EXISTS message_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                topic TEXT NOT NULL,
                payload TEXT NOT NULL,
                source TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        // Generic module state storage
        conn.execute(
            "CREATE TABLE IF NOT EXISTS module_state (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                module_name TEXT NOT NULL UNIQUE,
                state_data TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    pub fn log_bus_message(&self, topic: &str, payload: &str, source: &str) -> Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let conn = self.app_conn.lock().unwrap();
        conn.execute(
            "INSERT INTO message_log (topic, payload, source, timestamp)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![topic, payload, source, timestamp],
        )?;

        Ok(())
    }

    pub fn save_module_state(&self, module_name: &str, state_data: &str) -> Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let conn = self.app_conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO module_state (module_name, state_data, updated_at)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![module_name, state_data, timestamp],
        )?;

        Ok(())
    }

    pub fn get_module_state(&self, module_name: &str) -> Result<Option<String>> {
        let conn = self.app_conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT state_data FROM module_state WHERE module_name = ?1"
        )?;

        let mut rows = stmt.query_map(rusqlite::params![module_name], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;

        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }
}
