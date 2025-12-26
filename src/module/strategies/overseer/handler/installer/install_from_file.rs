use std::fs;
use std::path::Path;

use super::ModuleInstaller;

impl ModuleInstaller {
    pub(in crate::module) async fn install_from_file(
        &self,
        source_path: &Path,
        custom_name: Option<String>,
    ) -> color_eyre::Result<String> {
        let config_content = fs::read_to_string(source_path)?;
        let config: serde_yaml::Value = serde_yaml::from_str(&config_content)?;

        let original_name = config
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| color_eyre::eyre::eyre!("Invalid config: missing name"))?;

        let module_name = custom_name.unwrap_or_else(|| {
            source_path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or(original_name)
                .to_string()
        });

        let module_path = self.wasteland_path.join(&module_name);

        if module_path.exists() {
            return Err(color_eyre::eyre::eyre!("Module already exists"));
        }

        if let Some(source_dir) = source_path.parent() {
            self.copy_dir_recursive(source_dir, &module_path)?;
        }

        Ok(module_name)
    }
}
