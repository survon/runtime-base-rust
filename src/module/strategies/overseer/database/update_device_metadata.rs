use rusqlite::params;

use crate::util::database::Database;

impl Database {
    pub(in crate::module) fn _overseer__update_device_metadata(
        &self,
        mac_address: &str,
        device_type: &str,
        firmware_version: &str,
    ) -> rusqlite::Result<()> {
        let conn = self.app_conn.lock().unwrap();
        conn.execute(
            "UPDATE known_devices
             SET device_type = ?1, firmware_version = ?2
             WHERE mac_address = ?3",
            params![device_type, firmware_version, mac_address],
        )?;
        Ok(())
    }
}
