pub mod config;
pub mod strategies;
pub mod trait_module_handler;

mod get_template;
mod get_view_type;
mod has_knowledge_dir;
mod load_from_manifest_path;
mod module_manager;
mod render_detail;
mod render_overview_cta;
mod render_state;
mod trait_clone;

use ratatui::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub use config::*;
pub use module_manager::{ModuleManager, ModuleManagerView};
pub use render_state::ModuleRenderState;
pub use trait_module_handler::ModuleHandler;

use crate::ui::template::UiTemplate;

#[derive(Debug)]
pub struct Module {
    pub config: ModuleConfig,
    pub path: PathBuf,
    pub cached_template: Option<Box<dyn UiTemplate>>,
    pub render_state: ModuleRenderState,
}
