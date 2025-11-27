// src/modules/monitoring/handler.rs
use color_eyre::Result;
use ratatui::crossterm::event::KeyCode;
use std::any::Any;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use crate::{log_info, log_error, log_debug, log_warn};
use crate::modules::{Module, module_handler::ModuleHandler};
use crate::util::io::{
    event::AppEvent,
    bus::{MessageBus},
};

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
enum HandlerMessage {
    TelemetryReceived {
        value_a: f64,
        value_b: f64,
        value_c: i64,
        timestamp: Instant,
    },
}

/// Handles monitoring sensors (gauges, meters, etc.)
/// Works with multiple instances - each monitors its own bus_topic
#[derive(Debug)]
pub struct MonitoringHandler {
    device_id: String,
    last_update: Option<Instant>,
    current_values: (f64, f64, i64), // (a, b, c)
    message_bus: MessageBus,
    // Channel for async updates
    message_tx: mpsc::UnboundedSender<HandlerMessage>,
    message_rx: mpsc::UnboundedReceiver<HandlerMessage>,
}

impl MonitoringHandler {
    pub fn new(message_bus: MessageBus, device_id: String, bus_topic: String) -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        let mut handler = Self {
            device_id: device_id.clone(),
            last_update: None,
            current_values: (0.0, 0.0, 0),
            message_bus: message_bus.clone(),
            message_tx,
            message_rx,
        };

        // Start listening for telemetry from the device
        handler.start_telemetry_listener(bus_topic);

        handler
    }

    fn start_telemetry_listener(&mut self, bus_topic: String) {
        let tx = self.message_tx.clone();
        let bus = self.message_bus.clone();
        let device_id = self.device_id.clone();

        tokio::spawn(async move {
            log_info!("🔊 Starting monitoring telemetry listener for topic: {}", bus_topic);
            let mut receiver = bus.subscribe(bus_topic.clone()).await;

            while let Some(msg) = receiver.recv().await {
                log_debug!("📨 Monitoring handler received message for device {}: {}", device_id, msg.payload);

                // Parse the payload - it could be in multiple formats:
                // 1. Direct compact format: {"a":72,"b":45,"c":1}
                // 2. Full SSP format: {"p":"ssp/1.0","t":"tel","i":"a01","d":{"a":72,"b":45,"c":1}}
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&msg.payload) {

                    // Try direct format first (what's actually being sent)
                    if let Some(value_a) = data.get("a").and_then(|v| v.as_f64()) {
                        // Direct format: {"a":72,"b":45,"c":1}
                        let value_b = data.get("b")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);

                        let value_c = data.get("c")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);

                        log_info!("✅ Monitoring telemetry [{}]: a={}, b={}, c={}",
                            device_id, value_a, value_b, value_c);

                        let _ = tx.send(HandlerMessage::TelemetryReceived {
                            value_a,
                            value_b,
                            value_c,
                            timestamp: Instant::now(),
                        });
                    }
                    // Fallback: Try full SSP format with "d" field
                    else if let Some(d) = data.get("d").and_then(|v| v.as_object()) {
                        let value_a = d.get("a")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);

                        let value_b = d.get("b")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);

                        let value_c = d.get("c")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);

                        log_info!("✅ Monitoring telemetry (SSP format) [{}]: a={}, b={}, c={}",
                            device_id, value_a, value_b, value_c);

                        let _ = tx.send(HandlerMessage::TelemetryReceived {
                            value_a,
                            value_b,
                            value_c,
                            timestamp: Instant::now(),
                        });
                    } else {
                        log_warn!("⚠️ Unrecognized telemetry format for {}: {}", device_id, msg.payload);
                    }
                } else {
                    log_error!("❌ Failed to parse JSON for {}: {}", device_id, msg.payload);
                }
            }

            log_warn!("🔇 Telemetry listener ended for topic: {}", bus_topic);
        });
    }

    fn process_messages(&mut self) {
        // Non-blocking: process all queued messages
        while let Ok(msg) = self.message_rx.try_recv() {
            match msg {
                HandlerMessage::TelemetryReceived { value_a, value_b, value_c, timestamp } => {
                    self.current_values = (value_a, value_b, value_c);
                    self.last_update = Some(timestamp);
                    log_debug!("📊 Updated values for {}: a={}, b={}, c={}",
                        self.device_id, value_a, value_b, value_c);
                }
            }
        }
    }

    fn is_connected(&self) -> bool {
        if let Some(last_update) = self.last_update {
            last_update.elapsed() < CONNECTION_TIMEOUT
        } else {
            false
        }
    }

    fn time_since_last_update(&self) -> Option<Duration> {
        self.last_update.map(|t| t.elapsed())
    }
}

impl ModuleHandler for MonitoringHandler {
    fn handle_key(&mut self, key_code: KeyCode, _module: &mut Module) -> Option<AppEvent> {
        match key_code {
            KeyCode::Char('r') => {
                // Manual refresh request (though telemetry is automatic)
                log_info!("Manual refresh requested for {}", self.device_id);
                None
            }
            KeyCode::Esc => Some(AppEvent::Back),
            _ => None,
        }
    }

    fn handle_event(&mut self, _event: &AppEvent, _module: &mut Module) -> Result<bool> {
        Ok(false)
    }

    fn update_bindings(&mut self, module: &mut Module) {
        // Process any queued async messages first
        self.process_messages();

        let (value_a, value_b, value_c) = self.current_values;

        // Update the SSP compact keys that the gauge template reads
        module.config.bindings.insert(
            "a".to_string(),
            serde_json::json!(value_a),
        );

        module.config.bindings.insert(
            "b".to_string(),
            serde_json::json!(value_b),
        );

        module.config.bindings.insert(
            "c".to_string(),
            serde_json::json!(value_c),
        );

        // Add connection status
        let is_connected = self.is_connected();
        module.config.bindings.insert(
            "is_connected".to_string(),
            serde_json::json!(is_connected),
        );

        // Add time since last update (for debugging)
        if let Some(elapsed) = self.time_since_last_update() {
            module.config.bindings.insert(
                "seconds_since_update".to_string(),
                serde_json::json!(elapsed.as_secs()),
            );
        }

        // Update display name to show connection status
        if !is_connected {
            module.config.bindings.insert(
                "status_suffix".to_string(),
                serde_json::json!(" [Disconnected]"),
            );

            if self.last_update.is_none() {
                log_warn!("⚠️ Device {} has never sent telemetry", self.device_id);
            } else {
                log_warn!("⚠️ Device {} connection lost ({}s since last update)",
                    self.device_id,
                    self.time_since_last_update().unwrap().as_secs()
                );
            }
        } else {
            module.config.bindings.insert(
                "status_suffix".to_string(),
                serde_json::json!(""),
            );
        }
    }

    fn module_type(&self) -> &str {
        "monitoring"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
