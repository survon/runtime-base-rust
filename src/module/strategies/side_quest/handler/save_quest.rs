use crate::log_error;
use crate::module::strategies::side_quest::{
    database::SideQuestDatabase,
    handler::{SideQuestHandler, SideQuestView},
};

impl SideQuestHandler {
    pub(in crate::module) fn save_quest(&mut self) {
        let description = if self.form_description.is_empty() {
            None
        } else {
            Some(self.form_description.clone())
        };

        match self.database.create_side_quest(
            &self.form_title,
            description,
            &self.form_topic,
            &self.form_urgency,
            self.form_trigger_date,
        ) {
            Ok(quest_id) => {
                self.status_message = Some(format!("✓ Created: {}", self.form_title));

                // Publish calendar event if trigger date exists
                if let Some(date) = self.form_trigger_date {
                    self.publish_calendar_event(quest_id, date);
                }

                self.load_quests();
                self.current_view = SideQuestView::QuestList;
                self.reset_form();
            }
            Err(e) => {
                log_error!("Failed to create quest: {}", e);
                self.status_message = Some(format!("Error: {}", e));
            }
        }
    }
}
