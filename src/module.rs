use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use color_eyre::Result;
use ratatui::prelude::*;
use ratatui::buffer::Buffer;
use std::time::{Duration, Instant};
use crate::ui::template::{get_template, UiTemplate};

/// Runtime rendering state for modules (not serialized)
#[derive(Debug, Clone)]
pub struct ModuleRenderState {
    pub blink_state: bool,
    pub last_blink: Instant,
    pub animation_frame: usize,
    pub is_focused: bool,
    pub is_actively_blinking: bool,
}

impl ModuleRenderState {
    pub fn update_blink(&mut self, interval: Duration) -> bool {
        if self.last_blink.elapsed() >= interval {
            self.blink_state = !self.blink_state;
            self.last_blink = Instant::now();
            true
        } else {
            false
        }
    }

    pub fn start_blinking(&mut self) {
        if !self.is_actively_blinking {
            self.is_actively_blinking = true;
            self.last_blink = Instant::now();
        }
    }

    pub fn stop_blinking(&mut self) {
        if self.is_actively_blinking {
            self.is_actively_blinking = false;
            self.blink_state = false; // Reset to normal state
        }
    }
}

impl Default for ModuleRenderState {
    fn default() -> Self {
        Self {
            blink_state: false,
            last_blink: Instant::now(),
            animation_frame: 0,
            is_focused: false,
            is_actively_blinking: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    pub name: String,
    pub module_type: String,
    pub bus_topic: String,
    pub template: String,
    pub bindings: HashMap<String, serde_json::Value>,

    // Optional fields for different module types
    pub ports: Option<Vec<String>>,
    pub messages: Option<Vec<String>>,
    pub game_type: Option<String>,
    pub model: Option<String>,
    pub view_type: Option<String>,
    pub thresholds: Option<HashMap<String, f64>>,
    pub rules: Option<HashMap<String, String>>,
}

impl ModuleConfig {
    pub fn is_blinkable(&self) -> bool {
        self.bindings.get("is_blinkable").and_then(|v| v.as_bool()) == Some(true)
    }
}

#[derive(Debug)]
pub struct Module {
    pub config: ModuleConfig,
    pub path: std::path::PathBuf,
    pub cached_template: Option<Box<dyn UiTemplate>>,
    pub render_state: ModuleRenderState,
}

impl Module {
    pub fn load_from_path(path: &Path) -> Result<Self> {
        let config_path = path.join("config.yml");
        let config_content = fs::read_to_string(&config_path)?;
        let config: ModuleConfig = serde_yaml::from_str(&config_content)?;
        println!("LOADED CONFIG: {:?}", config);

        Ok(Module {
            config,
            path: path.to_path_buf(),
            cached_template: None,
            render_state: ModuleRenderState::default(),
        })
    }

    pub fn get_template(&mut self) -> std::result::Result<&Box<dyn UiTemplate>, String> {
        if self.cached_template.is_none() {
            let template = get_template(&self.config.template)
                .ok_or_else(|| format!("Unknown template: {}", self.config.template))?;

            for binding in template.required_bindings() {
                if !self.config.bindings.contains_key(*binding) {
                    return Err(format!(
                        "Module '{}' missing required binding '{}' for template '{}'",
                        self.config.name, binding, self.config.template
                    ));
                }
            }

            self.cached_template = Some(template);
        }

        Ok(self.cached_template.as_ref().unwrap())
    }

    pub fn render(&mut self, is_selected: bool, area: Rect, buf: &mut Buffer) -> std::result::Result<(), String> {
        self.get_template()?;

        if self.config.is_blinkable() {
            // Get blink interval from bindings (default 500ms)
            let blink_interval_ms = self.config.bindings
                .get("blink_interval_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(500);

            let blink_interval = std::time::Duration::from_millis(blink_interval_ms);

            if self.render_state.last_blink.elapsed() >= blink_interval {
                self.render_state.blink_state = !self.render_state.blink_state;
                self.render_state.last_blink = Instant::now();

            }
        }

        let mut template = self.cached_template.take()
            .ok_or_else(|| "Template not loaded".to_string())?;

        // Now we can call render with &mut self since cached_template is None
        template.render(is_selected, area, buf, self);

        // Put the template back
        self.cached_template = Some(template);

        Ok(())
    }

    pub fn has_knowledge_dir(&self) -> bool {
        self.path.join("knowledge").exists()
    }

    pub fn get_view_type(&self) -> &str {
        self.config.view_type.as_deref().unwrap_or("default")
    }
}

impl Clone for Module {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            path: self.path.clone(),
            cached_template: None,
            render_state: self.render_state.clone(),
        }
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

    pub fn has_active_blinks(&self) -> bool {
        self.modules
            .iter()
            .any(|m| m.render_state.is_actively_blinking)
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

    pub fn get_modules_mut(&mut self) -> &mut [Module] {
        &mut self.modules
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

        let module_count = self.get_modules().len();
        if self.selected_module >= module_count && module_count > 0 {
            self.selected_module = 0;
        }
    }
}
