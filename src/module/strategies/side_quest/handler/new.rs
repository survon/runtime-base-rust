use crate::util::{
    database::Database,
    io::bus::MessageBus,
};
use crate::module::strategies::side_quest::{
    handler::{CreateStep, SideQuestHandler, SideQuestView},
    QuestUrgency
};

impl SideQuestHandler {
    pub fn new(database: Database, message_bus: MessageBus) -> Self {
        let mut handler = Self {
            current_view: SideQuestView::QuestList,
            selected_index: 0,
            quests: Vec::new(),
            database: database.clone(),
            message_bus,
            create_step: CreateStep::Title,
            form_title: String::new(),
            form_description: String::new(),
            form_topic: String::new(),
            form_urgency: QuestUrgency::Casual,
            form_trigger_date: None,
            available_topics: vec![
                "outdoor".to_string(),
                "food".to_string(),
                "learning".to_string(),
                "fitness".to_string(),
                "social".to_string(),
                "creative".to_string(),
                "adventure".to_string(),
                "hobby".to_string(),
            ],
            status_message: None,
        };

        // Load quests from database
        handler.load_quests();

        handler
    }
}
