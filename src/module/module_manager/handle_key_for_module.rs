use crossterm::event::KeyCode;

use crate::{log_debug, log_error, module::ModuleManager, util::io::event::AppEvent};

impl ModuleManager {
    pub fn handle_key_for_module(
        &mut self,
        module_idx: usize,
        key_code: KeyCode,
    ) -> Option<AppEvent> {
        if let Some(module) = self.modules.get(module_idx) {
            let module_type = module.config.module_type.clone();

            // For monitoring modules, use device_id for handler key (NOT module_idx!)
            let handler_key = if module_type == "monitoring" {
                let device_id = module
                    .config
                    .bindings
                    .get("device_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                format!("monitoring_{}", device_id) // ← Must match registration! TODO scale this
            } else {
                module_type.clone()
            };

            log_debug!(
                "🔑 Looking up handler: '{}' for module at index {}",
                handler_key,
                module_idx
            );

            // Now we can safely get mutable references to both
            let handler = self.handlers.get_mut(&handler_key)?;
            let module = self.modules.get_mut(module_idx)?;

            log_debug!("✓ Found handler, calling handle_key with {:?}", key_code);

            let result = handler.handle_key(key_code, module);

            if result.is_some() {
                log_debug!("✓ Handler returned event: {:?}", result);
            } else {
                log_debug!("Handler returned None");
            }

            result
        } else {
            log_error!("❌ Module at index {} not found!", module_idx);
            None
        }
    }
}
