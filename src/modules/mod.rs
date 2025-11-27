pub mod llm;
pub mod module_handler;
pub mod wasteland_manager;
pub mod valve_control;
pub mod monitoring;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use module_handler::ModuleHandler;
use std::fs;
use std::path::{Path, PathBuf};
use color_eyre::Result;
use ratatui::prelude::*;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    DefaultTerminal,
};
use std::time::{Duration, Instant};
use std::sync::Arc;

use crate::ui::template::{get_template, UiTemplate};
use crate::util::{
    database::Database,
    io::{
        get_all_event_message_topics,
        bus::{BusMessage, BusReceiver, BusSender, MessageBus},
        event::{AppEvent, Event, EventHandler},
        discovery::{DiscoveryManager}
    }
};
use crate::{log_info, log_error, log_debug, log_warn};

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
    pub path: PathBuf,
    pub cached_template: Option<Box<dyn UiTemplate>>,
    pub render_state: ModuleRenderState,
}

impl Module {
    pub fn load_from_path(path: &Path) -> Result<Self> {
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
    pub modules_path: PathBuf,
    pub namespace: String,
    pub selected_module: usize,
    event_receivers: Vec<BusReceiver>,
    handlers: HashMap<String, Box<dyn ModuleHandler>>,
}

impl ModuleManager {
    pub fn new(modules_path: PathBuf, namespace: String) -> Self {
        Self {
            modules: Vec::new(),
            modules_path,
            namespace,
            selected_module: 0,
            event_receivers: Vec::new(),
            handlers: HashMap::new(),
        }
    }

