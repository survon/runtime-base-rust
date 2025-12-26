use crate::module::ModuleManager;

impl ModuleManager {
    pub(super) fn get_displayable_indices(&self) -> Vec<usize> {
        self.modules
            .iter()
            .enumerate()
            .filter(|(_, m)| Self::is_displayable_module(m))
            .map(|(i, _)| i)
            .collect()
    }
}
