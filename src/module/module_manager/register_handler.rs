use crate::module::{
    ModuleManager,
    ModuleHandler,
};

impl ModuleManager {
    pub fn register_handler(&mut self, handler: Box<dyn ModuleHandler>) {
        let module_type = handler.module_type().to_string();
        self.handlers.insert(module_type, handler);
    }
}
