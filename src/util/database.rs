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

            conn.execute(
                "CREATE TABLE IF NOT EXISTS trusted_devices (
                    mac_address TEXT PRIMARY KEY,
                    device_name TEXT NOT NULL,
                    trusted_at INTEGER NOT NULL,
                    trusted_by TEXT DEFAULT 'user'
                )",
                [],
            )?;
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

    /// Check if a device is trusted
    pub fn is_device_trusted(&self, mac_address: &str) -> Result<bool> {
        let conn = self.app_conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM trusted_devices WHERE mac_address = ?1"
        )?;

        let count: i64 = stmt.query_row([mac_address], |row| row.get(0))?;
        Ok(count > 0)
    }

    /// Trust a device
    pub fn trust_device(&self, mac_address: &str, device_name: &str) -> Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let conn = self.app_conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO trusted_devices (mac_address, device_name, trusted_at, trusted_by)
             VALUES (?1, ?2, ?3, 'user')",
            params![mac_address, device_name, timestamp],
        )?;

        Ok(())
    }

    /// Untrust a device
    pub fn untrust_device(&self, mac_address: &str) -> Result<()> {
        let conn = self.app_conn.lock().unwrap();
        conn.execute(
            "DELETE FROM trusted_devices WHERE mac_address = ?1",
            [mac_address],
        )?;
        Ok(())
    }

    /// Get all trusted devices
    pub fn get_trusted_devices(&self) -> Result<Vec<(String, String)>> {
        let conn = self.app_conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT mac_address, device_name FROM trusted_devices ORDER BY trusted_at DESC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
            ))
        })?;

        let mut devices = Vec::new();
        for row in rows {
            devices.push(row?);
        }

        Ok(devices)
    }
}
