use crate::log_error;
use crate::modules::wasteland_manager::database::WastelandDatabase;

use super::WastelandManagerHandler;

impl WastelandManagerHandler {
    pub(super) fn refresh_known_devices(&mut self) {
        match self.database.get_all_known_devices() {
            Ok(devices) => {
                self.known_devices = devices;
                self.status_message = None;
            }
            Err(e) => {
                log_error!("Failed to load known devices: {}", e);
                self.status_message = Some(format!("Error loading devices: {}", e));
            }
        }
    }
}
