use std::fs;

use crate::{log_debug, log_error, log_info};
use crate::module::config::ConfigValidator;

use super::OverseerHandler;

impl OverseerHandler {
    pub(in crate::module) fn update_module_config(
        &self,
        module_name: &str,
        updated_config: &serde_json::Value,
    ) -> color_eyre::Result<()> {
        let config_path = self.wasteland_path.join(module_name).join("config.yml");

        if !config_path.exists() {
            return Err(color_eyre::eyre::eyre!("Module not found"));
        }

        // Convert serde_json to serde_yaml first
        // This ensures proper type conversion between the two formats
        let yaml_value: serde_yaml::Value = serde_json::from_value(
            serde_json::to_value(updated_config)?
        )?;

        let yaml_str = serde_yaml::to_string(&yaml_value)?;

        log_debug!("YAML to validate:\n{}", yaml_str);

        // Validate before writing
        match ConfigValidator::validate(&yaml_str) {
            Ok(_) => {
                log_info!("Config validation passed for {}", module_name);
            }
            Err(e) => {
                log_error!("Config validation failed: {}", e);
                return Err(color_eyre::eyre::eyre!("Validation failed: {}", e));
            }
        }

        // Write to file
        fs::write(&config_path, yaml_str)?;

        log_info!("Updated config for module: {}", module_name);

        Ok(())
    }
}
