use crate::util::database::Database;
use crate::module::strategies::side_quest::{
    database::parse_quest_row,
    SideQuest,
};

impl Database {
    pub(in crate::module) fn _side_quest__get_quests_by_topic(&self, topic: &str) -> rusqlite::Result<Vec<SideQuest>> {
        let conn = self.app_conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, description, topic, urgency, trigger_date, created_at, completed_at, is_active
             FROM side_quests
             WHERE topic = ?1 AND is_active = 1
             ORDER BY created_at DESC"
        )?;

        let quests = stmt.query_map([topic], |row| {
            parse_quest_row(row)
        })?;

        quests.collect()
    }
}
