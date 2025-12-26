use serde::{Deserialize, Serialize};
use crate::module::BaseModuleConfig;

pub mod handler;
mod validation;

/// Valve control module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValveControlConfig {
    #[serde(flatten)]
    pub base: BaseModuleConfig,
    pub bindings: ValveControlBindings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValveControlBindings {
    // SSP compact data
    pub a: i32, // valve_open: 0=closed, 1=open
    pub b: i32, // position: 0-100%
    pub c: i32, // message_count

    // Device metadata
    pub device_id: String,
    pub device_type: String,
    pub firmware_version: String,

    // UI display
    pub state: bool, // Derived from "a"
    pub label: String,
    pub toggle_on_label: String,
    pub toggle_off_label: String,
    pub description: String,
    pub is_blinkable: bool,
}
