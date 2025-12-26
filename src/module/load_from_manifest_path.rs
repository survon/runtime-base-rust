use std::{
    fs,
    path::Path,
};

use super::{Module, ModuleConfig, ModuleRenderState};

impl Module {
    pub fn load_from_manifest_path(path: &Path) -> color_eyre::Result<Self> {
        let config_path = path.join("config.yml");
        let config_content = fs::read_to_string(&config_path)?;
        let config: ModuleConfig = serde_yaml::from_str(&config_content)?;

        Ok(Module {
            config,
            path: path.to_path_buf(),
            cached_template: None,
            render_state: ModuleRenderState::default(),
        })
    }
}
