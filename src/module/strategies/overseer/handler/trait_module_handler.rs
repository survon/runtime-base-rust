use std::any::Any;
use crossterm::event::KeyCode;

use crate::module::{
    trait_module_handler::ModuleHandler,
    Module,
};
use crate::module::strategies::overseer::handler::OverseerHandler;
use crate::util::io::event::AppEvent;

impl ModuleHandler for OverseerHandler {
    fn handle_key(&mut self, key_code: KeyCode, _module: &mut Module) -> Option<AppEvent> {
        self._handle_key(key_code, _module)
    }

    fn handle_event(&mut self, _event: &AppEvent, _module: &mut Module) -> color_eyre::Result<bool> {
        Ok(false)
    }

    fn update_bindings(&mut self, module: &mut Module) {
        self._update_bindings(module);
    }

    fn module_type(&self) -> &str {
        "overseer"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
