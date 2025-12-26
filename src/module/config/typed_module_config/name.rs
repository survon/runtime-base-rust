use super::TypedModuleConfig;

impl TypedModuleConfig {
    /// Get module name
    pub fn name(&self) -> Option<&str> {
        self.base().map(|b| b.name.as_str())
    }
}
