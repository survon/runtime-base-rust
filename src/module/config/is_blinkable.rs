use crate::module::ModuleConfig;

impl ModuleConfig {
    pub fn is_blinkable(&self) -> bool {
        self.bindings.get("is_blinkable").and_then(|v| v.as_bool()) == Some(true)
    }
}
