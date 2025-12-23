use std::fs;

use super::ModuleInstaller;

impl ModuleInstaller {
    pub(super) async fn install_from_registry(
        &self,
        manifest_id: &str,
        custom_name: Option<String>,
    ) -> color_eyre::Result<String> {
        let manifests = self.list_registry_manifests().await?;
        let manifest = manifests
            .iter()
            .find(|m| m.id == manifest_id)
            .ok_or_else(|| color_eyre::eyre::eyre!("Module not found in registry"))?;

        // Create manifest directory
        let manifest_name = custom_name.unwrap_or_else(|| manifest.id.clone());
        let manifest_path = self.wasteland_path.join(&manifest_name);

        if manifest_path.exists() {
            return Err(color_eyre::eyre::eyre!("Module already exists"));
        }

        fs::create_dir_all(&manifest_path)?;

        // Generate config.yml based on registry template
        self.create_module_config(
            &manifest_path,
            &manifest.name,
            &manifest.module_type,
            &manifest.template,
            None,
        )?;

        Ok(manifest_name)
    }
}
