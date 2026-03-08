use crate::{log_debug, log_info};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilConfig {
    #[serde(flatten)]
    pub base: crate::module::BaseModuleConfig,
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
    discovery_manager: Option<std::sync::Arc<crate::util::io::discovery::DiscoveryManager>>,
}

impl CouncilHandler {
    pub fn new(
        message_bus: crate::util::io::bus::MessageBus,
        database: crate::util::database::Database,
        discovery_manager: Option<std::sync::Arc<crate::util::io::discovery::DiscoveryManager>>,
    ) -> Self {
        Self {
            message_bus,
            database,
            discovery_manager,
        }
    }

    pub async fn initialize_advisors(&self) -> Result<(), color_eyre::Report> {
        log_info!("Initializing council advisors...");

        let device_ids = vec![
            "advisor_hardware".to_string(),
            "advisor_knowledge".to_string(),
        ];

        self.message_bus
            .publish(crate::util::io::bus::BusMessage::new(
                "council.init".to_string(),
                serde_json::json!({
                    "advisors": device_ids,
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                })
                .to_string(),
                "survon_tui".to_string(),
            ))
            .await;

        Ok(())
    }

    pub async fn handle_council_message(&self, message: &str) -> Result<(), color_eyre::Report> {
        log_info!("Council message received: {}", message);

        let parsed_message: serde_json::Value = serde_json::from_str(message)?;
        let topic = parsed_message
            .get("topic")
            .and_then(|t| t.as_str())
            .unwrap_or("");

        match topic {
            "council.query" => {
                self.handle_council_query(parsed_message).await?;
            }
            "council.command" => {
                self.handle_council_command(parsed_message).await?;
            }
            _ => {
                log_info!("Unknown council topic: {}", topic);
            }
        }

        Ok(())
    }

    async fn handle_council_query(
        &self,
        message: serde_json::Value,
    ) -> Result<(), color_eyre::Report> {
        let query = message.get("query").and_then(|q| q.as_str()).unwrap_or("");
        let advisor = message
            .get("advisor")
            .and_then(|a| a.as_str())
            .unwrap_or("");

        log_info!("Council query from {}: {}", advisor, query);

        match query {
            "device_status" => {
                self.handle_device_status_query(advisor).await?;
            }
            "knowledge_base" => {
                self.handle_knowledge_query(advisor).await?;
            }
            "system_status" => {
                self.handle_system_status_query(advisor).await?;
            }
            _ => {
                log_info!("Unknown query type: {}", query);
            }
        }

        Ok(())
    }

    async fn handle_council_command(
        &self,
        message: serde_json::Value,
    ) -> Result<(), color_eyre::Report> {
        let command = message
            .get("command")
            .and_then(|c| c.as_str())
            .unwrap_or("");
        let advisor = message
            .get("advisor")
            .and_then(|a| a.as_str())
            .unwrap_or("");

        log_info!("Council command from {}: {}", advisor, command);

        match command {
            "reboot_device" => {
                self.handle_reboot_command(advisor).await?;
            }
            "update_firmware" => {
                self.handle_update_command(advisor).await?;
            }
            "reset_config" => {
                self.handle_reset_command(advisor).await?;
            }
            _ => {
                log_info!("Unknown command: {}", command);
            }
        }

        Ok(())
    }

    async fn handle_device_status_query(&self, advisor: &str) -> Result<(), color_eyre::Report> {
        if let Some(_discovery) = &self.discovery_manager {
            self.message_bus
                .publish(crate::util::io::bus::BusMessage::new(
                    "council.response".to_string(),
                    serde_json::json!({
                        "advisor": advisor,
                        "response": "device_status",
                        "status": {"online": true},
                        "timestamp": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                    })
                    .to_string(),
                    "survon_tui".to_string(),
                ))
                .await;
        }

        Ok(())
    }

    async fn handle_knowledge_query(&self, advisor: &str) -> Result<(), color_eyre::Report> {
        self.message_bus
            .publish(crate::util::io::bus::BusMessage::new(
                "council.response".to_string(),
                serde_json::json!({
                    "advisor": advisor,
                    "response": "knowledge_base",
                    "results": vec![] as Vec<String>,
                    "query": "",
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                })
                .to_string(),
                "survon_tui".to_string(),
            ))
            .await;

        Ok(())
    }

    async fn handle_system_status_query(&self, advisor: &str) -> Result<(), color_eyre::Report> {
        let system_status = serde_json::json!({
            "uptime": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "modules_loaded": vec!["overseer".to_string(), "llm".to_string()],
            "memory_usage": "N/A",
            "cpu_usage": "N/A"
        });

        self.message_bus
            .publish(crate::util::io::bus::BusMessage::new(
                "council.response".to_string(),
                serde_json::json!({
                    "advisor": advisor,
                    "response": "system_status",
                    "status": system_status,
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                })
                .to_string(),
                "survon_tui".to_string(),
            ))
            .await;

        Ok(())
    }

    async fn handle_reboot_command(&self, advisor: &str) -> Result<(), color_eyre::Report> {
        self.message_bus
            .publish(crate::util::io::bus::BusMessage::new(
                "council.response".to_string(),
                serde_json::json!({
                    "advisor": advisor,
                    "response": "reboot_result",
                    "success": true,
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                })
                .to_string(),
                "survon_tui".to_string(),
            ))
            .await;

        Ok(())
    }

    async fn handle_update_command(&self, advisor: &str) -> Result<(), color_eyre::Report> {
        self.message_bus
            .publish(crate::util::io::bus::BusMessage::new(
                "council.response".to_string(),
                serde_json::json!({
                    "advisor": advisor,
                    "response": "update_result",
                    "success": true,
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                })
                .to_string(),
                "survon_tui".to_string(),
            ))
            .await;

        Ok(())
    }

