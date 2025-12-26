use super::OverseerHandler;

impl OverseerHandler {
    pub(in crate::module) fn handle_restore_module(&mut self) {
        if self.selected_index < self.archived_modules.len() {
            let archive_name = &self.archived_modules[self.selected_index];

            match self.restore_module(archive_name, None) {
                Ok(_) => {
                    self.status_message = Some(format!("✓ Restored {}", archive_name));
                    self.archived_modules.remove(self.selected_index);
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                    self.refresh_data_async();
                }
                Err(e) => {
                    self.status_message = Some(format!("Failed to restore: {}", e));
                }
            }
        }
    }
}
