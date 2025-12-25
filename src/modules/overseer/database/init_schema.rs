use crate::{
    log_info,
    util::database::Database,
};

impl Database {
    pub(super) fn _overseer__init_schema(&self) -> rusqlite::Result<()> {
        let conn = self.app_conn.lock().unwrap();

        // Known devices table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS known_devices (
                mac_address TEXT PRIMARY KEY,
                device_name TEXT NOT NULL,
                device_type TEXT,
                firmware_version TEXT,
                first_seen INTEGER NOT NULL,
                last_seen INTEGER NOT NULL,
                is_trusted INTEGER NOT NULL DEFAULT 0,
                rssi INTEGER
            )",
            [],
        )?;

        // Create index for trust queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_trusted
             ON known_devices(is_trusted)",
            [],
        )?;

        // MIGRATION: Copy data from old trusted_devices table if it exists
        let old_table_exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='trusted_devices'",
            [],
            |row| row.get(0)
        ).unwrap_or(0);

        if old_table_exists > 0 {
            log_info!("Migrating data from trusted_devices to known_devices...");

            // Copy trusted devices to new table
            conn.execute(
                "INSERT OR IGNORE INTO known_devices (mac_address, device_name, first_seen, last_seen, is_trusted)
                 SELECT mac_address, device_name, trusted_at, trusted_at, 1
                 FROM trusted_devices",
                [],
            )?;

            // Drop old table
            conn.execute("DROP TABLE IF EXISTS trusted_devices", [])?;

            log_info!("Migration complete!");
        }

        Ok(())
    }
}
