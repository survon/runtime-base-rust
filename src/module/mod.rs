pub mod trait_module_handler;
pub mod strategies;
pub mod config;

mod render_state;
mod module_manager;
mod trait_clone;
mod load_from_manifest_path;
mod get_template;
mod render_overview_cta;
mod render_detail;
mod has_knowledge_dir;
mod get_view_type;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use ratatui::prelude::*;

pub use config::*;
pub use render_state::ModuleRenderState;
pub use module_manager::{ModuleManager, ModuleManagerView};
pub use trait_module_handler::ModuleHandler;

use crate::ui::template::UiTemplate;

#[derive(Debug)]
pub struct Module {
    pub config: ModuleConfig,
    pub path: PathBuf,
    pub cached_template: Option<Box<dyn UiTemplate>>,
    pub render_state: ModuleRenderState,
}

