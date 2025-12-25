use chrono::{DateTime, Utc};
use rusqlite::params;

use crate::util::database::Database;

use super::QuestUrgency;

impl Database {
    pub(super) fn _side_quest__create_side_quest(
        &self,
        title: &str,
        description: Option<String>,
        topic: &str,
        urgency: &QuestUrgency,
        trigger_date: Option<DateTime<Utc>>,
    ) -> rusqlite::Result<i64> {
        let now = Utc::now().to_rfc3339();
        let trigger_str = trigger_date.map(|d| d.to_rfc3339());
        let conn = self.app_conn.lock().unwrap();

        conn.execute(
            "INSERT INTO side_quests (title, description, topic, urgency, trigger_date, created_at, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)",
            params![
                title,
                description,
                topic,
                urgency.as_str(),
                trigger_str,
                now,
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }
}
