use serde::{Deserialize, Serialize};
use crate::module::BaseModuleConfig;

pub mod config_editor;
pub mod database;
pub mod handler;

/// Wasteland Manager module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverseerConfig {
    #[serde(flatten)]
    pub base: BaseModuleConfig,
    pub bindings: OverseerBindings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverseerBindings {
    pub current_view: String,
    pub selected_index: i32,
    pub pending_devices: Vec<String>,
    pub known_devices: Vec<String>,
    pub module_list: Vec<String>,
    pub installed_modules: Vec<String>,
    pub archived_modules: Vec<String>,

    #[serde(default)]
    pub status_message: Option<String>,
    #[serde(default)]
    pub is_blinkable: Option<bool>,
}
