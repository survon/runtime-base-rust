use std::any::Any;
use crossterm::event::KeyCode;

use crate::{
    modules::{
        Module,
        module_handler::ModuleHandler,
        valve_control::handler::ValveControlHandler,
    },
    util::io::event::AppEvent,
};

impl ModuleHandler for ValveControlHandler {
    fn handle_key(&mut self, key_code: KeyCode, _module: &mut Module) -> Option<AppEvent> {
        self._handle_key(key_code, _module)
    }

    fn handle_event(&mut self, _event: &AppEvent, _module: &mut Module) -> color_eyre::Result<bool> {
        Ok(false)
    }

    fn update_bindings(&mut self, module: &mut Module) {
        self._update_bindings(module)
    }

    fn module_type(&self) -> &str {
        "valve_control"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
