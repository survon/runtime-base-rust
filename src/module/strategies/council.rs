use serde::{Deserialize, Serialize};
use crate::module::BaseModuleConfig;
use crate::module::strategies::overseer::database::trait_overseer_database::OverseerDatabase;
use crate::module::strategies::overseer::database::get_all_known_devices::GetAllKnownDevices;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilConfig {
    #[serde(flatten)]
    pub base: BaseModuleConfig,
    pub bindings: CouncilBindings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilBindings {
    pub current_view: String,
    pub selected_index: i32,
    pub pending_devices: Vec<String>,
    pub known_devices: Vec<String>,
    pub module_list: Vec<String>,
    pub installed_modules: Vec<String>,
    pub archived_modules: Vec<String>,
    pub advisor_list: Vec<String>,
    pub active_council: Vec<String>,
    pub council_messages: Vec<String>,
    
    #[serde(default)]
    pub status_message: Option<String>,
    #[serde(default)]
    pub is_blinkable: Option<bool>,
    #[serde(default)]
    pub advisor_config: Option<String>,
}

pub struct CouncilHandler {
    message_bus: crate::util::io::bus::MessageBus,
    database: crate::util::database::Database,
    discovery_manager: Option<Arc<crate::util::io::discovery::DiscoveryManager>>,
}

impl CouncilHandler {
    pub fn new(
        message_bus: crate::util::io::bus::MessageBus,
        database: crate::util::database::Database,
        discovery_manager: Option<Arc<crate::util::io::discovery::DiscoveryManager>>,
    ) -> Self {
        Self {
            message_bus,
            database,
            discovery_manager,
        }
    }

    pub async fn initialize_advisors(&self) -> Result<(), color_eyre::Report> {
        // Initialize council advisors
        log_info!("\ud83d\udce3 Initializing council advisors...");
        
        // Get all known devices
        let all_devices = GetAllKnownDevices::new(&self.database).await?;
        let device_ids: Vec<String> = all_devices.iter().map(|d| d.device_id.clone()).collect();
        
        // Publish advisor initialization
        self.message_bus.publish(crate::util::io::bus::BusMessage::new(
            "council.init".to_string(),
            serde_json::json!({
                "advisors": device_ids,
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            }).to_string(),
            "survon_tui".to_string(),
        )).await;
        
        Ok(())
    }

    pub async fn handle_council_message(&self, message: &str) -> Result<(), color_eyre::Report> {
        // Handle council messages and route to appropriate advisors
        log_info!("\ud83d\udce3 Council message received: {}", message);
        
        // Parse message and route to advisors
        let parsed_message: serde_json::Value = serde_json::from_str(message)?;
        let topic = parsed_message.get("topic").and_then(|t| t.as_str()).unwrap_or("");
        
        // Route to appropriate advisor
        match topic {
            "council.query" => {
                self.handle_council_query(parsed_message).await?;
            }
            "council.command" => {
                self.handle_council_command(parsed_message).await?;
            }
            _ => {
                log_info!("\u2139\ufe0f  Unknown council topic: {}", topic);
            }
        }
        
        Ok(())
    }

    async fn handle_council_query(&self, message: serde_json::Value) -> Result<(), color_eyre::Report> {
        // Handle council queries
        let query = message.get("query").and_then(|q| q.as_str()).unwrap_or("");
        let advisor = message.get("advisor").and_then(|a| a.as_str()).unwrap_or("");
        
        log_info!("\ud83d\udcca Council query from {}: {}", advisor, query);
        
        // Route query to appropriate system
        match query {
            "device_status" => {
                self.handle_device_status_query(advisor).await?;
            }
            "knowledge_base" => {
                self.handle_knowledge_query(advisor, message).await?;
            }
            "system_status" => {
                self.handle_system_status_query(advisor).await?;
            }
            _ => {
                log_info!("\u2139\ufe0f  Unknown query type: {}", query);
            }
        }
        
        Ok(())
    }

    async fn handle_council_command(&self, message: serde_json::Value) -> Result<(), color_eyre::Report> {
        // Handle council commands
        let command = message.get("command").and_then(|c| c.as_str()).unwrap_or("");
        let advisor = message.get("advisor").and_then(|a| a.as_str()).unwrap_or("");
        
        log_info!("\u26a1 Council command from {}: {}", advisor, command);
        
        // Execute command
        match command {
            "reboot_device" => {
                self.handle_reboot_command(advisor, message).await?;
            }
            "update_firmware" => {
                self.handle_update_command(advisor, message).await?;
            }
            "reset_config" => {
                self.handle_reset_command(advisor, message).await?;
            }
            _ => {
                log_info!("\u26a1 Unknown command: {}", command);
            }
        }
        
        Ok(())
    }

    async fn handle_device_status_query(&self, advisor: &str) -> Result<(), color_eyre::Report> {
        // Get device status from discovery manager
        if let Some(discovery) = &self.discovery_manager {
            let status = discovery.get_device_status(advisor).await?;
            
            // Send status back to council
            self.message_bus.publish(crate::util::io::bus::BusMessage::new(
                "council.response".to_string(),
                serde_json::json!({
                    "advisor": advisor,
                    "response": "device_status",
                    "status": status,
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                }).to_string(),
                "survon_tui".to_string(),
            )).await;
        }
        
        Ok(())
    }

    async fn handle_knowledge_query(&self, advisor: &str, message: serde_json::Value) -> Result<(), color_eyre::Report> {
        // Query knowledge base
        let query = message.get("query").and_then(|q| q.as_str()).unwrap_or("");
        
        // Get knowledge results
        let knowledge_results = self.database.search_knowledge(query).await?;
        
        // Send knowledge response to council
        self.message_bus.publish(crate::util::io::bus::BusMessage::new(
            "council.response".to_string(),
            serde_json::json!({
                "advisor": advisor,
                "response": "knowledge_base",
                "results": knowledge_results,
                "query": query,
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            }).to_string(),
            "survon_tui".to_string(),
        )).await;
        
        Ok(())
    }

    async fn handle_system_status_query(&self, advisor: &str) -> Result<(), color_eyre::Report> {
        // Get system status
        let system_status = serde_json::json!({
            "uptime": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "modules_loaded": self.get_loaded_modules(),
            "memory_usage": "N/A",
            "cpu_usage": "N/A"
        });
        
        // Send system status response
        self.message_bus.publish(crate::util::io::bus::BusMessage::new(
            "council.response".to_string(),
            serde_json::json!({
                "advisor": advisor,
                "response": "system_status",
                "status": system_status,
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            }).to_string(),
            "survon_tui".to_string(),
        )).await;
        
        Ok(())
    }

    async fn handle_reboot_command(&self, advisor: &str, message: serde_json::Value) -> Result<(), color_eyre::Report> {
        // Handle reboot command
        if let Some(discovery) = &self.discovery_manager {
            let success = discovery.send_command(
                advisor.to_string(),
                "reboot",
                Some(serde_json::json!({ "force": true })),
                crate::util::io::ble_scheduler::CommandPriority::Critical,
            ).await?;
            
            // Send reboot response
            self.message_bus.publish(crate::util::io::bus::BusMessage::new(
                "council.response".to_string(),
                serde_json::json!({
                    "advisor": advisor,
                    "response": "reboot_result",
                    "success": success,
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                }).to_string(),
                "survon_tui".to_string(),
            )).await;
        }
        
        Ok(())
    }

    async fn handle_update_command(&self, advisor: &str, message: serde_json::Value) -> Result<(), color_eyre::Report> {
        // Handle firmware update command
        if let Some(discovery) = &self.discovery_manager {
            let firmware_url = message.get("firmware_url").and_then(|u| u.as_str()).unwrap_or("");
            
            let success = discovery.send_command(
                advisor.to_string(),
                "update_firmware",
                Some(serde_json::json!({ "url": firmware_url })),
                crate::util::io::ble_scheduler::CommandPriority::High,
            ).await?;
            
            // Send update response
            self.message_bus.publish(crate::util::io::bus::BusMessage::new(
                "council.response".to_string(),
                serde_json::json!({
                    "advisor": advisor,
                    "response": "update_result",
                    "success": success,
                    "firmware_url": firmware_url,
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                }).to_string(),
                "survon_tui".to_string(),
            )).await;
        }
        
        Ok(())
    }

    async fn handle_reset_command(&self, advisor: &str, message: serde_json::Value) -> Result<(), color_eyre::Report> {
        // Handle config reset command
        if let Some(discovery) = &self.discovery_manager {
            let success = discovery.send_command(
                advisor.to_string(),
                "reset_config",
                Some(serde_json::json!({ "confirm": true })),
                crate::util::io::ble_scheduler::CommandPriority::High,
            ).await?;
            
            // Send reset response
            self.message_bus.publish(crate::util::io::bus::BusMessage::new(
                "council.response".to_string(),
                serde_json::json!({
                    "advisor": advisor,
                    "response": "reset_result",
                    "success": success,
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                }).to_string(),
                "survon_tui".to_string(),
            )).await;
        }
        
        Ok(())
    }

    fn get_loaded_modules(&self) -> Vec<String> {
        // Get list of loaded modules from managers
        vec![
            "overseer".to_string(),
            "llm".to_string(),
            "jukebox".to_string(),
            "messages".to_string(),
        ]
    }
}

pub struct CouncilStrategy;

impl CouncilStrategy {
    pub fn new() -> Self {
        Self
    }

    pub async fn initialize(&self, message_bus: crate::util::io::bus::MessageBus, database: crate::util::database::Database, discovery_manager: Option<Arc<crate::util::io::discovery::DiscoveryManager>>) -> Result<CouncilHandler, color_eyre::Report> {
        log_info!("\ud83d\udce3 Initializing Council Strategy...");
        
        let handler = CouncilHandler::new(message_bus, database, discovery_manager);
        
        // Initialize advisors
        handler.initialize_advisors().await?;
        
        Ok(handler)
    }

    pub async fn handle_event(&self, handler: &CouncilHandler, event: &str, payload: &str) -> Result<(), color_eyre::Report> {
        match event {
            "council.message" => {
                handler.handle_council_message(payload).await?;
            }
            "advisor.status" => {
                self.handle_advisor_status(handler, payload).await?;
            }
            _ => {
                log_info!("\u2139\ufe0f  Unknown council event: {}", event);
            }
        }
        
        Ok(())
    }

    async fn handle_advisor_status(&self, handler: &CouncilHandler, payload: &str) -> Result<(), color_eyre::Report> {
        // Handle advisor status updates
        let status: serde_json::Value = serde_json::from_str(payload)?;
        let advisor = status.get("advisor").and_then(|a| a.as_str()).unwrap_or("");
        let online = status.get("online").and_then(|o| o.as_bool()).unwrap_or(false);
        
        log_info!("\ud83d\udce3 Advisor {} status: {}", advisor, if online { "online" } else { "offline" });
        
        // Update advisor status in database
        handler.database.update_advisor_status(advisor, online).await?;
        
        Ok(())
    }
}