use rusqlite::params;
use crate::util::database::Database;

impl Database {
    pub(super) fn _overseer__is_device_trusted(&self, mac_address: &str) -> rusqlite::Result<bool> {
        let conn = self.app_conn.lock().unwrap();

        let trusted: rusqlite::Result<i64, _> = conn.query_row(
            "SELECT is_trusted FROM known_devices WHERE mac_address = ?1",
            params![mac_address],
            |row| row.get(0),
        );

        Ok(trusted.unwrap_or(0) == 1)
    }
}
