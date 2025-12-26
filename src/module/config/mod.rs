mod base_module_config;
mod service_discovery_config;
mod generic_config;
mod typed_module_config;
mod is_blinkable;
mod validation;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub use crate::module::strategies::{
    album::{AlbumConfig, AlbumBindings, Track},
    com::{ComConfig, ComBindings},
    knowledge::{KnowledgeConfig, KnowledgeBindings},
    llm::{LlmConfig, LlmBindings},
    monitoring::{MonitoringConfig, MonitoringBindings},
    overseer::{OverseerConfig, OverseerBindings},
    side_quest::{SideQuestConfig, SideQuestBindings},
    valve_control::{ValveControlConfig, ValveControlBindings},
};

pub use base_module_config::BaseModuleConfig;
pub use generic_config::GenericConfig;
pub use typed_module_config::TypedModuleConfig;
pub use service_discovery_config::ServiceDiscoveryConfig;
pub use validation::{ConfigValidator, ValidationError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    pub name: String,
    pub module_type: String,
    pub bus_topic: String,
    pub template: String,
    pub bindings: HashMap<String, serde_json::Value>,

    // Optional fields for different module types
    pub ports: Option<Vec<String>>,
    pub messages: Option<Vec<String>>,
    pub game_type: Option<String>,
    pub model: Option<String>,
    pub view_type: Option<String>,
    pub thresholds: Option<HashMap<String, f64>>,
    pub rules: Option<HashMap<String, String>>,
}


/// Supported template list
pub fn get_supported_templates() -> Vec<&'static str> {
    vec![
        "gauge_card",
        "chart_card",
        "status_badge_card",
        "toggle_switch",
        "activity_card",
        "llm_card",
        "side_quest_card",
        "overseer_card",
        "",  // Empty template for hidden modules
    ]
}

/// Supported module types
pub fn get_supported_module_types() -> Vec<&'static str> {
    vec![
        "monitoring",
        "valve_control",
        "llm",
        "side_quest",
        "overseer",
        "album",
        "knowledge",
        "com",
        "system",
    ]
}
