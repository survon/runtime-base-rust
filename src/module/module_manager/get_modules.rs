use crate::module::{Module, ModuleManager};

impl ModuleManager {
    pub fn get_modules(&self) -> &[Module] {
        &self.modules
    }
}
