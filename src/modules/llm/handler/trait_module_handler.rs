use std::any::Any;
use crossterm::event::KeyCode;

use crate::{
    modules::{
        llm::LlmHandler,
        Module,
        module_handler::ModuleHandler,
    },
    util::io::event::AppEvent
};

impl ModuleHandler for LlmHandler {
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
        "llm"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
