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
    llm::{LlmService}
};
use crate::modules::ModuleManager;

/// Create LLM service if an LLM module is configured
pub async fn create_llm_service_if_available(
    module_manager: &ModuleManager,
    database: &Database,
) -> Result<Option<LlmService>> {
    let llm_modules = module_manager.get_modules_by_type("llm");

    if let Some(_llm_module) = llm_modules.first() {
        println!("Creating search-powered knowledge service");
        let service = LlmService::new(database.clone());
        Ok(Some(service))
    } else {
        Ok(None)
    }
}

