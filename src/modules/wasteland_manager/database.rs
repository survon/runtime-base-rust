// src/modules/wasteland_manager/database.rs
// Database operations for the Wasteland Manager module (device trust/discovery)

use rusqlite::{params, Result};
use serde::{Deserialize, Serialize};

use crate::util::database::Database;

/// Device record from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownDevice {
    pub mac_address: String,
    pub device_name: String,
    pub device_type: Option<String>,
    pub firmware_version: Option<String>,
    pub first_seen: i64,
    pub last_seen: i64,
    pub is_trusted: bool,
    pub rssi: Option<i16>,
}

/// Trait to add Wasteland Manager-specific database operations to Database
pub trait WastelandDatabase {
    fn init_wasteland_schema(&self) -> Result<()>;

    // Device discovery and tracking
    fn record_device_discovery(&self, mac_address: &str, device_name: &str, rssi: i16) -> Result<bool>;
    fn update_device_metadata(&self, mac_address: &str, device_type: &str, firmware_version: &str) -> Result<()>;

    // Device trust management
    fn is_device_trusted(&self, mac_address: &str) -> Result<bool>;
    fn set_device_trust(&self, mac_address: &str, trusted: bool) -> Result<()>;
    fn trust_device(&self, mac_address: &str, device_name: &str) -> Result<()>;
    fn untrust_device(&self, mac_address: &str) -> Result<()>;

    // Device queries
    fn get_trusted_devices(&self) -> Result<Vec<(String, String)>>;
    fn get_all_known_devices(&self) -> Result<Vec<KnownDevice>>;
    fn delete_device(&self, mac_address: &str) -> Result<()>;
}

impl WastelandDatabase for Database {
    fn init_wasteland_schema(&self) -> Result<()> {
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
            println!("Migrating data from trusted_devices to known_devices...");

            // Copy trusted devices to new table
            conn.execute(
                "INSERT OR IGNORE INTO known_devices (mac_address, device_name, first_seen, last_seen, is_trusted)
                 SELECT mac_address, device_name, trusted_at, trusted_at, 1
                 FROM trusted_devices",
                [],
            )?;

            // Drop old table
            conn.execute("DROP TABLE IF EXISTS trusted_devices", [])?;

            println!("Migration complete!");
        }

        Ok(())
    }

    fn record_device_discovery(
        &self,
        mac_address: &str,
        device_name: &str,
        rssi: i16,
    ) -> Result<bool> {
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

    fn update_device_metadata(
        &self,
        mac_address: &str,
        device_type: &str,
        firmware_version: &str,
    ) -> Result<()> {
        let conn = self.app_conn.lock().unwrap();
        conn.execute(
            "UPDATE known_devices
             SET device_type = ?1, firmware_version = ?2
             WHERE mac_address = ?3",
            params![device_type, firmware_version, mac_address],
        )?;
        Ok(())
    }

    fn is_device_trusted(&self, mac_address: &str) -> Result<bool> {
        let conn = self.app_conn.lock().unwrap();

        let trusted: Result<i64, _> = conn.query_row(
            "SELECT is_trusted FROM known_devices WHERE mac_address = ?1",
            params![mac_address],
            |row| row.get(0),
        );

        Ok(trusted.unwrap_or(0) == 1)
    }

    fn set_device_trust(&self, mac_address: &str, trusted: bool) -> Result<()> {
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

    fn trust_device(&self, mac_address: &str, device_name: &str) -> Result<()> {
        // Ensure device exists in database
        self.record_device_discovery(mac_address, device_name, 0)?;
        // Then trust it
        self.set_device_trust(mac_address, true)
    }

    fn untrust_device(&self, mac_address: &str) -> Result<()> {
        self.set_device_trust(mac_address, false)
    }

    fn get_trusted_devices(&self) -> Result<Vec<(String, String)>> {
        let conn = self.app_conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT mac_address, device_name FROM known_devices WHERE is_trusted = 1"
        )?;

        let devices = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(devices)
    }

    fn get_all_known_devices(&self) -> Result<Vec<KnownDevice>> {
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
            .collect::<Result<Vec<_>, _>>()?;

        Ok(devices)
    }

    fn delete_device(&self, mac_address: &str) -> Result<()> {
        let conn = self.app_conn.lock().unwrap();
        conn.execute(
            "DELETE FROM known_devices WHERE mac_address = ?1",
            params![mac_address],
        )?;
        Ok(())
    }
}
