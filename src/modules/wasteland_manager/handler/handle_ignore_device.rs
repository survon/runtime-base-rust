use super::WastelandManagerHandler;

impl WastelandManagerHandler {
    pub(super) fn handle_ignore_device(&mut self) {
        if self.selected_index < self.pending_devices.len() {
            self.pending_devices.remove(self.selected_index);
            if self.selected_index > 0 {
                self.selected_index -= 1;
            }
            self.status_message = Some("Device ignored".to_string());
        }
    }
}
