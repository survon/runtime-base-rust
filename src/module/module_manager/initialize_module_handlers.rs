use std::{
    path::PathBuf,
    sync::Arc,
};

use crate::{
    log_debug,
    log_info,
    log_warn,
    module::{
        ModuleManager,
        strategies::{llm, monitoring, overseer, side_quest, valve_control},
    },
    util::{
        database::Database,
        io::{
            bus::MessageBus,
            discovery::DiscoveryManager,
        },
    }
};

impl ModuleManager {
    pub async fn initialize_module_handlers(
        &mut self,
        wasteland_path: PathBuf,
        discovery_manager: Option<Arc<DiscoveryManager>>,
        database: &Database,
        message_bus: &MessageBus
    ) -> color_eyre::Result<()> {
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

        log_info!("🔧 Initializing module handlers for namespace: {}", self.namespace);

        for (module_type, device_id, bus_topic) in modules_info {
            match module_type.as_str() {
                "llm" => {
                    if !self.handlers.contains_key("llm") {
                        use crate::module::strategies::llm;

                        log_info!("📚 Registering LLM handler");

                        let llm_service = llm::create_llm_service_if_available(
                            self,
                            database,
                        ).await.ok().flatten();

                        let llm_handler = Box::new(llm::handler::LlmHandler::new(llm_service));
                        self.register_handler(llm_handler);

                        log_info!("✅ LLM handler registered");
                    }
                }

                "side_quest" => {
                    if !self.handlers.contains_key("side_quest") {
                        log_info!("🗺️  Registering Side Quest handler");

                        let handler = Box::new(
                            side_quest::handler::SideQuestHandler::new(
                                database.clone(),
                                message_bus.clone()
                            )
                        );
                        self.register_handler(handler);

                        log_info!("✅ Side Quest handler registered");
                    }
                }

                "overseer" => {
                    if !self.handlers.contains_key("overseer") {
                        log_info!("🗂️ Registering Wasteland Manager handler");

                        self.register_handler(Box::new(
                            overseer::handler::OverseerHandler::new(
                                wasteland_path.clone(),
                                discovery_manager.clone(),
                                database.clone(),
                                message_bus.clone()
                            )
                        ));

                        log_info!("✅ Wasteland Manager handler registered");
                    }
                }

                "valve_control" => {
                    if !self.handlers.contains_key("valve_control") && !device_id.is_empty() {
                        use crate::module::strategies::valve_control;

                        log_info!("🚰 Registering valve_control handler for device: {}", device_id);

                        let handler = Box::new(
                            valve_control::handler::ValveControlHandler::new(
                                message_bus.clone(),
                                device_id.clone(),
                                bus_topic.clone(),
                                discovery_manager.clone(),  // ← NEW!
                            )
                        );
                        self.register_handler(handler);

                        log_info!("✅ Valve control handler registered");
                    }
                }
                "monitoring" => {
                    let handler_key = format!("monitoring_{}", device_id);

                    if !self.handlers.contains_key(&handler_key) && !device_id.is_empty() {
                        use crate::module::strategies::monitoring;

                        log_info!("📊 Registering monitoring handler:");
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
                        log_debug!("ℹ️ Monitoring handler already exists: {}", handler_key);
                    }
                }

                "system" => {
                    // System modules don't need handlers yet
                }

                _ => {
                    log_debug!("No handler needed for module type: {}", module_type);
                }
            }
        }

        log_info!("📋 Handler registration complete. Active handlers:");
        for key in self.handlers.keys() {
            log_info!("   ✓ {}", key);
        }

        Ok(())
    }
}
