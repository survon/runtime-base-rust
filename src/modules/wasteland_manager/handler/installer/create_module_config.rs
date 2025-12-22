use std::fs;
use std::path::Path;

use super::ModuleInstaller;

impl ModuleInstaller {
    pub(super) fn create_module_config(
        &self,
        module_path: &Path,
        name: &str,
        module_type: &str,
        template: &str,
        bindings: Option<serde_yaml::Mapping>,
    ) -> color_eyre::Result<()> {
        let config_path = module_path.join("config.yml");

        let mut config = serde_yaml::Mapping::new();
        config.insert(
            serde_yaml::Value::String("name".to_string()),
            serde_yaml::Value::String(name.to_string()),
        );
        config.insert(
            serde_yaml::Value::String("module_type".to_string()),
            serde_yaml::Value::String(module_type.to_string()),
        );
        config.insert(
            serde_yaml::Value::String("bus_topic".to_string()),
            serde_yaml::Value::String(name.to_lowercase().replace(" ", "_")),
        );
        config.insert(
            serde_yaml::Value::String("template".to_string()),
            serde_yaml::Value::String(template.to_string()),
        );

        let default_bindings = bindings.unwrap_or_else(|| {
            let mut b = serde_yaml::Mapping::new();
            b.insert(
                serde_yaml::Value::String("is_blinkable".to_string()),
                serde_yaml::Value::Bool(true),
            );
            b
        });

        config.insert(
            serde_yaml::Value::String("bindings".to_string()),
            serde_yaml::Value::Mapping(default_bindings),
        );

        let yaml_content = serde_yaml::to_string(&config)?;
        fs::write(config_path, yaml_content)?;

        Ok(())
    }
}
