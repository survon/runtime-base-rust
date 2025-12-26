use serde::{Deserialize, Serialize};
use crate::module::BaseModuleConfig;

pub mod handler;
mod validation;

/// Monitoring module (gauges, charts, status badges)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    #[serde(flatten)]
    pub base: BaseModuleConfig,
    pub bindings: MonitoringBindings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringBindings {
    // SSP compact data keys
    pub a: f64,
    pub b: f64,
    pub c: f64,

    // Device metadata
    pub device_id: String,
    pub device_type: String,
    pub firmware_version: String,

    // Display configuration
    pub display_name: String,
    pub unit_of_measure_label: String,

    // Thresholds (optional, depends on template)
    #[serde(default)]
    pub max_value: Option<f64>,
    #[serde(default)]
    pub warn_threshold: Option<f64>,
    #[serde(default)]
    pub danger_threshold: Option<f64>,

    // Chart-specific (optional)
    #[serde(default)]
    pub chart_type: Option<String>, // "line", "bar", "sparkline"

    // Connection tracking
    #[serde(default)]
    pub is_connected: Option<bool>,
    #[serde(default)]
    pub seconds_since_update: Option<i64>,
    #[serde(default)]
    pub status_suffix: Option<String>,

    // Internal state
    #[serde(default)]
    pub _chart_history: Option<Vec<f64>>,
}
