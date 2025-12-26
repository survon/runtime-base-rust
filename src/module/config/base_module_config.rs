use serde::{Deserialize, Serialize};

/// Base configuration that all modules must have
///
/// *Note: `module_type` is NOT included because it's consumed by the enum's tag
/// since the enum variant already tells us the type*
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseModuleConfig {
    pub name: String,
    pub bus_topic: String,
    pub template: String,

    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_blinkable: Option<bool>,
}
