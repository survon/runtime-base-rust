// src/modules/llm/mod.rs
//! LLM Module - UI and handler for LLM interactions
//! Core LLM logic is now in util::llm

pub mod handler;
pub mod database;
pub use database::{ChatMessage, KnowledgeChunk, LlmDatabase};
pub use handler::{LlmHandler, ChatManager};

use color_eyre::Result;
use crate::util::{
    database::Database,
    llm::{LlmService, LlmStrategyType}
};
use crate::modules::ModuleManager;

/// Create LLM service if an LLM module is configured
pub async fn create_llm_service_if_available(
    module_manager: &ModuleManager,
    database: &Database,
) -> Result<Option<LlmService>> {
    let llm_modules = module_manager.get_modules_by_type("llm");

    if let Some(llm_module) = llm_modules.first() {
        let model_name = llm_module.config.model.as_deref().unwrap_or("knowledge");

        println!("Creating LLM service with model: {}", model_name);

        let service = LlmService::from_model_name(database.clone(), model_name).await?;
        Ok(Some(service))
    } else {
        println!("No LLM modules found - LLM chat will not be available");
        Ok(None)
    }
}
