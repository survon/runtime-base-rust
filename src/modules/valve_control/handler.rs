// src/modules/valve_control/handler.rs
use color_eyre::Result;
use ratatui::crossterm::event::KeyCode;
use std::any::Any;
use tokio::sync::mpsc;
use crate::{log_info, log_error, log_debug};
use crate::modules::{Module, module_handler::ModuleHandler};
use crate::util::io::{
    event::AppEvent,
    bus::{MessageBus, BusMessage},
    serial::{SspMessage, Transport},
};

#[derive(Debug, Clone)]
enum HandlerMessage {
    StateChanged(bool),
    StatusUpdate(String),
    TelemetryReceived { valve_open: bool, sensor_value: f64 },
}

/// Handles valve control via Arduino/BLE
#[derive(Debug)]
pub struct ValveControlHandler {
    current_state: bool,  // true = open, false = closed
    target_state: bool,   // What state we're trying to achieve
    status_message: Option<String>,
    message_bus: MessageBus,
    device_id: String,
    // Channel for async updates
    message_tx: mpsc::UnboundedSender<HandlerMessage>,
    message_rx: mpsc::UnboundedReceiver<HandlerMessage>,
}

impl ValveControlHandler {
    pub fn new(message_bus: MessageBus, device_id: String, bus_topic: String) -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        let mut handler = Self {
            current_state: false,
            target_state: false,
            status_message: None,
            message_bus: message_bus.clone(),
            device_id: device_id.clone(),
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

        tokio::spawn(async move {
            log_info!("Starting valve telemetry listener for topic: {}", bus_topic);
            let mut receiver = bus.subscribe(bus_topic).await;

            while let Some(msg) = receiver.recv().await {
                log_debug!("Received valve telemetry message");

                // Parse the SSP compact format message
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&msg.payload) {
                    log_debug!("Received message payload: {}", msg.payload);

                    // SSP Compact Format: {"p":"ssp/1.0","t":"tel","i":"v01","d":{"a":1,"b":95,"c":123}}
                    if let Some(d) = data.get("d").and_then(|v| v.as_object()) {
                        // Extract valve state from "a" key (0=closed, 1=open)
                        let valve_open = d.get("a")
                            .and_then(|v| v.as_i64())
                            .map(|i| i != 0)
                            .unwrap_or(false);

                        // Extract position from "b" key (0-100%)
                        let sensor_value = d.get("b")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);

                        log_info!("Valve telemetry: open={}, position={}%", valve_open, sensor_value);

                        let _ = tx.send(HandlerMessage::TelemetryReceived {
                            valve_open,
                            sensor_value,
                        });
                    }
                }
            }
        });
    }

    fn process_messages(&mut self) {
        // Non-blocking: process all queued messages
        while let Ok(msg) = self.message_rx.try_recv() {
            match msg {
                HandlerMessage::StateChanged(new_state) => {
                    self.current_state = new_state;
                    self.status_message = Some(
                        if new_state { "✓ Valve opened".to_string() }
                        else { "✓ Valve closed".to_string() }
                    );
                }
                HandlerMessage::StatusUpdate(status) => {
                    self.status_message = Some(status);
                }
                HandlerMessage::TelemetryReceived { valve_open, sensor_value } => {
                    self.current_state = valve_open;
                    // Clear status message once telemetry confirms state
                    if valve_open == self.target_state {
                        self.status_message = None;
                    }
                }
            }
        }
    }

    fn toggle_valve(&mut self) {
        let new_state = !self.current_state;
        self.target_state = new_state;

        let action = if new_state { "open" } else { "close" };
        self.status_message = Some(format!("⏳ Sending {} command...", action));

        log_info!("Toggling valve to: {}", action);

        // Send command to Arduino via message bus
        let tx = self.message_tx.clone();
        let bus = self.message_bus.clone();
        let device_id = self.device_id.clone();

        tokio::spawn(async move {
            // Create SSP compact command message
            let command = serde_json::json!({
                "p": "ssp/1.0",
                "t": "cmd",
                "i": device_id,
                "s": chrono::Utc::now().timestamp() as u64,
                "d": {
                    "action": if new_state { "open" } else { "close" }
                }
            });

            let bus_msg = BusMessage::new(
                device_id.clone(),
                command.to_string(),
                "valve_control_handler".to_string(),
            );

            match bus.publish(bus_msg).await {
                Ok(_) => {
                    log_info!("✓ Valve command published");
                    let _ = tx.send(HandlerMessage::StatusUpdate(
                        format!("Command sent, waiting for response...")
                    ));
                }
                Err(e) => {
                    log_error!("Failed to publish valve command: {}", e);
                    let _ = tx.send(HandlerMessage::StatusUpdate(
                        format!("❌ Failed to send command: {}", e)
                    ));
                }
            }
        });
    }
}

impl ModuleHandler for ValveControlHandler {
    fn handle_key(&mut self, key_code: KeyCode, _module: &mut Module) -> Option<AppEvent> {
        match key_code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                // Toggle valve on Enter or Space
                self.toggle_valve();
                None
            }
            KeyCode::Char('o') => {
                // Force open
                if !self.current_state {
                    self.toggle_valve();
                }
                None
            }
            KeyCode::Char('c') => {
                // Force close
                if self.current_state {
                    self.toggle_valve();
                }
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

        // Update the state binding that the toggle_switch template reads
        module.config.bindings.insert(
            "state".to_string(),
            serde_json::json!(self.current_state),
        );

        // Update status message if present
        if let Some(status) = &self.status_message {
            module.config.bindings.insert(
                "status_message".to_string(),
                serde_json::json!(status),
            );
        }

        // Update description with current state
        let description = if self.current_state {
            "Valve is OPEN - Flow active"
        } else {
            "Valve is CLOSED - Flow stopped"
        };

        module.config.bindings.insert(
            "description".to_string(),
            serde_json::json!(description),
        );
    }

    fn module_type(&self) -> &str {
        "valve_control"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
