use crate::util::database::Database;

impl Database {
    pub(in crate::module) fn _overseer__get_trusted_devices(&self) -> rusqlite::Result<Vec<(String, String)>> {
        let conn = self.app_conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT mac_address, device_name FROM known_devices WHERE is_trusted = 1"
        )?;

        let devices = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?
            .collect::<rusqlite::Result<Vec<_>, _>>()?;

        Ok(devices)
    }
}
