use crate::{log_error, log_info};

use super::{SideQuestHandler,SideQuestDatabase};

impl SideQuestHandler {
    pub(super) fn load_quests(&mut self) {
        match self.database.get_active_side_quests() {
            Ok(quests) => {
                self.quests = quests;
                log_info!("Loaded {} side quests", self.quests.len());
            }
            Err(e) => {
                log_error!("Failed to load side quests: {}", e);
                self.status_message = Some(format!("Error loading quests: {}", e));
            }
        }
    }
}
