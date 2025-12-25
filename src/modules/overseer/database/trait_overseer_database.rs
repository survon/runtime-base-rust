use rusqlite::params;
use crate::log_info;
use crate::modules::overseer::database::KnownDevice;
use crate::util::database::Database;

/// Trait to add Overseer-specific database operations to Database
pub trait OverseerDatabase {
    fn init_overseer_schema(&self) -> rusqlite::Result<()>;

    // Device discovery and tracking
    fn record_device_discovery(&self, mac_address: &str, device_name: &str, rssi: i16) -> rusqlite::Result<bool>;
    fn update_device_metadata(&self, mac_address: &str, device_type: &str, firmware_version: &str) -> rusqlite::Result<()>;

    // Device trust management
    fn is_device_trusted(&self, mac_address: &str) -> rusqlite::Result<bool>;
    fn set_device_trust(&self, mac_address: &str, trusted: bool) -> rusqlite::Result<()>;
    fn trust_device(&self, mac_address: &str, device_name: &str) -> rusqlite::Result<()>;
    fn untrust_device(&self, mac_address: &str) -> rusqlite::Result<()>;

    // Device queries
    fn get_trusted_devices(&self) -> rusqlite::Result<Vec<(String, String)>>;
    fn get_all_known_devices(&self) -> rusqlite::Result<Vec<KnownDevice>>;
    fn delete_device(&self, mac_address: &str) -> rusqlite::Result<()>;
}

impl OverseerDatabase for Database {
    fn init_overseer_schema(&self) -> rusqlite::Result<()> {
        self._overseer__init_schema()
    }

    fn record_device_discovery(
        &self,
        mac_address: &str,
        device_name: &str,
        rssi: i16,
    ) -> rusqlite::Result<bool> {
        self._overseer__record_device_discovery(mac_address, device_name, rssi)
    }

    fn update_device_metadata(
        &self,
        mac_address: &str,
        device_type: &str,
        firmware_version: &str,
    ) -> rusqlite::Result<()> {
        self._overseer__update_device_metadata(mac_address, device_type, firmware_version)
    }

    fn is_device_trusted(&self, mac_address: &str) -> rusqlite::Result<bool> {
        self._overseer__is_device_trusted(mac_address)
    }

    fn set_device_trust(&self, mac_address: &str, trusted: bool) -> rusqlite::Result<()> {
        self._overseer__set_device_trust(mac_address, trusted)
    }

    fn trust_device(&self, mac_address: &str, device_name: &str) -> rusqlite::Result<()> {
       self._overseer__trust_device(mac_address, device_name)
    }

    fn untrust_device(&self, mac_address: &str) -> rusqlite::Result<()> {
        self.set_device_trust(mac_address, false)
    }

    fn get_trusted_devices(&self) -> rusqlite::Result<Vec<(String, String)>> {
        self._overseer__get_trusted_devices()
    }

    fn get_all_known_devices(&self) -> rusqlite::Result<Vec<KnownDevice>> {
        self._overseer__get_all_known_devices()
    }

    fn delete_device(&self, mac_address: &str) -> rusqlite::Result<()> {
        self._overseer__delete_device(mac_address)
    }
}
