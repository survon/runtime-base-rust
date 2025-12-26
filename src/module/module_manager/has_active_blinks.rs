use crate::module::ModuleManager;

impl ModuleManager {
    pub fn has_active_blinks(&self) -> bool {
        self.modules
            .iter()
            .any(|m| m.render_state.is_actively_blinking)
    }
}
