mod validation;

use serde::{Deserialize, Serialize};

use crate::module::config::BaseModuleConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComBindings {
    #[serde(default)]
    pub state: Option<bool>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub toggle_on_label: Option<String>,
    #[serde(default)]
    pub toggle_off_label: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub ports: Option<Vec<String>>,
    #[serde(default)]
    pub messages: Option<Vec<String>>,
    #[serde(default)]
    pub activity_log: Option<Vec<String>>,
    #[serde(default)]
    pub status: Option<String>,
}

/// Communication module (toggle switches, activity logs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComConfig {
    #[serde(flatten)]
    pub base: BaseModuleConfig,
    pub bindings: ComBindings,
}
