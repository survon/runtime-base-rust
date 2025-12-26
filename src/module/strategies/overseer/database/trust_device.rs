use crate::util::database::Database;

use super::trait_overseer_database::OverseerDatabase;

impl Database {
    pub(in crate::module) fn _overseer__trust_device(&self, mac_address: &str, device_name: &str) -> rusqlite::Result<()> {
        // Ensure device exists in database
        self.record_device_discovery(mac_address, device_name, 0)?;
        // Then trust it
        self.set_device_trust(mac_address, true)
    }
}
