use crate::module::{Module, ModuleManager};

impl ModuleManager {
    pub fn get_knowledge_modules(&self) -> Vec<&Module> {
        self.get_modules_by_type("knowledge")
    }
}
