use crate::module::{ModuleHandler, ModuleManager};

impl ModuleManager {
    pub fn get_handler(&self, module_type: &str) -> Option<&(dyn ModuleHandler + 'static)> {
        self.handlers.get(module_type).map(|h| &**h)
    }
}
