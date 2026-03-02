use serde::{Deserialize, Serialize};
use crate::module::strategies::council::CouncilHandler;
use crate::module::strategies::council::CouncilConfig;
use crate::module::strategies::council::CouncilBindings;
use crate::util::io::bus::MessageBus;
use crate::util::database::Database;
use std::sync::Arc;
use tokio::sync::mpsc;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilAdvisor {
    pub id: String,
    pub name: String,
    pub role: String,
    pub capabilities: Vec<String>,
    pub status: AdvisorStatus,
    pub last_seen: Option<u64>>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdvisorStatus {
    Online,
    Offline,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilMessage {
    pub id: String,
    pub advisor_id: String,
    pub topic: String,
    pub content: String,
    pub timestamp: u64,
    pub priority: MessagePriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
}

pub struct CouncilService {
    config: CouncilConfig,
    message_bus: MessageBus,
    database: Database,
    advisors: HashMap<String, CouncilAdvisor>,
    message_tx: mpsc::Sender<CouncilMessage>,
    message_rx: mpsc::Receiver<CouncilMessage>,
    running: bool,
}

impl CouncilService {
    pub fn new(
        config: CouncilConfig,
        message_bus: MessageBus,
        database: Database,
    ) -> Self {
        let (tx, rx) = mpsc::channel<CouncilMessage>(100);
        
        Self {
            config,
            message_bus,
            database,
            advisors: HashMap::new(),
            message_tx: tx,
            message_rx: rx,
            running: true,
        }
    }

    pub async fn start(&mut self) -> Result<(), color_eyre::Report> {
        log_info!("\ud83d\udce3 Council Service starting...");
        
        // Initialize advisors
        self.initialize_advisors().await?;
        
        // Start message processing
        self.start_message_processor().await?;
        
        // Start advisor monitoring
        self.start_advisor_monitor().await?;
        
        log_info!("\u2705 Council Service running");
        Ok(())
    }

    async fn initialize_advisors(&mut self) -> Result<(), color_eyre::Report> {
        log_info!("\ud83d\udce3 Discovering council advisors...");
        
        // Get all known devices
        let all_devices = self.database.get_all_known_devices().await?;
        
        // Create advisors from devices
        for device in all_devices {
            let advisor = CouncilAdvisor {
                id: device.device_id.clone(),
                name: format!("Advisor-{}", device.device_id),
                role: "Device Advisor".to_string(),
                capabilities: vec![
                    "device_status".to_string(),
                    "system_health".to_string(),
                    "maintenance".to_string(),
                ],
                status: AdvisorStatus::Unknown,
                last_seen: None,
                config: None,
            };
            
            self.advisors.insert(device.device_id.clone(), advisor);
        }
        
        // Add virtual advisors
        self.add_virtual_advisors();
        
        Ok(())
    }

    fn add_virtual_advisors(&mut self) {
        // Add knowledge advisor
        let knowledge_advisor = CouncilAdvisor {
            id: "knowledge".to_string(),
            name: "Knowledge Advisor".to_string(),
            role: "Information Specialist".to_string(),
            capabilities: vec![
                "knowledge_base".to_string(),
                "research".to_string(),
                "documentation".to_string(),
            ],
            status: AdvisorStatus::Online,
            last_seen: Some(std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()),
            config: None,
        };
        self.advisors.insert("knowledge".to_string(), knowledge_advisor);
        
        // Add system advisor
        let system_advisor = CouncilAdvisor {
            id: "system".to_string(),
            name: "System Advisor".to_string(),
            role: "System Administrator".to_string(),
            capabilities: vec![
                "system_status".to_string(),
                "performance".to_string(),
                "security".to_string(),
            ],
            status: AdvisorStatus::Online,
            last_seen: Some(std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()),
            config: None,
        };
        self.advisors.insert("system".to_string(), system_advisor);
        
        // Add LLM advisor
        let llm_advisor = CouncilAdvisor {
            id: "llm".to_string(),
            name: "LLM Advisor".to_string(),
            role: "AI Assistant".to_string(),
            capabilities: vec![
                "conversation".to_string(),
                "analysis".to_string(),
                "creativity".to_string(),
            ],
            status: AdvisorStatus::Online,
            last_seen: Some(std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()),
            config: None,
        };
        self.advisors.insert("llm".to_string(), llm_advisor);
    }

    async fn start_message_processor(&mut self) -> Result<(), color_eyre::Report> {
        log_info!("\ud83d\udce3 Starting council message processor...");
        
        // Subscribe to council topics
        self.message_bus.subscribe("council.*".to_string()).await;
        
        // Spawn message processing task
        let message_bus_clone = self.message_bus.clone();
        let database_clone = self.database.clone();
        let advisors_clone = self.advisors.clone();
        let message_tx_clone = self.message_tx.clone();
        
        tokio::spawn(async move {
            while let Ok(message) = message_bus_clone.recv().await {
                if let Ok(parsed) = serde_json::from_str::<CouncilMessage>(&message.payload) {
                    // Process council message
                    if let Err(e) = message_tx_clone.send(parsed).await {
                        log_error!("Failed to send council message: {}", e);
                    }
                }
            }
        });
        
        Ok(())
    }

    async fn start_advisor_monitor(&mut self) -> Result<(), color_eyre::Report> {
        log_info!("\ud83d\udee1️  Starting advisor status monitoring...");
        
        // Monitor advisor status every 30 seconds
        let message_bus_clone = self.message_bus.clone();
        let database_clone = self.database.clone();
        let advisors_clone = self.advisors.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // Check advisor status
                for (id, advisor) in &advisors_clone {
                    match id.as_str() {
                        "knowledge" | "system" | "llm" => {
                            // Virtual advisors always online
                            continue;
                        }
                        _ => {
                            // Check device status
                            if let Ok(status) = database_clone.get_device_status(id).await {
                                let online = status.is_some();
                                let new_status = if online {
                                    AdvisorStatus::Online
                                } else {
                                    AdvisorStatus::Offline
                                };
                                
                                if advisor.status != new_status {
                                    // Update status
                                    let _ = database_clone.update_advisor_status(id, online).await;
                                }
                            }
                        }
                    }
                }
            }
        });
        
        Ok(())
    }

    pub async fn submit_message(
        &self,
        advisor_id: &str,
        topic: &str,
        content: &str,
        priority: MessagePriority,
    ) -> Result<(), color_eyre::Report> {
        let message = CouncilMessage {
            id: uuid::Uuid::new_v4().to_string(),
            advisor_id: advisor_id.to_string(),
            topic: topic.to_string(),
            content: content.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            priority,
        };
        
        // Send message to advisor
        self.message_tx.send(message).await?;
        
        Ok(())
    }

    pub async fn get_advisors(&self) -> Vec<CouncilAdvisor> {
        self.advisors.values().cloned().collect()
    }

    pub async fn get_advisor(&self, advisor_id: &str) -> Option<CouncilAdvisor> {
        self.advisors.get(advisor_id).cloned()
    }

    pub async fn get_messages(&self) -> Vec<CouncilMessage> {
        // This would need to be implemented with a message store
        vec![]
    }

    pub fn stop(&mut self) {
        self.running = false;
    }
}

