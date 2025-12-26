use crate::module::strategies::side_quest::{
    database::{parse_quest_row, Database},
    SideQuest,
};

impl Database {
    pub(in crate::module) fn _side_quest__get_active_side_quests(&self) -> rusqlite::Result<Vec<SideQuest>> {
        let conn = self.app_conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, description, topic, urgency, trigger_date, created_at, completed_at, is_active
             FROM side_quests
             WHERE is_active = 1
             ORDER BY
                CASE urgency
                    WHEN 'Critical' THEN 0
                    WHEN 'Pressing' THEN 1
                    WHEN 'Moderate' THEN 2
                    WHEN 'Casual' THEN 3
                    WHEN 'Chill' THEN 4
                END,
                created_at DESC"
        )?;

        let quests = stmt.query_map([], |row| {
            parse_quest_row(row)
        })?;

        quests.collect()
    }
}
