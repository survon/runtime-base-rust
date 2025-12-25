use super::QuestUrgency;

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
}
