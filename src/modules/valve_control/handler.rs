// src/modules/valve_control/handler.rs - REFACTORED with Schedule Support
use color_eyre::Result;
use ratatui::crossterm::event::KeyCode;
use std::any::Any;
use tokio::sync::mpsc;
use crate::{log_info, log_error, log_debug};
use crate::modules::{Module, module_handler::ModuleHandler};
use crate::util::io::{
    event::AppEvent,
    bus::{MessageBus, BusMessage},
    ble_scheduler::{extract_schedule_metadata, CommandPriority}, // NEW: Import schedule support
};

#[derive(Debug, Clone)]
enum HandlerMessage {
    StateChanged(bool),
    StatusUpdate(String),
    TelemetryReceived { valve_open: bool, sensor_value: f64 },
    // NEW: Schedule update from telemetry
    ScheduleUpdate {
        mode: String,
        cmd_in: u64,
        cmd_dur: u64,
    },
}

/// Handles valve control via Arduino/BLE
#[derive(Debug)]
pub struct ValveControlHandler {
    current_state: bool,  // true = open, false = closed
    target_state: bool,   // What state we're trying to achieve
    status_message: Option<String>,
    message_bus: MessageBus,
    device_id: String,
    message_tx: mpsc::UnboundedSender<HandlerMessage>,
    message_rx: mpsc::UnboundedReceiver<HandlerMessage>,

    // NEW: Schedule tracking
    current_mode: Option<String>,
    cmd_window_opens_in: Option<u64>,
    cmd_window_duration: Option<u64>,

    // NEW: Command queueing support
    discovery_manager: Option<std::sync::Arc<crate::util::io::discovery::DiscoveryManager>>,
}

impl ValveControlHandler {
    pub fn new(
        message_bus: MessageBus,
        device_id: String,
        bus_topic: String,
        discovery_manager: Option<std::sync::Arc<crate::util::io::discovery::DiscoveryManager>>,
    ) -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        let mut handler = Self {
            current_state: false,
            target_state: false,
            status_message: None,
            message_bus: message_bus.clone(),
            device_id: device_id.clone(),
            message_tx,
            message_rx,
            current_mode: None,
            cmd_window_opens_in: None,
            cmd_window_duration: None,
            discovery_manager,
        };

        handler.start_telemetry_listener(bus_topic);