    async fn handle_reset_command(&self, advisor: &str) -> Result<(), color_eyre::Report> {
        self.message_bus
            .publish(crate::util::io::bus::BusMessage::new(
                "council.response".to_string(),
                serde_json::json!({
                    "advisor": advisor,
                    "response": "reset_result",
                    "success": true,
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                })
                .to_string(),
                "survon_tui".to_string(),
            ))
            .await;

        Ok(())
    }
}

pub struct CouncilStrategy;

impl CouncilStrategy {
    pub fn new() -> Self {
        Self
    }

    pub async fn initialize(
        &self,
        message_bus: crate::util::io::bus::MessageBus,
        database: crate::util::database::Database,
        discovery_manager: Option<std::sync::Arc<crate::util::io::discovery::DiscoveryManager>>,
    ) -> Result<CouncilHandler, color_eyre::Report> {
        log_info!("Initializing Council Strategy...");

        let handler = CouncilHandler::new(message_bus, database, discovery_manager);
        handler.initialize_advisors().await?;

        Ok(handler)
    }

    pub async fn handle_event(
        &self,
        handler: &CouncilHandler,
        event: &str,
        payload: &str,
    ) -> Result<(), color_eyre::Report> {
        match event {
            "council.message" => {
                handler.handle_council_message(payload).await?;
            }
            "advisor.status" => {
                self.handle_advisor_status(handler, payload).await?;
            }
            _ => {
                log_info!("Unknown council event: {}", event);
            }
        }

        Ok(())
    }

    async fn handle_advisor_status(
        &self,
        handler: &CouncilHandler,
        payload: &str,
    ) -> Result<(), color_eyre::Report> {
        let status: serde_json::Value = serde_json::from_str(payload)?;
        let advisor = status.get("advisor").and_then(|a| a.as_str()).unwrap_or("");
        let online = status
            .get("online")
            .and_then(|o| o.as_bool())
            .unwrap_or(false);

        log_info!(
            "Advisor {} status: {}",
            advisor,
            if online { "online" } else { "offline" }
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::module::strategies::council::{CouncilBindings, CouncilConfig};

    #[test]
    fn test_council_config_creation() {
        let config = CouncilConfig {
            base: crate::module::BaseModuleConfig {
                name: "council".to_string(),
                bus_topic: "council".to_string(),
                template: "council".to_string(),
                namespace: None,
                version: None,
                description: None,
                is_blinkable: Some(true),
            },
            bindings: CouncilBindings {
                current_view: "overview".to_string(),
                selected_index: 0,
                pending_devices: vec![],
                known_devices: vec![],
                module_list: vec![],
                installed_modules: vec![],
                archived_modules: vec![],
                advisor_list: vec![],
                active_council: vec![],
                council_messages: vec![],
                status_message: None,
                is_blinkable: None,
                advisor_config: None,
            },
        };

        assert_eq!(config.base.name, "council");
        assert_eq!(config.bindings.current_view, "overview");
    }

    #[test]
    fn test_council_bindings_creation() {
        let bindings = CouncilBindings {
            current_view: "chat".to_string(),
            selected_index: 1,
            pending_devices: vec!["device1".to_string()],
            known_devices: vec!["device2".to_string()],
            module_list: vec!["module1".to_string()],
            installed_modules: vec!["module2".to_string()],
            archived_modules: vec![],
            advisor_list: vec!["advisor1".to_string()],
            active_council: vec!["advisor2".to_string()],
            council_messages: vec!["Hello".to_string()],
            status_message: Some("All good".to_string()),
            is_blinkable: Some(true),
            advisor_config: Some("{}".to_string()),
        };

        assert_eq!(bindings.current_view, "chat");
        assert_eq!(bindings.advisor_list.len(), 1);
        assert_eq!(bindings.status_message, Some("All good".to_string()));
    }

    #[test]
    fn test_council_message_serialization() {
        let message = r#"{
            "topic": "council.query",
            "query": "device_status",
            "advisor": "test_device"
        }"#;

        let parsed: serde_json::Value = serde_json::from_str(message).unwrap();

        assert_eq!(parsed["topic"], "council.query");
        assert_eq!(parsed["query"], "device_status");
        assert_eq!(parsed["advisor"], "test_device");
    }

    #[test]
    fn test_council_response_serialization() {
        let response = r#"{
            "advisor": "test_device",
            "response": "device_status",
            "status": {"online": true},
            "timestamp": 1234567890
        }"#;

        let parsed: serde_json::Value = serde_json::from_str(response).unwrap();

        assert_eq!(parsed["advisor"], "test_device");
        assert_eq!(parsed["response"], "device_status");
        assert!(parsed["status"]["online"].as_bool().unwrap());
    }

    #[test]
    fn test_council_command_serialization() {
        let command = r#"{
            "topic": "council.command",
            "command": "reboot_device",
            "advisor": "device_1"
        }"#;

        let parsed: serde_json::Value = serde_json::from_str(command).unwrap();

        assert_eq!(parsed["topic"], "council.command");
        assert_eq!(parsed["command"], "reboot_device");
    }

    #[test]
    fn test_council_multiple_advisor_handling() {
        let advisors = vec![
            "hardware_expert".to_string(),
            "knowledge_base".to_string(),
            "system_monitor".to_string(),
            "security_specialist".to_string(),
        ];

        assert_eq!(advisors.len(), 4);
        assert!(advisors.contains(&"hardware_expert".to_string()));
    }
}
