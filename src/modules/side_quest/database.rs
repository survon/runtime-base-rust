// src/modules/side_quest/database.rs
// Database operations for the Side Quest module (task/quest management)

use rusqlite::{params, Result};
use chrono::{DateTime, Utc};

use crate::util::database::Database;
use super::{SideQuest, QuestUrgency};

/// Trait to add Side Quest-specific database operations to Database
pub trait SideQuestDatabase {
    fn init_side_quest_schema(&self) -> Result<()>;

    // Quest creation and management
    fn create_side_quest(
        &self,
        title: &str,
        description: Option<String>,
        topic: &str,
        urgency: &QuestUrgency,
        trigger_date: Option<DateTime<Utc>>,
    ) -> Result<i64>;

    fn complete_side_quest(&self, quest_id: i64) -> Result<()>;
    fn delete_side_quest(&self, quest_id: i64) -> Result<()>;

    // Quest queries
    fn get_active_side_quests(&self) -> Result<Vec<SideQuest>>;
    fn get_quests_by_topic(&self, topic: &str) -> Result<Vec<SideQuest>>;
    fn get_quests_with_deadlines(&self, days_ahead: i64) -> Result<Vec<SideQuest>>;
}

impl SideQuestDatabase for Database {
    fn init_side_quest_schema(&self) -> Result<()> {
        let conn = self.app_conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS side_quests (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                description TEXT,
                topic TEXT NOT NULL,
                urgency TEXT NOT NULL,
                trigger_date TEXT,
                created_at TEXT NOT NULL,
                completed_at TEXT,
                is_active INTEGER NOT NULL DEFAULT 1
            )",
            [],
        )?;

        Ok(())
    }

    fn create_side_quest(
        &self,
        title: &str,
        description: Option<String>,
        topic: &str,
        urgency: &QuestUrgency,
        trigger_date: Option<DateTime<Utc>>,
    ) -> Result<i64> {
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

    fn get_active_side_quests(&self) -> Result<Vec<SideQuest>> {
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

    fn complete_side_quest(&self, quest_id: i64) -> Result<()> {
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

    fn delete_side_quest(&self, quest_id: i64) -> Result<()> {
        let conn = self.app_conn.lock().unwrap();
        conn.execute(
            "DELETE FROM side_quests WHERE id = ?1",
            params![quest_id],
        )?;

        Ok(())
    }

    fn get_quests_by_topic(&self, topic: &str) -> Result<Vec<SideQuest>> {
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

    fn get_quests_with_deadlines(&self, days_ahead: i64) -> Result<Vec<SideQuest>> {
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

// Helper function to parse a quest row
fn parse_quest_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SideQuest> {
    let urgency_str: String = row.get(4)?;
    let urgency = match urgency_str.as_str() {
        "Chill" => QuestUrgency::Chill,
        "Casual" => QuestUrgency::Casual,
        "Moderate" => QuestUrgency::Moderate,
        "Pressing" => QuestUrgency::Pressing,
        "Critical" => QuestUrgency::Critical,
        _ => QuestUrgency::Casual,
    };

    let trigger_date: Option<String> = row.get(5)?;
    let trigger_parsed = trigger_date.and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let created_at: String = row.get(6)?;
    let created_parsed = DateTime::parse_from_rfc3339(&created_at)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let completed_at: Option<String> = row.get(7)?;
    let completed_parsed = completed_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    Ok(SideQuest {
        id: row.get(0)?,
        title: row.get(1)?,
        description: row.get(2)?,
        topic: row.get(3)?,
        urgency,
        trigger_date: trigger_parsed,
        created_at: created_parsed,
        completed_at: completed_parsed,
        is_active: row.get(8)?,
    })
}
