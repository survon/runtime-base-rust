use crate::module::trait_module_handler::ModuleHandler;
use crate::util::io::bus::BusMessage;
use crate::util::io::event::AppEvent;
use crate::module::config::base_module_config::BaseModuleConfig;
use crate::module::render_state::trait_default::RenderDefault;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilAdvisor {
    pub name: String,
    pub title: String,
    pub position: String,
    pub endpoint: String,
    pub model: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilModuleState {
    pub available_advisors: Vec<CouncilAdvisor>,
    pub active_advisor: Option<CouncilAdvisor>,
    pub advisor_status: HashMap<String, String>,
    pub last_discovery: Option<std::time::Instant>,
}

pub struct CouncilModuleHandler {
    state: CouncilModuleState,
    config: BaseModuleConfig,
}

impl CouncilModuleHandler {
    pub fn new(config: BaseModuleConfig) -> Self {
        Self {
            state: CouncilModuleState {
                available_advisors: Vec::new(),
                active_advisor: None,
                advisor_status: HashMap::new(),
                last_discovery: None,
            },
            config,
        }
    }

    async fn discover_advisors(&mut self) -> Result<(), color_eyre::Report> {
        use crate::util::service::discovery::ServiceDiscovery;
        use crate::util::io::bus::MessageBus;

        // Check if discovery is already in progress
        if let Some(last_discovery) = self.state.last_discovery {
            if last_discovery.elapsed().as_secs() < 60 {
                return Ok(());
            }
        }

        let discovery = ServiceDiscovery::new(
            self.config.service_discovery.as_ref().unwrap().scan_pattern.clone(),
            self.config.service_discovery.as_ref().unwrap().web_port,
        );

        match discovery.discover_services().await {
            Ok(mut services) => {
                // Update advisor statuses
                for service in &mut services {
                    self.state.advisor_status.insert(service.name.clone(), service.status.clone());
                }

                self.state.available_advisors = services;
                self.state.last_discovery = Some(std::time::Instant::now());
                
                log_info!("✅ Discovered {} council advisors", self.state.available_advisors.len());
            }
            Err(e) => {
                log_error!("❌ Council advisor discovery failed: {}", e);
                // Keep previous advisors but mark as unavailable
                for advisor in &mut self.state.available_advisors {
                    advisor.status = "out_of_office".to_string();
                }
            }
        }

        Ok(())
    }

    fn get_active_advisor_endpoint(&self) -> Option<&str> {
        self.state.active_advisor.as_ref().map(|a| a.endpoint.as_str())
    }
}

#[async_trait::async_trait]
impl ModuleHandler for CouncilModuleHandler {
    async fn handle_key(&mut self, key_code: KeyCode) -> Option<AppEvent> {
        match key_code {
            KeyCode::Char('a') => {
                // Cycle through advisors
                if !self.state.available_advisors.is_empty() {
                    let current_idx = self.state.available_advisors.iter().position(|a| {
                        self.state.active_advisor.as_ref().map(|active| active.name == a.name).unwrap_or(false)
                    });

                    let new_idx = match current_idx {
                        Some(idx) if idx + 1 < self.state.available_advisors.len() => idx + 1,
                        Some(_) => 0,
                        None => 0,
                    };

                    self.state.active_advisor = Some(self.state.available_advisors[new_idx].clone());
                    log_info!("Selected advisor: {}", self.state.available_advisors[new_idx].name);
                    
                    Some(AppEvent::NoOp)
                } else {
                    None
                }
            }
            KeyCode::Char('r') => {
                // Refresh advisor list
                let _ = self.discover_advisors().await;
                Some(AppEvent::NoOp)
            }
            _ => None,
        }
    }

    async fn process_messages(&mut self, message: &BusMessage) -> Result<(), color_eyre::Report> {
        // Handle council-specific messages
        if message.topic == "council_chat" {
            // Route to active advisor if one is selected
            if let Some(endpoint) = self.get_active_advisor_endpoint() {
                // Forward the message to the advisor's endpoint
                log_debug!("Routing council message to: {}", endpoint);
                // TODO: Implement actual message forwarding
            }
        }

        Ok(())
    }

    async fn update_bindings(&mut self) -> Result<(), color_eyre::Report> {
        // Update advisor status
        for advisor in &mut self.state.available_advisors {
            if let Some(status) = self.state.advisor_status.get(&advisor.name) {
                advisor.status = status.clone();
            }
        }

        // Update model_info binding
        let model_info = if let Some(active) = &self.state.active_advisor {
            format!("Consulting {} ({})", active.title, active.status)
        } else if !self.state.available_advisors.is_empty() {
            format!("{} advisors available", self.state.available_advisors.len())
        } else {
            "No advisors available".to_string()
        };

        // Update bindings via config
        self.config.bindings.insert("model_info".to_string(), serde_json::Value::String(model_info));
        
        // Update available_advisors binding
        let advisors_json = serde_json::to_value(&self.state.available_advisors)?;
        self.config.bindings.insert("available_advisors".to_string(), advisors_json);

        Ok(())
    }

    fn get_module_name(&self) -> &str {
        &self.config.name
    }

    fn get_module_type(&self) -> &str {
        &self.config.module_type
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl RenderDefault for CouncilModuleHandler {
    fn render(&self, _area: Rect, _buf: &mut Buffer) -> Result<(), color_eyre::Report> {
        // Default rendering - actual template will be used
        Ok(())
    }
}