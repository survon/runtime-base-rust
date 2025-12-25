use chrono::{DateTime, Utc};

use super::{SideQuest, QuestUrgency};

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
}
