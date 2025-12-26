use crate::module::Module;

impl Clone for Module {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            path: self.path.clone(),
            cached_template: None,
            render_state: self.render_state.clone(),
        }
    }
}
