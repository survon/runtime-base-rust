use crate::modules::overseer::database::WastelandDatabase;

use super::OverseerHandler;

impl OverseerHandler {
    pub(super) fn handle_toggle_trust(&mut self) {
        if self.selected_index < self.known_devices.len() {
            let device = &self.known_devices[self.selected_index];
            let mac = device.mac_address.clone();
            let new_trust = !device.is_trusted;

            if let Err(e) = self.database.set_device_trust(&mac, new_trust) {
                self.status_message = Some(format!("Failed to update trust: {}", e));
            } else {
                self.status_message = Some(if new_trust {
                    "Device trusted".to_string()
                } else {
                    "Device untrusted".to_string()
                });
                self.refresh_known_devices();
            }
        }
    }
}
