use crate::util::database::Database;

use super::KnownDevice;

impl Database {
    pub(super) fn _overseer__get_all_known_devices(&self) -> rusqlite::Result<Vec<KnownDevice>> {
        let conn = self.app_conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT mac_address, device_name, device_type, firmware_version,
                    first_seen, last_seen, is_trusted, rssi
             FROM known_devices
             ORDER BY last_seen DESC"
        )?;

        let devices = stmt
            .query_map([], |row| {
                Ok(KnownDevice {
                    mac_address: row.get(0)?,
                    device_name: row.get(1)?,
                    device_type: row.get(2)?,
                    firmware_version: row.get(3)?,
                    first_seen: row.get(4)?,
                    last_seen: row.get(5)?,
                    is_trusted: row.get::<_, i64>(6)? == 1,
                    rssi: row.get(7)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>, _>>()?;

        Ok(devices)
    }
}
