use std::fs;

use crate::modules::overseer::config_editor::ConfigEditor;
use super::{OverseerHandler, WastelandView};

impl OverseerHandler {
    pub(super) fn handle_manage_modules_enter(&mut self) {
        if self.selected_index < self.installed_modules.len() {
            let module_name = &self.installed_modules[self.selected_index];

            // Load the module's config
            let config_path = self.wasteland_path.join(module_name).join("config.yml");

            match fs::read_to_string(&config_path) {
                Ok(yaml) => {
                    // Parse to get full config AND bindings
                    match serde_yaml::from_str::<serde_json::Value>(&yaml) {
                        Ok(config) => {
                            let module_type = config
                                .get("module_type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string();

                            let bindings = config
                                .get("bindings")
                                .and_then(|v| serde_json::from_value(v.clone()).ok())
                                .unwrap_or_default();

                            // Create editor with FULL config
                            self.config_editor = Some(ConfigEditor::from_manifest(
                                module_name.clone(),
                                module_type,
                                &config,
                                &bindings,
                            ));

                            self.current_view = WastelandView::EditConfig;
                        }
                        Err(e) => {
                            self.status_message = Some(format!("Failed to parse config: {}", e));
                        }
                    }
                }
                Err(e) => {
                    self.status_message = Some(format!("Failed to load config: {}", e));
                }
            }
        }
    }
}
