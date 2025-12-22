use crate::modules::overseer::database::WastelandDatabase;

use super::OverseerHandler;

impl OverseerHandler {
    pub(super) fn handle_delete_device(&mut self) {
        if self.selected_index < self.known_devices.len() {
            let device = &self.known_devices[self.selected_index];
            let mac = device.mac_address.clone();

            if let Err(e) = self.database.delete_device(&mac) {
                self.status_message = Some(format!("Failed to delete: {}", e));
            } else {
                self.status_message = Some("Device deleted".to_string());
                self.refresh_known_devices();
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
        }
    }
}
