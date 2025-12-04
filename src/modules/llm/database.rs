// src/modules/llm/database.rs
// Database operations for the LLM module (chat history and knowledge base)

use rusqlite::{params, Result};
use serde::{Deserialize, Serialize};

use crate::util::database::Database;

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

/// Trait to add LLM-specific database operations to Database
pub trait LlmDatabase {
    fn init_llm_schema(&self) -> Result<()>;

    // Chat operations
    fn insert_chat_message(&self, message: ChatMessage) -> Result<i64>;
    fn get_chat_history(&self, session_id: &str, limit: usize) -> Result<Vec<ChatMessage>>;

    // Knowledge base operations
    fn insert_knowledge_chunk(&self, chunk: KnowledgeChunk) -> Result<()>;
    fn search_knowledge(&self, query: &str, domains: &[String], limit: usize) -> Result<Vec<KnowledgeChunk>>;
    fn clear_knowledge(&self) -> Result<()>;
}

impl LlmDatabase for Database {
    fn init_llm_schema(&self) -> Result<()> {
        // Chat messages table
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
        }

        // Knowledge base FTS5 virtual table
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

        // Analytics tables
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

    fn insert_chat_message(&self, message: ChatMessage) -> Result<i64> {
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

    fn get_chat_history(&self, session_id: &str, limit: usize) -> Result<Vec<ChatMessage>> {
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

    fn insert_knowledge_chunk(&self, chunk: KnowledgeChunk) -> Result<()> {
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

    fn search_knowledge(&self, query: &str, domains: &[String], limit: usize) -> Result<Vec<KnowledgeChunk>> {
        let clean_query = sanitize_fts5_query(query);
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
        results = execute_search(self, &clean_query, domains, limit * 2)?;
        println!("Strategy 1 (AND search): found {} results", results.len());

        // Strategy 2: If no results, try OR search
        if results.is_empty() && clean_query.contains(' ') {
            let or_query = clean_query.split_whitespace().collect::<Vec<_>>().join(" OR ");
            println!("Strategy 2 (OR search): trying '{}'", or_query);
            results = execute_search(self, &or_query, domains, limit * 2)?;
            println!("Strategy 2 (OR search): found {} results", results.len());
        }

        // Strategy 3: If still no results, try each word individually
        if results.is_empty() {
            let words: Vec<&str> = clean_query.split_whitespace().collect();
            println!("Strategy 3 (individual words): trying {} words", words.len());
            for word in &words {
                let word_results = execute_search(self, word, domains, limit)?;
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

    fn clear_knowledge(&self) -> Result<()> {
        let conn = self.knowledge_conn.lock().unwrap();
        conn.execute("DELETE FROM knowledge", [])?;
        Ok(())
    }
}

// Helper functions

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

fn execute_search(db: &Database, search_query: &str, domains: &[String], limit: usize) -> Result<Vec<KnowledgeChunk>> {
    let conn = db.knowledge_conn.lock().unwrap();

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
