use rusqlite::params;
use crate::util::database::Database;

impl Database {
    pub(in crate::module) fn _overseer__set_device_trust(&self, mac_address: &str, trusted: bool) -> rusqlite::Result<()> {
        let conn = self.app_conn.lock().unwrap();
        let rows_affected = conn.execute(
            "UPDATE known_devices SET is_trusted = ?1 WHERE mac_address = ?2",
            params![if trusted { 1 } else { 0 }, mac_address],
        )?;

        if rows_affected == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        Ok(())
    }
}
