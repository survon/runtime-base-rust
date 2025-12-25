pub mod handler;
pub mod database;
mod quest_urgency;
mod new;
mod complete;
mod display_summary;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use quest_urgency::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideQuest {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub topic: String, // "outdoor", "food", "learning", etc.
    pub urgency: QuestUrgency,
    pub trigger_date: Option<DateTime<Utc>>, // When does offer expire / deadline hit
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}
