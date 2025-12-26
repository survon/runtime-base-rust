mod chat_message;
mod init_llm_schema;
mod insert_chat_message;
mod get_chat_history;
mod insert_knowledge_chunk;
mod search_knowledge;
mod clear_knowledge;
mod trait_llm_database;

use rusqlite::{params, Result};
use serde::{Deserialize, Serialize};

use crate::log_debug;
use crate::util::database::Database;

pub use chat_message::{ChatMessage};
pub use trait_llm_database::LlmDatabase;

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
