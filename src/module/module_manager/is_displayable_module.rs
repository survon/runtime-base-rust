use crate::module::{Module, ModuleManager};

impl ModuleManager {
    pub fn is_displayable_module(module: &Module) -> bool {
        !module.config.template.is_empty()
    }
}
