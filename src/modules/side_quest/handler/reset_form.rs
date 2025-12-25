use crate::modules::side_quest::{
    QuestUrgency,
    handler::SideQuestHandler
};

impl SideQuestHandler {
    pub(super) fn reset_form(&mut self) {
        self.form_title = String::new();
        self.form_description = String::new();
        self.form_topic = String::new();
        self.form_urgency = QuestUrgency::Casual;
        self.form_trigger_date = None;
        self.selected_index = 0;
    }
}
