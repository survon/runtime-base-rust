pub mod handler;
pub mod database;
mod quest_urgency;
mod new;
mod complete;
mod display_summary;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use quest_urgency::*;
use crate::module::BaseModuleConfig;

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

/// Side Quest module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideQuestConfig {
    #[serde(flatten)]
    pub base: BaseModuleConfig,
    pub bindings: SideQuestBindings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideQuestBindings {
    pub current_view: String, // "QuestList", "CreateQuest", "QuestDetail"
    pub selected_index: i32,
    pub quests: Vec<String>,
    pub quest_count: i32,

    // Create form state
    pub create_step: String,
    pub form_title: String,
    pub form_description: String,
    pub form_topic: String,
    pub form_urgency: String,

    // Available options
    pub available_topics: Vec<String>,
    pub urgency_options: Vec<String>,

    // Detail view
    pub selected_quest_title: String,
    pub selected_quest_description: String,
    pub selected_quest_topic: String,
    pub selected_quest_urgency: String,
    pub selected_quest_trigger: String,

    #[serde(default)]
    pub status_message: Option<String>,
    #[serde(default)]
    pub is_blinkable: Option<bool>,
}
