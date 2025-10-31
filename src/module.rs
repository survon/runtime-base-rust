use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use color_eyre::Result;
use crate::app::AppMode;
use crate::llm::create_llm_engine_if_available;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    pub name: String,
    pub module_type: String,
    pub bus_topic: String,

    // Optional fields for different module types
    pub ports: Option<Vec<String>>,
    pub messages: Option<Vec<String>>,
    pub game_type: Option<String>,
    pub model: Option<String>,
    pub view_type: Option<String>,
    pub thresholds: Option<HashMap<String, f64>>,
    pub rules: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub config: ModuleConfig,
    pub path: std::path::PathBuf,
}

impl Module {
    pub fn load_from_path(path: &Path) -> Result<Self> {
        let config_path = path.join("config.yml");
        let config_content = fs::read_to_string(&config_path)?;
        let config: ModuleConfig = serde_yaml::from_str(&config_content)?;
        println!("LOADED CONFIG: {:?}", config);  // ← ADD THIS
        Ok(Module {
            config,
            path: path.to_path_buf(),
        })
    }

    pub fn has_knowledge_dir(&self) -> bool {
        self.path.join("knowledge").exists()
    }

    pub fn get_view_type(&self) -> &str {
        self.config.view_type.as_deref().unwrap_or("default")
    }
}

#[derive(Debug)]
pub struct ModuleManager {
    modules: Vec<Module>,
    modules_path: std::path::PathBuf,
    pub selected_module: usize,
}

impl ModuleManager {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            modules_path: std::path::PathBuf::from("./wasteland/modules"),
            selected_module: 0,
        }
    }

    pub fn discover_modules(&mut self) -> Result<()> {
        self.modules.clear();

        if !self.modules_path.exists() {
            fs::create_dir_all(&self.modules_path)?;
            return Ok(());
        }

        for entry in fs::read_dir(&self.modules_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let config_path = path.join("config.yml");
                if config_path.exists() {
                    match Module::load_from_path(&path) {
                        Ok(module) => {
                            println!("Loaded module: {}", module.config.name);
                            self.modules.push(module);
                        }
                        Err(e) => {
                            eprintln!("Failed to load module at {:?}: {}", path, e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn get_modules(&self) -> &[Module] {
        &self.modules
    }

    pub fn get_modules_by_type(&self, module_type: &str) -> Vec<&Module> {
        self.modules
            .iter()
            .filter(|m| m.config.module_type == module_type)
            .collect()
    }

    pub fn get_knowledge_modules(&self) -> Vec<&Module> {
        self.get_modules_by_type("knowledge")
    }


    pub fn next_module(&mut self) {
        let module_count = self.get_modules().len();
        if module_count > 0 {
            self.selected_module = (self.selected_module + 1) % module_count;
        }
    }

    pub fn prev_module(&mut self) {
        let module_count = self.get_modules().len();
        if module_count > 0 {
            self.selected_module = if self.selected_module == 0 {
                module_count - 1
            } else {
                self.selected_module - 1
            };
        }
    }

    pub fn select_current_module(&mut self) -> Option<&Module> {
        self.get_modules().get(self.selected_module)
    }

    pub async fn refresh_modules(&mut self) {
        if let Err(e) = self.discover_modules() {
            eprintln!("Failed to refresh modules: {}", e);
        }

        // Reset selection if it's out of bounds
        let module_count = self.get_modules().len();
        if self.selected_module >= module_count && module_count > 0 {
            self.selected_module = 0;
        }
    }
}
