use crate::module::ModuleManager;

impl ModuleManager {
    pub fn update_module_bindings(&mut self, module_idx: usize) {
        if let Some(module) = self.modules.get(module_idx) {
            let module_type = module.config.module_type.clone();

            // For monitoring modules, use device_id for handler key (NOT module_idx!)
            // TODO again don't couple this to monitoring..
            let handler_key = if module_type == "monitoring" {
                let device_id = module.config.bindings
                    .get("device_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                format!("monitoring_{}", device_id)  // ← Must match registration! TODO scale this..
            } else {
                module_type.clone()
            };

            // Now we can safely get mutable references to both
            if let Some(handler) = self.handlers.get_mut(&handler_key) {
                if let Some(module) = self.modules.get_mut(module_idx) {
                    handler.update_bindings(module);
                }
            }
        }
    }
}
