use super::QuestUrgency;

impl QuestUrgency {
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
