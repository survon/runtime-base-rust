use chrono::{DateTime, Utc};

use crate::util::database::Database;
use crate::module::strategies::side_quest::{
    QuestUrgency,
    SideQuest,
};

/// Trait to add Side Quest-specific database operations to Database
pub trait SideQuestDatabase {
    fn init_side_quest_schema(&self) -> rusqlite::Result<()>;

    // Quest creation and management
    fn create_side_quest(
        &self,
        title: &str,
        description: Option<String>,
        topic: &str,
        urgency: &QuestUrgency,
        trigger_date: Option<DateTime<Utc>>,
    ) -> rusqlite::Result<i64>;

    fn complete_side_quest(&self, quest_id: i64) -> rusqlite::Result<()>;
    fn delete_side_quest(&self, quest_id: i64) -> rusqlite::Result<()>;

    // Quest queries
    fn get_active_side_quests(&self) -> rusqlite::Result<Vec<SideQuest>>;
    fn get_quests_by_topic(&self, topic: &str) -> rusqlite::Result<Vec<SideQuest>>;
    fn get_quests_with_deadlines(&self, days_ahead: i64) -> rusqlite::Result<Vec<SideQuest>>;
}

impl SideQuestDatabase for Database {
    fn init_side_quest_schema(&self) -> rusqlite::Result<()> {
        self._side_quest__init_schema()
    }

    fn create_side_quest(
        &self,
        title: &str,
        description: Option<String>,
        topic: &str,
        urgency: &QuestUrgency,
        trigger_date: Option<DateTime<Utc>>,
    ) -> rusqlite::Result<i64> {
        self._side_quest__create_side_quest(title, description, topic, urgency, trigger_date)
    }

    fn complete_side_quest(&self, quest_id: i64) -> rusqlite::Result<()> {
        self._side_quest__complete_side_quest(quest_id)
    }

    fn delete_side_quest(&self, quest_id: i64) -> rusqlite::Result<()> {
        self._side_quest__delete_side_quest(quest_id)
    }

    fn get_active_side_quests(&self) -> rusqlite::Result<Vec<SideQuest>> {
        self._side_quest__get_active_side_quests()
    }

    fn get_quests_by_topic(&self, topic: &str) -> rusqlite::Result<Vec<SideQuest>> {
        self._side_quest__get_quests_by_topic(topic)
    }

    fn get_quests_with_deadlines(&self, days_ahead: i64) -> rusqlite::Result<Vec<SideQuest>> {
       self._side_quest__get_quests_with_deadlines(days_ahead)
    }
}
