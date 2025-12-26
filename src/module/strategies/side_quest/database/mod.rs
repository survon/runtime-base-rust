mod trait_side_quest_database;
mod init_schema;
mod create_side_quest;
mod get_active_side_quests;
mod complete_side_quest;
mod delete_side_quest;
mod get_quests_by_topic;
mod get_quests_with_deadlines;

use chrono::{DateTime, Utc};

use crate::util::database::Database;
use super::{SideQuest, QuestUrgency};

pub use trait_side_quest_database::SideQuestDatabase;

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
