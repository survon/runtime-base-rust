use rusqlite::params;

use crate::util::database::Database;

impl Database {
    pub(super) fn _overseer__record_device_discovery(
        &self,
        mac_address: &str,
        device_name: &str,
        rssi: i16,
    ) -> rusqlite::Result<bool> {
        let conn = self.app_conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Check if device already exists
        let exists: bool = conn.query_row(
            "SELECT COUNT(*) FROM known_devices WHERE mac_address = ?1",
            params![mac_address],
            |row| row.get::<_, i64>(0).map(|count| count > 0),
        )?;

        if exists {
            // Update last seen, RSSI, and device name
            conn.execute(
                "UPDATE known_devices
                 SET last_seen = ?1, rssi = ?2, device_name = ?3
                 WHERE mac_address = ?4",
                params![now, rssi, device_name, mac_address],
            )?;
            Ok(false) // Not a new device
        } else {
            // Insert new device (untrusted by default)
            conn.execute(
                "INSERT INTO known_devices
                 (mac_address, device_name, first_seen, last_seen, is_trusted, rssi)
                 VALUES (?1, ?2, ?3, ?4, 0, ?5)",
                params![mac_address, device_name, now, now, rssi],
            )?;
            Ok(true) // New device discovered
        }
    }
}
