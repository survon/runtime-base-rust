use chrono::Utc;

use super::SideQuest;

impl SideQuest {
    pub(super) fn complete(&mut self) {
        self.completed_at = Some(Utc::now());
        self.is_active = false;
    }
}
