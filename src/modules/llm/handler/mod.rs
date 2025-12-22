mod chat_manager;
mod new;
mod submit_message;
mod format_chat_history;
mod trait_module_handler;
mod handle_key;
mod update_bindings;

use std::any::Any;

use crate::{
    modules::{
        module_handler::ModuleHandler,
        llm::{
            handler::{chat_manager::*},
        }
    },
    util::{
        llm::LlmService,
    },
};


/// Coordinates interaction with chat agent
#[derive(Debug)]
pub struct LlmHandler {
    chat_manager: ChatManager,
    llm_service: Option<LlmService>,
    session_id: String,
}

