mod trait_overseer_database;
mod init_schema;
mod record_device_discovery;
mod update_device_metadata;
mod is_device_trusted;
mod set_device_trust;
mod trust_device;
mod get_trusted_devices;
mod get_all_known_devices;
mod delete_device;

use rusqlite::{params, Result};
use serde::{Deserialize, Serialize};

use crate::log_info;
use crate::util::database::Database;

pub use trait_overseer_database::{OverseerDatabase};

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
