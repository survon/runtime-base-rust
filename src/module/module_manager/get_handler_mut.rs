use crate::module::{ModuleHandler, ModuleManager};

impl ModuleManager {
    pub fn get_handler_mut(&mut self, module_type: &str) -> Option<&mut (dyn ModuleHandler + 'static)> {
        // First try direct module type lookup
        if let Some(handler) = self.handlers.get_mut(module_type) {
            return Some(&mut **handler);
        }

        // For monitoring modules, look up by device_id
        // TODO make this more elegant..
        if module_type == "monitoring" {
            // This will be called with just "monitoring", but we need the device_id
            // The update_module_bindings method will handle this differently
            return None;
        }

        None
    }
}
