mod chat_manager;
mod format_chat_history;
mod handle_key;
mod new;
mod submit_message;
mod trait_module_handler;
mod update_bindings;

use std::any::Any;

use crate::module::strategies::llm::handler::chat_manager::*;
use crate::{module::trait_module_handler::ModuleHandler, util::llm::LlmService};

/// Coordinates interaction with chat agent
#[derive(Debug)]
pub struct LlmHandler {
    chat_manager: ChatManager,
    llm_service: Option<LlmService>,
    session_id: String,
}
