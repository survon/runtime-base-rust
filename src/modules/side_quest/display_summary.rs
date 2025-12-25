use super::{SideQuest, QuestUrgency};

impl SideQuest {
    pub(super) fn display_summary(&self) -> String {
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
