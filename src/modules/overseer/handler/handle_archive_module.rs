use super::OverseerHandler;

impl OverseerHandler {
    pub(super) fn handle_archive_module(&mut self) {
        if self.selected_index < self.installed_modules.len() {
            let module_name = &self.installed_modules[self.selected_index];

            match self.archive_module(module_name) {
                Ok(_) => {
                    self.status_message = Some(format!("✓ Archived {}", module_name));
                    self.installed_modules.remove(self.selected_index);
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                    self.refresh_data_async();
                }
                Err(e) => {
                    self.status_message = Some(format!("Failed to archive: {}", e));
                }
            }
        }
    }
}
