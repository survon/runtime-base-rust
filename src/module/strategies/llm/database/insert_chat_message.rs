use rusqlite::params;

use crate::module::strategies::llm::database::ChatMessage;
use crate::util::database::Database;

impl Database {
    pub(in crate::module) fn _llm__insert_chat_message(
        &self,
        message: ChatMessage,
    ) -> rusqlite::Result<i64> {
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
}
