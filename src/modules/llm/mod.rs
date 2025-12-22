// src/modules/llm/mod.rs
//! LLM Module - UI and handler for LLM interactions
//! Core LLM logic is now in util::llm

pub mod handler;
pub mod database;

pub use database::{LlmDatabase};
pub use handler::{LlmHandler};

use color_eyre::Result;
use crate::log_debug;
use crate::util::{
    database::Database,
    llm::{LlmService}
};
use crate::modules::ModuleManager;

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

