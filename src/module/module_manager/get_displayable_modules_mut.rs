use crate::module::{Module, ModuleManager};

impl ModuleManager {
    pub fn get_displayable_modules_mut(&mut self) -> Vec<&mut Module> {
        self.modules
            .iter_mut()
            .filter(|m| Self::is_displayable_module(m))
            .collect()
    }
}
