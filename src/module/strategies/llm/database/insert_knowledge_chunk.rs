use rusqlite::params;

use crate::util::database::Database;
use crate::module::strategies::llm::database::KnowledgeChunk;

impl Database {
    pub(in crate::module) fn _llm__insert_knowledge_chunk(&self, chunk: KnowledgeChunk) -> rusqlite::Result<()> {
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
