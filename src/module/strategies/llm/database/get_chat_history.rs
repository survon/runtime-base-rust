use rusqlite::params;

use crate::util::database::Database;
use crate::module::strategies::llm::database::ChatMessage;

impl Database {
    pub(in crate::module) fn _llm__get_chat_history(&self, session_id: &str, limit: usize) -> rusqlite::Result<Vec<ChatMessage>> {
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
}
