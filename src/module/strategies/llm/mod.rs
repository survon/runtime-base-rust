// src/modules/llm/mod.rs
//! LLM Module - UI and handler for LLM interactions
//! Core LLM logic is now in util::llm

pub mod handler;
pub mod database;
mod validation;

pub use database::{LlmDatabase};
pub use handler::{LlmHandler};

use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::{
    log_debug,
    util::{
        database::Database,
        llm::{LlmService}
    },
    module::{BaseModuleConfig, ModuleManager, ServiceDiscoveryConfig},
};

/// LLM module (chat interfaces)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    #[serde(flatten)]
    pub base: BaseModuleConfig,
    pub model: String, // "search", "summarizer", "council"
    pub bindings: LlmBindings,

    #[serde(default)]
    pub service_discovery: Option<ServiceDiscoveryConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmBindings {
    pub model_info: String,
    pub chat_history: Vec<serde_json::Value>,
    pub chat_input: String,
    pub scroll_offset: i32,

    // Council-specific
    #[serde(default)]
    pub available_advisors: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub active_advisor: Option<serde_json::Value>,
    #[serde(default)]
    pub current_link_index: Option<i32>,
}

/// Create LLM service if an LLM module is configured
pub async fn create_llm_service_if_available(
    module_manager: &ModuleManager,
    database: &Database,
) -> Result<Option<LlmService>> {
    let llm_modules = module_manager.get_modules_by_type("llm");

    if let Some(llm_module) = llm_modules.first() {
        let model = llm_module.config.bindings
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("search");

        match model {
            "council" => {
                // Council mode - service will be created per-advisor dynamically
                log_debug!("Council mode: services created on-demand");
                Ok(Some(LlmService::new(database.clone())))
            },
            _ => {
                // Existing local search/summarizer mode
                log_debug!("Creating search-powered knowledge service");
                let service = LlmService::new(database.clone());
                Ok(Some(service))
            }
        }
    } else {
        Ok(None)
    }
}

