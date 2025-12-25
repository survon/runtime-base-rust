use crate::util::database::Database;

use super::{ChatMessage, KnowledgeChunk};

/// Trait to add LLM-specific database operations to Database
pub trait LlmDatabase {
    fn init_llm_schema(&self) -> rusqlite::Result<()>;

    // Chat operations
    fn insert_chat_message(&self, message: ChatMessage) -> rusqlite::Result<i64>;
    fn get_chat_history(&self, session_id: &str, limit: usize) -> rusqlite::Result<Vec<ChatMessage>>;

    // Knowledge base operations
    fn insert_knowledge_chunk(&self, chunk: KnowledgeChunk) -> rusqlite::Result<()>;
    fn search_knowledge(&self, query: &str, domains: &[String], limit: usize) -> rusqlite::Result<Vec<KnowledgeChunk>>;
    fn clear_knowledge(&self) -> rusqlite::Result<()>;
}

impl LlmDatabase for Database {
    fn init_llm_schema(&self) -> rusqlite::Result<()> {
        self._llm__init_schema()
    }

    fn insert_chat_message(&self, message: ChatMessage) -> rusqlite::Result<i64> {
        self._llm__insert_chat_message(message)
    }

    fn get_chat_history(&self, session_id: &str, limit: usize) -> rusqlite::Result<Vec<ChatMessage>> {
        self._llm__get_chat_history(session_id, limit)
    }

    fn insert_knowledge_chunk(&self, chunk: KnowledgeChunk) -> rusqlite::Result<()> {
        self._llm__insert_knowledge_chunk(chunk)
    }

    fn search_knowledge(&self, query: &str, domains: &[String], limit: usize) -> rusqlite::Result<Vec<KnowledgeChunk>> {
        self._llm__search_knowledge(query, domains, limit)
    }

    fn clear_knowledge(&self) -> rusqlite::Result<()> {
        self._llm__clear_knowledge()
    }
}
