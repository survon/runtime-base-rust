use crossterm::event::KeyCode;

use crate::{
    modules::side_quest::{
        database::SideQuestDatabase,
        handler::{SideQuestHandler, SideQuestView},
    },
    util::io::event::AppEvent
};

impl SideQuestHandler {
    pub(super) fn handle_detail_key(&mut self, key_code: KeyCode) -> Option<AppEvent> {
        match key_code {
            KeyCode::Esc => {
                self.current_view = SideQuestView::QuestList;
                Some(AppEvent::NoOp)
            }
            KeyCode::Char('c') => {
                // Complete quest from detail view
                if let Some(quest) = self.quests.get_mut(self.selected_index) {
                    if let Err(e) = self.database.complete_side_quest(quest.id) {
                        self.status_message = Some(format!("Error: {}", e));
                    } else {
                        quest.complete();
                        self.status_message = Some(format!("✓ Completed: {}", quest.title));
                        self.current_view = SideQuestView::QuestList;
                        self.load_quests();
                    }
                }
                None
            }
            _ => None,
        }
    }
}