        handler
    }

    fn start_telemetry_listener(&mut self, bus_topic: String) {
        let tx = self.message_tx.clone();
        let bus = self.message_bus.clone();
        let device_id = self.device_id.clone();

        tokio::spawn(async move {
            log_info!("🚰 Starting valve telemetry listener for topic: {}", bus_topic);
            let mut receiver = bus.subscribe(bus_topic).await;

            while let Some(msg) = receiver.recv().await {
                log_debug!("Received valve telemetry message");

                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&msg.payload) {
                    log_debug!("Received message payload: {}", msg.payload);

                    // ========================================
                    // NEW: Extract schedule metadata FIRST
                    // ========================================
                    if let Some(metadata) = extract_schedule_metadata(&data) {
                        log_info!("📅 [{}] Valve schedule metadata found!", device_id);

                        let mode = metadata.get("mode")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();

                        let cmd_in = metadata.get("cmd_in")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);

                        let cmd_dur = metadata.get("cmd_dur")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(10);

                        log_info!("📅 [{}] Mode: {}, CMD window in: {}s", device_id, mode, cmd_in);

                        let _ = tx.send(HandlerMessage::ScheduleUpdate {
                            mode,
                            cmd_in,
                            cmd_dur,
                        });
                    }

                    // ========================================
                    // EXISTING: Extract valve state
                    // ========================================
                    // SSP Compact Format: {"p":"ssp/1.0","t":"tel","i":"v01","d":{"a":1,"b":95,"c":123}}
                    if let Some(d) = data.get("d").and_then(|v| v.as_object()) {
                        let valve_open = d.get("a")
                            .and_then(|v| v.as_i64())
                            .map(|i| i != 0)
                            .unwrap_or(false);

                        let sensor_value = d.get("b")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);

                        log_info!("🚰 Valve telemetry: open={}, position={}%", valve_open, sensor_value);

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
                    if valve_open == self.target_state {
                        self.status_message = None;
                    }
                }
                // NEW: Handle schedule updates
                HandlerMessage::ScheduleUpdate { mode, cmd_in, cmd_dur } => {
                    self.current_mode = Some(mode.clone());
                    self.cmd_window_opens_in = Some(cmd_in);
                    self.cmd_window_duration = Some(cmd_dur);

                    log_info!("📅 [{}] Valve schedule updated: mode={}, window_in={}s",
                        self.device_id, mode, cmd_in);
                }
            }
        }
    }

    fn toggle_valve(&mut self) {
        let new_state = !self.current_state;
        self.target_state = new_state;

        let action = if new_state { "open" } else { "close" };

        // NEW: Use scheduler if available, otherwise fall back to direct publish
        if let Some(discovery) = &self.discovery_manager {
            self.status_message = Some(format!("⏳ Queueing {} command...", action));

            log_info!("🚰 Queueing valve {} command via scheduler", action);

            let discovery_clone = discovery.clone();
            let device_id = self.device_id.clone();
            let tx = self.message_tx.clone();

            tokio::spawn(async move {
                let payload = serde_json::json!({
                    "action": if new_state { "open" } else { "close" }
                });

                match discovery_clone.send_command(
                    device_id.clone(),
                    "valve_control",
                    Some(payload),
                    CommandPriority::High,  // Valve control is HIGH priority
                ).await {
                    Ok(_) => {
                        log_info!("✓ Valve command queued");
                        let _ = tx.send(HandlerMessage::StatusUpdate(
                            "Command queued, will send during CMD window".to_string()
                        ));
                    }
                    Err(e) => {
                        log_error!("Failed to queue valve command: {}", e);
                        let _ = tx.send(HandlerMessage::StatusUpdate(
                            format!("❌ Failed to queue command: {}", e)
                        ));
                    }
                }
            });
        } else {
            // FALLBACK: Direct publish (old behavior)
            self.status_message = Some(format!("⏳ Sending {} command...", action));

            log_info!("🚰 Sending valve {} command directly (no scheduler)", action);

            let tx = self.message_tx.clone();
            let bus = self.message_bus.clone();
            let device_id = self.device_id.clone();

            tokio::spawn(async move {
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
                            "Command sent, waiting for response...".to_string()
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

    fn is_in_cmd_window(&self) -> bool {
        self.current_mode.as_deref() == Some("cmd")
    }
}

impl ModuleHandler for ValveControlHandler {
    fn handle_key(&mut self, key_code: KeyCode, _module: &mut Module) -> Option<AppEvent> {
        match key_code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.toggle_valve();
                None
            }
            KeyCode::Char('o') => {
                if !self.current_state {
                    self.toggle_valve();
                }
                None
            }
            KeyCode::Char('c') => {
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
        self.process_messages();

        module.config.bindings.insert(
            "state".to_string(),
            serde_json::json!(self.current_state),
        );

        if let Some(status) = &self.status_message {
            module.config.bindings.insert(
                "status_message".to_string(),
                serde_json::json!(status),
            );
        }

        let description = if self.current_state {
            "Valve is OPEN - Flow active"
        } else {
            "Valve is CLOSED - Flow stopped"
        };

        module.config.bindings.insert(
            "description".to_string(),
            serde_json::json!(description),
        );

        // ========================================
        // NEW: Export schedule information
        // ========================================
        if let Some(mode) = &self.current_mode {
            module.config.bindings.insert(
                "device_mode".to_string(),
                serde_json::json!(mode),
            );
        }

        if let Some(cmd_in) = self.cmd_window_opens_in {
            module.config.bindings.insert(
                "cmd_window_in".to_string(),
                serde_json::json!(cmd_in),
            );
        }

        let cmd_status = if self.is_in_cmd_window() {
            "🟢 CMD WINDOW OPEN".to_string()
        } else if let Some(cmd_in) = self.cmd_window_opens_in {
            if cmd_in == 0 {
                "🟡 CMD WINDOW OPENING".to_string()
            } else if cmd_in < 10 {
                format!("🟡 CMD in {}s", cmd_in)
            } else {
                format!("⏰ CMD in {}s", cmd_in)
            }
        } else {
            "⚪ Unknown".to_string()
        };

        module.config.bindings.insert(
            "cmd_window_status".to_string(),
            serde_json::json!(cmd_status),
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
