use crossterm::event::KeyCode;

use crate::util::io::event::AppEvent;
use crate::module::strategies::side_quest::{
    database::SideQuestDatabase,
    handler::{
        CreateStep,
        SideQuestHandler,
        SideQuestView
    }
};

impl SideQuestHandler {
    pub(in crate::module) fn handle_quest_list_key(&mut self, key_code: KeyCode) -> Option<AppEvent> {
        match key_code {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                None
            }
            KeyCode::Down => {
                if self.selected_index < self.quests.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
                None
            }
            KeyCode::Char('n') => {
                // Start creating new quest
                self.current_view = SideQuestView::CreateQuest;
                self.create_step = CreateStep::Title;
                self.reset_form();
                self.status_message = Some("Enter quest title...".to_string());
                None
            }
            KeyCode::Char('c') => {
                // Complete selected quest
                if let Some(quest) = self.quests.get_mut(self.selected_index) {
                    if let Err(e) = self.database.complete_side_quest(quest.id) {
                        self.status_message = Some(format!("Error: {}", e));
                    } else {
                        quest.complete();
                        self.status_message = Some(format!("✓ Completed: {}", quest.title));
                        self.load_quests();
                    }
                }
                None
            }
            KeyCode::Char('d') => {
                // Delete selected quest
                if let Some(quest) = self.quests.get(self.selected_index) {
                    if let Err(e) = self.database.delete_side_quest(quest.id) {
                        self.status_message = Some(format!("Error: {}", e));
                    } else {
                        self.status_message = Some(format!("Deleted: {}", quest.title));
                        self.load_quests();
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                        }
                    }
                }
                None
            }
            KeyCode::Enter => {
                // View quest details
                if self.quests.get(self.selected_index).is_some() {
                    self.current_view = SideQuestView::QuestDetail;
                }
                None
            }
            _ => None,
        }
    }
}