    pub async fn initialize_module_handlers(
        &mut self,
        wasteland_path: PathBuf,
        discovery_manager: Option<Arc<DiscoveryManager>>,
        database: &Database,
        message_bus: &MessageBus
    ) -> Result<()> {
        // Collect module info that needs handlers FIRST (avoid borrowing conflict)
        let modules_info: Vec<(String, String, String)> = self.modules
            .iter()
            .map(|m| {
                let device_id = m.config.bindings
                    .get("device_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                (m.config.module_type.clone(), device_id, m.config.bus_topic.clone())
            })
            .collect();

        // Now register handlers based on collected info
        for (module_type, device_id, bus_topic) in modules_info {
            match module_type.as_str() {
                "llm" => {
                    // Only register once
                    if !self.handlers.contains_key("llm") {
                        use crate::modules::llm;

                        let llm_service = llm::create_llm_service_if_available(
                            self,
                            database,
                        ).await.ok().flatten();

                        let llm_handler = Box::new(llm::handler::LlmHandler::new(llm_service));
                        self.register_handler(llm_handler);
                    }
                }
                "system" => {}
                "wasteland_manager" => {
                    if !self.handlers.contains_key("wasteland_manager") {
                        self.register_handler(Box::new(wasteland_manager::handler::WastelandManagerHandler::new(
                            wasteland_path.clone(),
                            discovery_manager.clone(),
                            database.clone(),
                            message_bus.clone()
                        )))
                    }
                }
                "valve_control" => {
                    if !self.handlers.contains_key("valve_control") && !device_id.is_empty() {
                        use crate::modules::valve_control;

                        log_info!("Registering valve_control handler for device: {}", device_id);

                        let handler = Box::new(
                            valve_control::handler::ValveControlHandler::new(
                                message_bus.clone(),
                                device_id.clone(),
                                bus_topic.clone(),
                            )
                        );
                        self.register_handler(handler);
                    }
                }
                "monitoring" => {
                    // Register one handler per monitoring module
                    // Each handler monitors its own device_id and bus_topic
                    let handler_key = format!("monitoring_{}", device_id);

                    if !self.handlers.contains_key(&handler_key) && !device_id.is_empty() {
                        use crate::modules::monitoring;

                        log_info!("🔧 Registering monitoring handler:");
                        log_info!("   - Handler key: {}", handler_key);
                        log_info!("   - Device ID: {}", device_id);
                        log_info!("   - Bus topic: {}", bus_topic);

                        let handler = Box::new(
                            monitoring::handler::MonitoringHandler::new(
                                message_bus.clone(),
                                device_id.clone(),
                                bus_topic.clone(),
                            )
                        );

                        self.handlers.insert(handler_key.clone(), handler);
                        log_info!("✅ Monitoring handler registered: {}", handler_key);
                    } else if device_id.is_empty() {
                        log_warn!("⚠️ Skipping monitoring module with empty device_id");
                    } else {
                        log_info!("ℹ️ Monitoring handler already exists: {}", handler_key);
                    }
                }
                _ => {}
            }
        }

        // Log all registered handlers for debugging
        log_info!("📋 All registered handlers:");
        for key in self.handlers.keys() {
            log_info!("   - {}", key);
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
        // First try direct module type lookup
        if let Some(handler) = self.handlers.get_mut(module_type) {
            return Some(&mut **handler);
        }

        // For monitoring modules, look up by device_id
        if module_type == "monitoring" {
            // This will be called with just "monitoring", but we need the device_id
            // The update_module_bindings method will handle this differently
            return None;
        }

        None
    }

    pub fn handle_key_for_module(&mut self, module_idx: usize, key_code: KeyCode) -> Option<AppEvent> {
        if let Some(module) = self.modules.get(module_idx) {
            let module_type = module.config.module_type.clone();

            // For monitoring modules, use device-specific handler key
            let handler_key = if module_type == "monitoring" {
                let device_id = module.config.bindings
                    .get("device_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                format!("monitoring_{}", device_id)
            } else {
                module_type.clone()
            };

            log_debug!("⌨️ Key event for module {} (handler: {})", module_idx, handler_key);

            // Now we can safely get mutable references to both
            let handler = self.handlers.get_mut(&handler_key)?;
            let module = self.modules.get_mut(module_idx)?;

            handler.handle_key(key_code, module)
        } else {
            None
        }
    }

    pub fn update_module_bindings(&mut self, module_idx: usize) {
        if let Some(module) = self.modules.get(module_idx) {
            let module_type = module.config.module_type.clone();

            // For monitoring modules, use device-specific handler key
            let handler_key = if module_type == "monitoring" {
                let device_id = module.config.bindings
                    .get("device_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                log_debug!("🔍 Looking up monitoring handler:");
                log_debug!("   - Module index: {}", module_idx);
                log_debug!("   - Module name: {}", module.config.name);
                log_debug!("   - Device ID: {}", device_id);
                log_debug!("   - Bus topic: {}", module.config.bus_topic);

                let key = format!("monitoring_{}", device_id);
                log_debug!("   - Handler key: {}", key);
                key
            } else {
                module_type.clone()
            };

            // Check if handler exists
            if self.handlers.contains_key(&handler_key) {
                log_debug!("✅ Handler found: {}", handler_key);
            } else {
                log_warn!("❌ Handler NOT found: {}", handler_key);
                log_warn!("📋 Available handlers:");
                for key in self.handlers.keys() {
                    log_warn!("   - {}", key);
                }
            }

            // Now we can safely get mutable references to both
            if let Some(handler) = self.handlers.get_mut(&handler_key) {
                if let Some(module) = self.modules.get_mut(module_idx) {
                    log_debug!("🔄 Calling update_bindings for {}", handler_key);
                    handler.update_bindings(module);
                }
            } else {
                log_error!("❌ Failed to get handler for {}", handler_key);
            }
        }
    }

    /// Subscribe the module manager to app events it cares about
    pub async fn subscribe_to_events(&mut self, message_bus: &MessageBus) {
        let topics = get_all_event_message_topics();

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
                            // println!("Loaded module: {}", module.config.name);
                            self.modules.push(module);
                        }
                        Err(e) => {
                            panic!("Failed to load module at {:?}: {}", path, e);
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

    pub fn is_displayable_module(module: &Module) -> bool {
        !module.config.template.is_empty()
    }

    pub fn get_displayable_modules(&self) -> Vec<&Module> {
        self.modules
            .iter()
            .filter(|m| Self::is_displayable_module(m))
            .collect()
    }

    pub fn get_displayable_modules_mut(&mut self) -> Vec<&mut Module> {
        self.modules
            .iter_mut()
            .filter(|m| Self::is_displayable_module(m))
            .collect()
    }

    fn get_displayable_indices(&self) -> Vec<usize> {
        self.modules
            .iter()
            .enumerate()
            .filter(|(_, m)| Self::is_displayable_module(m))
            .map(|(i, _)| i)
            .collect()
    }

    pub fn prev_module(&mut self) {
        let displayable_indices = self.get_displayable_indices();
        if displayable_indices.is_empty() {
            return;
        }

        if let Some(current_pos) = displayable_indices.iter().position(|&idx| idx == self.selected_module) {
            let new_pos = if current_pos == 0 {
                displayable_indices.len() - 1
            } else {
                current_pos - 1
            };
            self.selected_module = displayable_indices[new_pos];
        } else {
            self.selected_module = displayable_indices[0];
        }
    }

    pub fn next_module(&mut self) {
        let displayable_indices = self.get_displayable_indices();
        if displayable_indices.is_empty() {
            return;
        }

        if let Some(current_pos) = displayable_indices.iter().position(|&idx| idx == self.selected_module) {
            let new_pos = (current_pos + 1) % displayable_indices.len();
            self.selected_module = displayable_indices[new_pos];
        } else {
            self.selected_module = displayable_indices[0];
        }
    }

    pub fn select_current_module(&mut self) -> Option<&Module> {
        self.get_modules().get(self.selected_module)
    }

    pub async fn refresh_modules(&mut self) {
        if let Err(e) = self.discover_modules() {
            panic!("Failed to refresh modules: {}", e);
        }

        let module_count = self.get_modules().len();
        if self.selected_module >= module_count && module_count > 0 {
            self.selected_module = 0;
        }
    }
}
