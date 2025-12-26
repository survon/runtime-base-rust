mod new;
mod initialize_module_handlers;
mod register_handler;
mod get_handler;
mod get_handler_mut;
mod handle_key_for_module;
mod update_module_bindings;
mod subscribe_to_events;
mod poll_events;
mod handle_event_message;
mod has_active_blinks;
mod discover_modules;
mod get_modules;
mod get_modules_mut;
mod get_modules_by_type;
mod get_knowledge_modules;
mod is_displayable_module;
mod get_displayable_modules;
mod get_displayable_modules_mut;
mod get_displayable_indices;
mod prev_module;
mod next_module;
mod select_current_module;
mod refresh_modules;

use std::collections::HashMap;
use std::path::PathBuf;

use crate::{
    module::{
        Module,
        trait_module_handler::ModuleHandler
    },
    util::io::bus::BusReceiver,
};
use crate::app::ModuleSource;

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
