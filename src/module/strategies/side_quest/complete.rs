use chrono::Utc;

use super::SideQuest;

impl SideQuest {
    pub(in crate::module) fn complete(&mut self) {
        self.completed_at = Some(Utc::now());
        self.is_active = false;
    }
}
