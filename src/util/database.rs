// src/database.rs
// Complete updated version with Arc<Mutex<>> wrapper for thread safety

use rusqlite::{Connection, params, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Option<i64>,
    pub session_id: String,
    pub role: String, // "user" or "assistant"
    pub content: String,
    pub timestamp: i64,
    pub module_name: String,
}

impl ChatMessage {
    pub fn new_user(session_id: String, content: String, module_name: String) -> Self {
        Self {
            id: None,
            session_id,
            role: "user".to_string(),
            content,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            module_name,
        }
    }

    pub fn new_assistant(session_id: String, content: String, module_name: String) -> Self {
        Self {
            id: None,
            session_id,
            role: "assistant".to_string(),
            content,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            module_name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeChunk {
    pub id: Option<i64>,
    pub source_file: String,
    pub domain: String,
    pub category: String,
    pub title: String,
    pub body: String,
    pub chunk_index: i32,
    pub metadata: String, // JSON string
}

/// Device record from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownDevice {
    pub mac_address: String,
    pub device_name: String,
    pub device_type: Option<String>,
    pub firmware_version: Option<String>,
    pub first_seen: i64,
    pub last_seen: i64,
    pub is_trusted: bool,
    pub rssi: Option<i16>,
}

#[derive(Clone)]
pub struct Database {
    app_conn: Arc<Mutex<Connection>>,
    knowledge_conn: Arc<Mutex<Connection>>,
    analytics_conn: Arc<Mutex<Connection>>,
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
    pub fn clear_knowledge(&self) -> Result<()> {
        let conn = self.knowledge_conn.lock().unwrap();
        conn.execute("DELETE FROM knowledge", [])?;
        Ok(())
    }

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
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        // App database tables
        {
            let conn = self.app_conn.lock().unwrap();
            conn.execute(
                "CREATE TABLE IF NOT EXISTS chat_messages (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT NOT NULL,
                    role TEXT NOT NULL,
                    content TEXT NOT NULL,
                    timestamp INTEGER NOT NULL,
                    module_name TEXT NOT NULL
                )",
                [],
            )?;

            conn.execute(
                "CREATE TABLE IF NOT EXISTS module_state (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    module_name TEXT NOT NULL UNIQUE,
                    state_data TEXT NOT NULL,
                    updated_at INTEGER NOT NULL
                )",
                [],
            )?;

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

            // NEW TABLE: known_devices (replaces trusted_devices)
            conn.execute(
                "CREATE TABLE IF NOT EXISTS known_devices (
                    mac_address TEXT PRIMARY KEY,
                    device_name TEXT NOT NULL,
                    device_type TEXT,
                    firmware_version TEXT,
                    first_seen INTEGER NOT NULL,
                    last_seen INTEGER NOT NULL,
                    is_trusted INTEGER NOT NULL DEFAULT 0,
                    rssi INTEGER
                )",
                [],
            )?;