pub struct CouncilServiceManager {
    services: HashMap<String, CouncilService>,
}

impl CouncilServiceManager {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub async fn register_service(
        &mut self,
        name: &str,
        service: CouncilService,
    ) -> Result<(), color_eyre::Report> {
        self.services.insert(name.to_string(), service);
        Ok(())
    }

    pub async fn get_service(&self, name: &str) -> Option<&CouncilService> {
        self.services.get(name)
    }

    pub async fn broadcast_message(
        &self,
        topic: &str,
        content: &str,
        priority: MessagePriority,
    ) -> Result<(), color_eyre::Report> {
        for service in self.services.values() {
            // Broadcast to all advisors
            for advisor in service.get_advisors().await {
                service.submit_message(
                    &advisor.id,
                    topic,
                    content,
                    priority,
                ).await?;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_council_service_initialization() {
        let config = CouncilConfig {
            base: Default::default(),
            bindings: Default::default(),
        };
        
        let message_bus = MessageBus::new().0;
        let database = Database::new_implied_all_schemas().unwrap();
        
        let mut service = CouncilService::new(config, message_bus, database);
        
        assert!(service.advisors.is_empty());
        
        service.initialize_advisors().await.unwrap();
        
        assert!(!service.advisors.is_empty());
        assert!(service.advisors.contains_key("knowledge"));
        assert!(service.advisors.contains_key("system"));
        assert!(service.advisors.contains_key("llm"));
    }

    #[tokio::test]
    async fn test_council_message_submission() {
        let config = CouncilConfig {
            base: Default::default(),
            bindings: Default::default(),
        };
        
        let message_bus = MessageBus::new().0;
        let database = Database::new_implied_all_schemas().unwrap();
        
        let mut service = CouncilService::new(config, message_bus, database);
        service.initialize_advisors().await.unwrap();
        
        let result = service.submit_message(
            "knowledge",
            "query",
            "What is the capital of France?",
            MessagePriority::Normal,
        ).await;
        
        assert!(result.is_ok());
    }
}