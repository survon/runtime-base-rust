pub mod llm;
pub mod module_handler;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use module_handler::ModuleHandler;
use std::fs;
use std::path::Path;
use color_eyre::Result;
use ratatui::prelude::*;
use ratatui::{
    buffer::{Buffer},
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};
use std::time::{Duration, Instant};

use crate::ui::template::{get_template, UiTemplate};
use crate::event::{AppEvent, Event, EventHandler};
use crate::bus::{MessageBus, BusMessage, BusReceiver, BusSender};
use crate::database::{Database};

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

            let blink_interval = Duration::from_millis(blink_interval_ms);

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
    pub namespace: String,
    pub selected_module: usize,
    event_receivers: Vec<BusReceiver>,
    handlers: HashMap<String, Box<dyn ModuleHandler>>,
}

impl ModuleManager {
    pub fn new(modules_path: std::path::PathBuf, namespace: String) -> Self {
        Self {
            modules: Vec::new(),
            modules_path,
            namespace,
            selected_module: 0,
            event_receivers: Vec::new(),
            handlers: HashMap::new(),
        }
    }

    pub async fn initialize_module_handlers(&mut self, database: &Database, bus_sender: BusSender) -> Result<()> {
        // Collect module types that need handlers FIRST (avoid borrowing conflict)
        let module_types: Vec<String> = self.modules
            .iter()
            .map(|m| m.config.module_type.clone())
            .collect();

        // Now register handlers based on collected types
        for module_type in module_types {
            match module_type.as_str() {
                "llm" => {
                    // Only register once
                    if !self.handlers.contains_key("llm") {
                        use crate::modules::llm;

                        // Try to create engine
                        let llm_engine = llm::create_llm_engine_if_available(
                            self,
                            database,
                            bus_sender.clone(),
                        ).await.ok().flatten();

                        let llm_handler = Box::new(llm::handler::LlmHandler::new(llm_engine));
                        self.register_handler(llm_handler);
                    }
                }
                // Add other module types here as needed
                _ => {}
            }
        }
        Ok(())
    }

    pub fn register_handler(&mut self, handler: Box<dyn ModuleHandler>) {
        let module_type = handler.module_type().to_string();
        self.handlers.insert(module_type, handler);
    }

    pub fn get_handler(&self, module_type: &str) -> Option<&(dyn ModuleHandler + 'static)> {
        self.handlers.get(module_type).map(|h| &**h)
    }

    pub fn get_handler_mut(&mut self, module_type: &str) -> Option<&mut (dyn ModuleHandler + 'static)> {
        self.handlers.get_mut(module_type).map(|h| &mut **h)
    }

    pub fn handle_key_for_module(&mut self, module_idx: usize, key_code: KeyCode) -> Option<AppEvent> {
        // Get the module type first (avoids split borrow issue)
        let module_type = self.modules.get(module_idx)?.config.module_type.clone();

        // Now we can safely get mutable references to both
        let handler = self.handlers.get_mut(&module_type)?;
        let module = self.modules.get_mut(module_idx)?;

        handler.handle_key(key_code, module)
    }

    pub fn update_module_bindings(&mut self, module_idx: usize) {
        // Get the module type first (avoids split borrow issue)
        if let Some(module_type) = self.modules.get(module_idx).map(|m| m.config.module_type.clone()) {
            // Now we can safely get mutable references to both
            if let Some(handler) = self.handlers.get_mut(&module_type) {
                if let Some(module) = self.modules.get_mut(module_idx) {
                    handler.update_bindings(module);
                }
            }
        }
    }

    /// Subscribe the module manager to app events it cares about
    pub async fn subscribe_to_events(&mut self, message_bus: &MessageBus) {
        let topics = vec![
            "app.event.increment",
            "app.event.decrement",
            "app.event.select",
            "app.event.refresh_modules",
        ];

        for topic in topics {
            let receiver = message_bus.subscribe(topic.to_string()).await;
            self.event_receivers.push(receiver);
        }
    }

    /// Poll for incoming events and handle them
    pub fn poll_events(&mut self) {
        // Collect messages first, then process them
        let mut messages = Vec::new();

        for receiver in &mut self.event_receivers {
            while let Ok(message) = receiver.try_recv() {
                messages.push(message);
            }
        }

        // Now process all collected messages
        for message in messages {
            self.handle_event_message(&message);
        }
    }

    fn handle_event_message(&mut self, message: &BusMessage) {
        match message.topic.strip_prefix("app.event.") {
            Some("increment") => {
                self.next_module();
            }
            Some("decrement") => {
                self.prev_module();
            }
            Some("refresh_modules") => {
                // Modules could reload their config, etc.
            }
            _ => {}
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
