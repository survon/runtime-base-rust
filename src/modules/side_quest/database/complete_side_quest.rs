use chrono::Utc;
use rusqlite::params;

use super::Database;

impl Database {
    pub(super) fn _side_quest__complete_side_quest(&self, quest_id: i64) -> rusqlite::Result<()> {
        let now = Utc::now().to_rfc3339();
        let conn = self.app_conn.lock().unwrap();

        conn.execute(
            "UPDATE side_quests
             SET completed_at = ?1, is_active = 0
             WHERE id = ?2",
            params![now, quest_id],
        )?;

        Ok(())
    }
}
