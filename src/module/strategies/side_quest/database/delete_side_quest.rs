use rusqlite::params;

use crate::util::database::Database;

impl Database {
    pub(in crate::module) fn _side_quest__delete_side_quest(&self, quest_id: i64) -> rusqlite::Result<()> {
        let conn = self.app_conn.lock().unwrap();
        conn.execute(
            "DELETE FROM side_quests WHERE id = ?1",
            params![quest_id],
        )?;

        Ok(())
    }
}
