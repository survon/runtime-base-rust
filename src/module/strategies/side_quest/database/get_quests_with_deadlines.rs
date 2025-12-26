use chrono::Utc;

use crate::util::database::Database;
use crate::module::strategies::side_quest::{
    database::parse_quest_row,
    SideQuest,
};

impl Database {
    pub(in crate::module) fn _side_quest__get_quests_with_deadlines(&self, days_ahead: i64) -> rusqlite::Result<Vec<SideQuest>> {
        let cutoff = (Utc::now() + chrono::Duration::days(days_ahead)).to_rfc3339();
        let conn = self.app_conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, title, description, topic, urgency, trigger_date, created_at, completed_at, is_active
             FROM side_quests
             WHERE is_active = 1
             AND trigger_date IS NOT NULL
             AND trigger_date <= ?1
             ORDER BY trigger_date ASC"
        )?;

        let quests = stmt.query_map([cutoff], |row| {
            parse_quest_row(row)
        })?;

        quests.collect()
    }
}
