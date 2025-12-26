use crate::module::{Module, ModuleManager};

impl ModuleManager {
    pub fn select_current_module(&mut self) -> Option<&Module> {
        self.get_modules().get(self.selected_module)
    }
}
