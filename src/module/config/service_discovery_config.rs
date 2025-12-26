use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryConfig {
    pub enabled: bool,
    pub method: String,
    pub scan_pattern: String,
    pub metadata_path: String,
    pub api_port: u16,
    pub web_port: u16,
    pub scan_interval_seconds: u64,
    pub required_contract: String,
}
