use super::QuestUrgency;

impl QuestUrgency {
    pub fn color_code(&self) -> &str {
        match self {
            QuestUrgency::Chill => "gray",
            QuestUrgency::Casual => "cyan",
            QuestUrgency::Moderate => "yellow",
            QuestUrgency::Pressing => "magenta",
            QuestUrgency::Critical => "red",
        }
    }
}
