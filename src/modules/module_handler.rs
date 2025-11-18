// src/modules/module_handler.rs

use color_eyre::Result;
use crate::event::AppEvent;
use crate::modules::Module;
use ratatui::crossterm::event::KeyCode;
use std::any::Any;
use std::fmt::Debug;

/// Trait for handling module-specific logic
pub trait ModuleHandler: Send + Sync + Debug {
    /// Handle key events for this module type
    fn handle_key(&mut self, key_code: KeyCode, module: &mut Module) -> Option<AppEvent>;

    /// Handle app events for this module type
    fn handle_event(&mut self, event: &AppEvent, module: &mut Module) -> Result<bool>;

    /// Update module bindings before render
    fn update_bindings(&mut self, module: &mut Module);

    /// Module type this handler supports
    fn module_type(&self) -> &str;

    /// Allow downcasting to concrete type
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
