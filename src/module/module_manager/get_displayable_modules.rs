use crate::module::{Module, ModuleManager};

impl ModuleManager {
    pub fn get_displayable_modules(&self) -> Vec<&Module> {
        self.modules
            .iter()
            .filter(|m| Self::is_displayable_module(m))
            .collect()
    }
}
