// src/modules/side_quest/mod.rs

pub mod handler;
pub mod database;
pub use database::SideQuestDatabase;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestUrgency {
    Chill,      // Someday, no rush
    Casual,     // Would be cool to do soon
    Moderate,   // Should probably do this
    Pressing,   // Getting time sensitive
    Critical,   // Offer expires or deadline approaching
}

impl QuestUrgency {
    pub fn as_str(&self) -> &str {
        match self {
            QuestUrgency::Chill => "Chill",
            QuestUrgency::Casual => "Casual",
            QuestUrgency::Moderate => "Moderate",
            QuestUrgency::Pressing => "Pressing",
            QuestUrgency::Critical => "Critical",
        }
    }

    pub fn color_code(&self) -> &str {
        match self {
            QuestUrgency::Chill => "gray",
            QuestUrgency::Casual => "cyan",
            QuestUrgency::Moderate => "yellow",
            QuestUrgency::Pressing => "magenta",
            QuestUrgency::Critical => "red",
        }
    }

    pub fn all() -> Vec<QuestUrgency> {
        vec![
            QuestUrgency::Chill,
            QuestUrgency::Casual,
            QuestUrgency::Moderate,
            QuestUrgency::Pressing,
            QuestUrgency::Critical,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideQuest {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub topic: String,                  // "outdoor", "food", "learning", etc.
    pub urgency: QuestUrgency,
    pub trigger_date: Option<DateTime<Utc>>,  // When does offer expire / deadline hit
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

impl SideQuest {
    pub fn new(
        id: i64,
        title: String,
        description: Option<String>,
        topic: String,
        urgency: QuestUrgency,
        trigger_date: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            title,
            description,
            topic,
            urgency,
            trigger_date,
            created_at: Utc::now(),
            completed_at: None,
            is_active: true,
        }
    }

    pub fn complete(&mut self) {
        self.completed_at = Some(Utc::now());
        self.is_active = false;
    }

    pub fn display_summary(&self) -> String {
        let urgency_icon = match self.urgency {
            QuestUrgency::Chill => "☁️",
            QuestUrgency::Casual => "🌤️",
            QuestUrgency::Moderate => "⚡",
            QuestUrgency::Pressing => "🔥",
            QuestUrgency::Critical => "🚨",
        };

        let date_str = if let Some(date) = self.trigger_date {
            format!(" [by {}]", date.format("%Y-%m-%d"))
        } else {
            String::new()
        };

        format!(
            "{} {} - {}{}",
            urgency_icon,
            self.title,
            self.topic,
            date_str
        )
    }
}
