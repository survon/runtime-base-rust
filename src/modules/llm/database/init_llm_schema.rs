use crate::util::database::Database;

impl Database {
    pub(super) fn _llm__init_schema(&self) -> rusqlite::Result<()> {
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
}
