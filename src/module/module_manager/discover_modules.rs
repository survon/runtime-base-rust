use std::fs;

use crate::module::{Module, ModuleManager};

impl ModuleManager {
    pub fn discover_modules(&mut self) -> color_eyre::Result<()> {
        self.modules.clear();

        if !self.manifests_path.exists() {
            fs::create_dir_all(&self.manifests_path)?;
            return Ok(());
        }

        for entry in fs::read_dir(&self.manifests_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let config_path = path.join("config.yml");
                if config_path.exists() {
                    match Module::load_from_manifest_path(&path) {
                        Ok(module) => {
                            self.modules.push(module);
                        }
                        Err(e) => {
                            panic!("Failed to load manifest at {:?}: {}", path, e);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
