use std::any::Any;
use crossterm::event::KeyCode;

use crate::{
    log_info, log_warn,
    modules::{
        Module,
        module_handler::ModuleHandler,
        monitoring::handler::MonitoringHandler,
    },
    util::io::event::AppEvent,
};

impl ModuleHandler for MonitoringHandler {
    fn handle_key(&mut self, key_code: KeyCode, _module: &mut Module) -> Option<AppEvent> {
        match key_code {
            KeyCode::Char('r') => {
                log_info!("Manual refresh requested for {}", self.device_id);
                None
            }
            _ => None,
        }
    }

    fn handle_event(&mut self, _event: &AppEvent, _module: &mut Module) -> color_eyre::Result<bool> {
        Ok(false)
    }

    fn update_bindings(&mut self, module: &mut Module) {
        self._update_bindings(module)
    }

    fn module_type(&self) -> &str {
        "monitoring"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
