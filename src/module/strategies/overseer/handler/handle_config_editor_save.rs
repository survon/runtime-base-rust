use std::fs;

use crate::{log_debug, log_error, log_info};
use super::{OverseerHandler, WastelandView};

impl OverseerHandler {
    pub(in crate::module) fn handle_config_editor_save(&mut self) {
        if let Some(editor) = &self.config_editor {
            let module_name = editor.module_name.clone();
            let config_path = self.wasteland_path.join(&module_name).join("config.yml");

            log_info!("Saving config for module: {}", module_name);

            // Read the original config
            match fs::read_to_string(&config_path) {
                Ok(yaml) => {
                    match serde_yaml::from_str::<serde_json::Value>(&yaml) {
                        Ok(original_config) => {
                            // Get the updated config from editor
                            let updated_config = editor.to_full_config(&original_config);

                            log_debug!("Updated config: {:?}", updated_config);

                            // Use the update_module_config method which validates and writes
                            match self.update_module_config(&module_name, &updated_config) {
                                Ok(_) => {
                                    self.status_message = Some(format!("✓ Saved {}", module_name));
                                    log_info!("Successfully saved config for {}", module_name);

                                    // Close editor and return to modules view
                                    self.config_editor = None;
                                    self.current_view = WastelandView::ManageModules;

                                    // Trigger module refresh via message bus
                                    self.trigger_module_refresh();
                                }
                                Err(e) => {
                                    self.status_message = Some(format!("❌ Save failed: {}", e));
                                    log_error!("Failed to save config: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            self.status_message =
                                Some(format!("Failed to parse original config: {}", e));
                            log_error!("Parse error: {}", e);
                        }
                    }
                }
                Err(e) => {
                    self.status_message = Some(format!("Failed to read config: {}", e));
                    log_error!("Read error: {}", e);
                }
            }
        }
    }
}
