use crate::module::Module;

impl Module {
    pub fn get_view_type(&self) -> &str {
        self.config.view_type.as_deref().unwrap_or("default")
    }
}
