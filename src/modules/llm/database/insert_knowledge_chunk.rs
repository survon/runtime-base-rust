use rusqlite::params;

use crate::{
    modules::llm::database::KnowledgeChunk,
    util::database::Database
};

impl Database {
    pub(super) fn _llm__insert_knowledge_chunk(&self, chunk: KnowledgeChunk) -> rusqlite::Result<()> {
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
}
