mod discover_modules;
mod get_displayable_indices;
mod get_displayable_modules;
mod get_displayable_modules_mut;
mod get_handler;
mod get_handler_mut;
mod get_knowledge_modules;
mod get_modules;
mod get_modules_by_type;
mod get_modules_mut;
mod handle_event_message;
mod handle_key_for_module;
mod has_active_blinks;
mod initialize_module_handlers;
mod is_displayable_module;
mod new;
mod next_module;
mod poll_events;
mod prev_module;
mod refresh_modules;
mod register_handler;
mod select_current_module;
mod subscribe_to_events;
mod update_module_bindings;

use std::collections::HashMap;
use std::path::PathBuf;

use crate::app::ModuleSource;
use crate::{
    module::{trait_module_handler::ModuleHandler, Module},
    util::io::bus::BusReceiver,
};

#[derive(Debug, PartialEq)]
pub enum ModuleManagerView {
    ModuleListView,
    ModuleDetail(ModuleSource, usize),
}

#[derive(Debug)]
pub struct ModuleManager {
    modules: Vec<Module>,
    pub manifests_path: PathBuf,
    pub namespace: String,
    pub selected_module: usize,
    pub current_view: ModuleManagerView,
    event_receivers: Vec<BusReceiver>,
    handlers: HashMap<String, Box<dyn ModuleHandler>>,
}
