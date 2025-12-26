use crate::module::{Module, ModuleManager};

impl ModuleManager {
    pub fn get_modules_mut(&mut self) -> &mut [Module] {
        &mut self.modules
    }
}
