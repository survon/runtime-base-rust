use rusqlite::params;
use crate::util::database::Database;

impl Database {
    pub(in crate::module) fn _overseer__delete_device(&self, mac_address: &str) -> rusqlite::Result<()> {
        let conn = self.app_conn.lock().unwrap();
        conn.execute(
            "DELETE FROM known_devices WHERE mac_address = ?1",
            params![mac_address],
        )?;
        Ok(())
    }
}
