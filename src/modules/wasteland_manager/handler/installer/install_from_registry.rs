use std::fs;

use super::ModuleInstaller;

impl ModuleInstaller {
    pub(super) async fn install_from_registry(
        &self,
        module_id: &str,
        custom_name: Option<String>,
    ) -> color_eyre::Result<String> {
        let modules = self.list_registry_modules().await?;
        let module = modules
            .iter()
            .find(|m| m.id == module_id)
            .ok_or_else(|| color_eyre::eyre::eyre!("Module not found in registry"))?;

        // Create module directory
        let module_name = custom_name.unwrap_or_else(|| module.id.clone());
        let module_path = self.wasteland_path.join(&module_name);

        if module_path.exists() {
            return Err(color_eyre::eyre::eyre!("Module already exists"));
        }

        fs::create_dir_all(&module_path)?;

        // Generate config.yml based on registry template
        self.create_module_config(
            &module_path,
            &module.name,
            &module.module_type,
            &module.template,
            None,
        )?;

        Ok(module_name)
    }
}
