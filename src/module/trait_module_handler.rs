// src/modules/module_handler.rs

use color_eyre::Result;
use ratatui::crossterm::event::KeyCode;
use std::{
    any::Any,
    fmt::Debug,
};

use crate::{
    util::io::event::AppEvent,
    module::Module
};

/// Trait for handling module-specific logic
pub trait ModuleHandler: Send + Sync + Debug {
    fn handle_key(&mut self, key_code: KeyCode, module: &mut Module) -> Option<AppEvent>;
    fn handle_event(&mut self, event: &AppEvent, module: &mut Module) -> Result<bool>;
    fn update_bindings(&mut self, module: &mut Module);
    fn module_type(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
