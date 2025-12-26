use serde::{Deserialize, Serialize};

use crate::module::BaseModuleConfig;

/// Knowledge module (document collections)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeConfig {
    #[serde(flatten)]
    pub base: BaseModuleConfig,
    pub bindings: KnowledgeBindings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBindings {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub last_updated: Option<String>,
}
