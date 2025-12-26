use crate::module::{Module, ModuleManager};

impl ModuleManager {
    pub fn get_modules_by_type(&self, module_type: &str) -> Vec<&Module> {
        self.modules
            .iter()
            .filter(|m| m.config.module_type == module_type)
            .collect()
    }
}