            // Create index for trust queries
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_trusted
                 ON known_devices(is_trusted)",
                [],
            )?;

            // MIGRATION: Copy data from old trusted_devices table if it exists
            // Check if old table exists
            let old_table_exists: i64 = conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='trusted_devices'",
                [],
                |row| row.get(0)
            ).unwrap_or(0);

            if old_table_exists > 0 {
                println!("Migrating data from trusted_devices to known_devices...");

                // Copy trusted devices to new table
                conn.execute(
                    "INSERT OR IGNORE INTO known_devices (mac_address, device_name, first_seen, last_seen, is_trusted)
                     SELECT mac_address, device_name, trusted_at, trusted_at, 1
                     FROM trusted_devices",
                    [],
                )?;

                // Drop old table
                conn.execute("DROP TABLE IF EXISTS trusted_devices", [])?;

                println!("Migration complete!");
            }
        }

        // Knowledge database - FTS5 virtual table
        {
            let conn = self.knowledge_conn.lock().unwrap();
            conn.execute(
                "CREATE VIRTUAL TABLE IF NOT EXISTS knowledge USING fts5(
                    source_file,
                    domain,
                    category,
                    title,
                    body,
                    chunk_index UNINDEXED,
                    metadata UNINDEXED
                )",
                [],
            )?;
        }

        // Analytics database tables
        {
            let conn = self.analytics_conn.lock().unwrap();
            conn.execute(
                "CREATE TABLE IF NOT EXISTS query_stats (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    query_text TEXT NOT NULL,
                    results_found INTEGER NOT NULL,
                    response_time_ms INTEGER NOT NULL,
                    timestamp INTEGER NOT NULL
                )",
                [],
            )?;

            conn.execute(
                "CREATE TABLE IF NOT EXISTS usage_patterns (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT NOT NULL,
                    interaction_type TEXT NOT NULL,
                    duration_ms INTEGER NOT NULL,
                    timestamp INTEGER NOT NULL
                )",
                [],
            )?;
        }

        Ok(())
    }

    // App database methods
    pub fn insert_chat_message(&self, message: ChatMessage) -> Result<i64> {
        let conn = self.app_conn.lock().unwrap();
        conn.execute(
            "INSERT INTO chat_messages (session_id, role, content, timestamp, module_name)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                message.session_id,
                message.role,
                message.content,
                message.timestamp,
                message.module_name
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_chat_history(&self, session_id: &str, limit: usize) -> Result<Vec<ChatMessage>> {
        let conn = self.app_conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, timestamp, module_name
             FROM chat_messages
             WHERE session_id = ?1
             ORDER BY timestamp ASC
             LIMIT ?2"
        )?;

        let rows = stmt.query_map(params![session_id, limit], |row| {
            Ok(ChatMessage {
                id: Some(row.get(0)?),
                session_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                timestamp: row.get(4)?,
                module_name: row.get(5)?,
            })
        })?;

        let mut messages = Vec::new();
        for row in rows {
            messages.push(row?);
        }

        Ok(messages)
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
            params![topic, payload, source, timestamp],
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
            params![module_name, state_data, timestamp],
        )?;

        Ok(())
    }

    pub fn get_module_state(&self, module_name: &str) -> Result<Option<String>> {
        let conn = self.app_conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT state_data FROM module_state WHERE module_name = ?1"
        )?;

        let mut rows = stmt.query_map(params![module_name], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;

        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    // Knowledge database methods
    pub fn insert_knowledge_chunk(&self, chunk: KnowledgeChunk) -> Result<()> {
        let conn = self.knowledge_conn.lock().unwrap();
        conn.execute(
            "INSERT INTO knowledge (source_file, domain, category, title, body, chunk_index, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                chunk.source_file,
                chunk.domain,
                chunk.category,
                chunk.title,
                chunk.body,
                chunk.chunk_index,
                chunk.metadata
            ],
        )?;

        Ok(())
    }

    fn sanitize_fts5_query(query: &str) -> String {
        // More permissive sanitization - keep common search terms
        query
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn search_knowledge(&self, query: &str, domains: &[String], limit: usize) -> Result<Vec<KnowledgeChunk>> {
        let clean_query = Self::sanitize_fts5_query(query);
        if clean_query.trim().is_empty() {
            return Ok(Vec::new());
        }

        println!("Searching knowledge with query: '{}' (sanitized from '{}')", clean_query, query);
        if !domains.is_empty() {
            println!("Filtering by domains: {:?}", domains);
        }

        // Try different search strategies
        let mut results = Vec::new();

        // Strategy 1: Try exact phrase match with AND
        results = self.execute_search(&clean_query, domains, limit * 2)?;
        println!("Strategy 1 (AND search): found {} results", results.len());

        // Strategy 2: If no results, try OR search
        if results.is_empty() && clean_query.contains(' ') {
            let or_query = clean_query.split_whitespace().collect::<Vec<_>>().join(" OR ");
            println!("Strategy 2 (OR search): trying '{}'", or_query);
            results = self.execute_search(&or_query, domains, limit * 2)?;
            println!("Strategy 2 (OR search): found {} results", results.len());
        }

        // Strategy 3: If still no results, try each word individually
        if results.is_empty() {
            let words: Vec<&str> = clean_query.split_whitespace().collect();
            println!("Strategy 3 (individual words): trying {} words", words.len());
            for word in &words {
                let word_results = self.execute_search(word, domains, limit)?;
                println!("  Word '{}': found {} results", word, word_results.len());
                if !word_results.is_empty() {
                    results.extend(word_results);
                    break; // Use first successful word
                }
            }
        }

        // Filter results by relevance for OR queries
        if clean_query.contains(" OR ") {
            let keywords: Vec<&str> = clean_query.split(" OR ").collect();
            results = results.into_iter()
                .filter(|chunk| {
                    let content_lower = format!("{} {}", chunk.title, chunk.body).to_lowercase();
                    let matches = keywords.iter().filter(|&&keyword| content_lower.contains(keyword)).count();
                    matches >= 2 || keywords.len() == 1
                })
                .take(limit)
                .collect();
        }

        println!("Final results: {} chunks", results.len());
        Ok(results.into_iter().take(limit).collect())
    }

    fn execute_search(&self, search_query: &str, domains: &[String], limit: usize) -> Result<Vec<KnowledgeChunk>> {
        let conn = self.knowledge_conn.lock().unwrap();

        let sql = if domains.is_empty() {
            "SELECT rowid, source_file, domain, category, title, body, chunk_index, metadata
         FROM knowledge WHERE knowledge MATCH ?1 ORDER BY rank LIMIT ?2".to_string()
        } else {
            let domain_placeholders = domains.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            format!(
                "SELECT rowid, source_file, domain, category, title, body, chunk_index, metadata
             FROM knowledge WHERE knowledge MATCH ?1 AND domain IN ({}) ORDER BY rank LIMIT ?{}",
                domain_placeholders,
                domains.len() + 2
            )
        };

        let mut stmt = conn.prepare(&sql)?;

        let limit_str = limit.to_string();
        let mut params_vec = vec![search_query];
        params_vec.extend(domains.iter().map(|d| d.as_str()));
        params_vec.push(&limit_str);

        let rows = stmt.query_map(rusqlite::params_from_iter(params_vec), |row| {
            Ok(KnowledgeChunk {
                id: Some(row.get(0)?),
                source_file: row.get(1)?,
                domain: row.get(2)?,
                category: row.get(3)?,
                title: row.get(4)?,
                body: row.get(5)?,
                chunk_index: row.get(6)?,
                metadata: row.get(7)?,
            })
        })?;

        let mut chunks = Vec::new();
        for row in rows {
            chunks.push(row?);
        }

        Ok(chunks)
    }

    // ========================================================================
    // DEVICE TRUST MANAGEMENT (NEW)
    // ========================================================================

    /// Record a discovered device (creates if new, updates if existing)
    /// Returns true if this is a NEW device
    pub fn record_device_discovery(
        &self,
        mac_address: &str,
        device_name: &str,
        rssi: i16,
    ) -> Result<bool> {
        let conn = self.app_conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Check if device already exists
        let exists: bool = conn.query_row(
            "SELECT COUNT(*) FROM known_devices WHERE mac_address = ?1",
            params![mac_address],
            |row| row.get::<_, i64>(0).map(|count| count > 0),
        )?;

        if exists {
            // Update last seen, RSSI, and device name
            conn.execute(
                "UPDATE known_devices
                 SET last_seen = ?1, rssi = ?2, device_name = ?3
                 WHERE mac_address = ?4",
                params![now, rssi, device_name, mac_address],
            )?;
            Ok(false) // Not a new device
        } else {
            // Insert new device (untrusted by default)
            conn.execute(
                "INSERT INTO known_devices
                 (mac_address, device_name, first_seen, last_seen, is_trusted, rssi)
                 VALUES (?1, ?2, ?3, ?4, 0, ?5)",
                params![mac_address, device_name, now, now, rssi],
            )?;
            Ok(true) // New device discovered
        }
    }

    /// Update device metadata after successful registration
    pub fn update_device_metadata(
        &self,
        mac_address: &str,
        device_type: &str,
        firmware_version: &str,
    ) -> Result<()> {
        let conn = self.app_conn.lock().unwrap();
        conn.execute(
            "UPDATE known_devices
             SET device_type = ?1, firmware_version = ?2
             WHERE mac_address = ?3",
            params![device_type, firmware_version, mac_address],
        )?;
        Ok(())
    }

    /// Check if a device is trusted
    pub fn is_device_trusted(&self, mac_address: &str) -> Result<bool> {
        let conn = self.app_conn.lock().unwrap();

        let trusted: Result<i64, _> = conn.query_row(
            "SELECT is_trusted FROM known_devices WHERE mac_address = ?1",
            params![mac_address],
            |row| row.get(0),
        );

        Ok(trusted.unwrap_or(0) == 1)
    }

    /// Set device trust status (used by UI module)
    pub fn set_device_trust(&self, mac_address: &str, trusted: bool) -> Result<()> {
        let conn = self.app_conn.lock().unwrap();
        let rows_affected = conn.execute(
            "UPDATE known_devices SET is_trusted = ?1 WHERE mac_address = ?2",
            params![if trusted { 1 } else { 0 }, mac_address],
        )?;

        if rows_affected == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        Ok(())
    }

    /// Trust a device (legacy compatibility - now uses record + set_trust)
    pub fn trust_device(&self, mac_address: &str, device_name: &str) -> Result<()> {
        // Ensure device exists in database
        self.record_device_discovery(mac_address, device_name, 0)?;
        // Then trust it
        self.set_device_trust(mac_address, true)
    }

    /// Untrust a device
    pub fn untrust_device(&self, mac_address: &str) -> Result<()> {
        self.set_device_trust(mac_address, false)
    }

    /// Get all trusted devices (for backward compatibility)
    pub fn get_trusted_devices(&self) -> Result<Vec<(String, String)>> {
        let conn = self.app_conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT mac_address, device_name FROM known_devices WHERE is_trusted = 1"
        )?;

        let devices = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(devices)
    }

    /// Get all known devices with full metadata
    pub fn get_all_known_devices(&self) -> Result<Vec<KnownDevice>> {
        let conn = self.app_conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT mac_address, device_name, device_type, firmware_version,
                    first_seen, last_seen, is_trusted, rssi
             FROM known_devices
             ORDER BY last_seen DESC"
        )?;

        let devices = stmt
            .query_map([], |row| {
                Ok(KnownDevice {
                    mac_address: row.get(0)?,
                    device_name: row.get(1)?,
                    device_type: row.get(2)?,
                    firmware_version: row.get(3)?,
                    first_seen: row.get(4)?,
                    last_seen: row.get(5)?,
                    is_trusted: row.get::<_, i64>(6)? == 1,
                    rssi: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(devices)
    }

    /// Delete a device from known devices
    pub fn delete_device(&self, mac_address: &str) -> Result<()> {
        let conn = self.app_conn.lock().unwrap();
        conn.execute(
            "DELETE FROM known_devices WHERE mac_address = ?1",
            params![mac_address],
        )?;
        Ok(())
    }
}
