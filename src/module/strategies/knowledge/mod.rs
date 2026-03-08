use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    #[serde(default)]
    pub document_count: Option<usize>,
    #[serde(default)]
    pub search_enabled: Option<bool>,
}

impl KnowledgeConfig {
    pub fn new(base: BaseModuleConfig) -> Self {
        Self {
            base,
            bindings: KnowledgeBindings::default(),
        }
    }
}

impl Default for KnowledgeBindings {
    fn default() -> Self {
        Self {
            description: Some(
                "Knowledge base module for document management and search".to_string(),
            ),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
            last_updated: Some(chrono::Utc::now().to_rfc3339()),
            document_count: Some(0),
            search_enabled: Some(true),
        }
    }
}
