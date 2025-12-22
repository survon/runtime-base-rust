// src/modules/module_handler.rs

use color_eyre::Result;
use crate::util::io::event::AppEvent;
use crate::modules::Module;
use ratatui::crossterm::event::KeyCode;
use std::any::Any;
use std::fmt::Debug;

/// Trait for handling module-specific logic
pub trait ModuleHandler: Send + Sync + Debug {
    fn handle_key(&mut self, key_code: KeyCode, module: &mut Module) -> Option<AppEvent>;
    fn handle_event(&mut self, event: &AppEvent, module: &mut Module) -> Result<bool>;
    fn update_bindings(&mut self, module: &mut Module);
    fn module_type(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
