// src/modules/monitoring/handler.rs - REFACTORED for Schedule Metadata
use color_eyre::Result;
use ratatui::crossterm::event::KeyCode;
use std::any::Any;
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use tokio::sync::mpsc;
use crate::{log_info, log_error, log_debug, log_warn};
use crate::modules::{Module, module_handler::ModuleHandler};
use crate::util::io::{
    event::AppEvent,
    bus::{MessageBus},
    ble_scheduler::extract_schedule_metadata,
};

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_HISTORY: usize = 50;

#[derive(Debug, Clone)]
enum HandlerMessage {
    TelemetryReceived {
        value_a: f64,
        value_b: f64,
        value_c: i64,
        timestamp: Instant,
    },
    ScheduleUpdate {
        mode: String,
        cmd_in: u64,
        cmd_dur: u64,
    },
}

/// Handles monitoring sensors (gauges, meters, etc.)
/// Works with multiple instances - each monitors its own bus_topic
#[derive(Debug)]
pub struct MonitoringHandler {
    device_id: String,
    last_update: Option<Instant>,
    current_values: (f64, f64, i64), // (a, b, c)
    history: VecDeque<(f64, f64, i64)>,
    message_bus: MessageBus,
    message_tx: mpsc::UnboundedSender<HandlerMessage>,
    message_rx: mpsc::UnboundedReceiver<HandlerMessage>,

    // NEW: Schedule tracking
    current_mode: Option<String>,        // "data" or "cmd"
    cmd_window_opens_in: Option<u64>,    // seconds until CMD window
    cmd_window_duration: Option<u64>,    // duration of CMD window
}

impl MonitoringHandler {
    pub fn new(message_bus: MessageBus, device_id: String, bus_topic: String) -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        let mut handler = Self {
            device_id: device_id.clone(),
            last_update: None,
            current_values: (0.0, 0.0, 0),
            history: VecDeque::new(),
            message_bus: message_bus.clone(),
            message_tx,
            message_rx,
            current_mode: None,
            cmd_window_opens_in: None,
            cmd_window_duration: None,
        };

        handler.start_telemetry_listener(bus_topic);

        handler
    }

    fn start_telemetry_listener(&mut self, bus_topic: String) {
        let tx = self.message_tx.clone();
        let bus = self.message_bus.clone();
        let device_id = self.device_id.clone();

        tokio::spawn(async move {
            log_info!("📻 Starting monitoring telemetry listener for device: {} on topic: {}", device_id, bus_topic);
            let mut receiver = bus.subscribe(bus_topic.clone()).await;
            log_info!("📻 Subscribed to topic: {}", bus_topic);

            while let Some(msg) = receiver.recv().await {
                log_info!("📻 [{}] Received message on topic {}", device_id, bus_topic);
                log_debug!("Raw payload: {}", msg.payload);

                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&msg.payload) {
                    if let Some(metadata) = extract_schedule_metadata(&data) {
                        log_info!("📅 [{}] Schedule metadata found!", device_id);

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

                        log_info!("📅 [{}] Mode: {}, CMD window in: {}s, Duration: {}s",
                            device_id, mode, cmd_in, cmd_dur);

                        let _ = tx.send(HandlerMessage::ScheduleUpdate {
                            mode,
                            cmd_in,
                            cmd_dur,
                        });
                    }

                    let values = if let Some(d) = data.get("d").and_then(|v| v.as_object()) {
                        // Full SSP format with "d" object
                        log_debug!("Parsing SSP format with 'd' object");
                        Some((
                            d.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            d.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            d.get("c").and_then(|v| v.as_i64()).unwrap_or(0)
                        ))
                    } else if data.is_object() {
                        // Direct payload format: {"a":72,"b":45,"c":335}
                        log_debug!("Parsing direct payload format");
                        Some((
                            data.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            data.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            data.get("c").and_then(|v| v.as_i64()).unwrap_or(0)
                        ))
                    } else {
                        log_warn!("📻 [{}] Unrecognized message format", device_id);
                        None
                    };

                    if let Some((value_a, value_b, value_c)) = values {
                        log_info!("📻 Monitoring telemetry [{}]: a={}, b={}, c={}",
                            device_id, value_a, value_b, value_c);

                        let _ = tx.send(HandlerMessage::TelemetryReceived {
                            value_a,
                            value_b,
                            value_c,
                            timestamp: Instant::now(),
                        });
                    }
                } else {
                    log_warn!("📻 [{}] Failed to parse JSON: {}", device_id, msg.payload);
                }
            }

            log_error!("📻 [{}] Telemetry listener ended!", device_id);
        });
    }

    fn process_messages(&mut self) {
        let mut message_count = 0;
        while let Ok(msg) = self.message_rx.try_recv() {
            message_count += 1;
            match msg {
                HandlerMessage::TelemetryReceived { value_a, value_b, value_c, timestamp } => {
                    self.current_values = (value_a, value_b, value_c);
                    self.last_update = Some(timestamp);

                    self.history.push_back((value_a, value_b, value_c));

                    while self.history.len() > MAX_HISTORY {
                        self.history.pop_front();
                    }

                    log_info!("🟢 Updated values for {}: a={}, b={}, c={}, history_size={}",
                        self.device_id, value_a, value_b, value_c, self.history.len());
                }

                // NEW: Handle schedule updates
                HandlerMessage::ScheduleUpdate { mode, cmd_in, cmd_dur } => {
                    self.current_mode = Some(mode.clone());
                    self.cmd_window_opens_in = Some(cmd_in);
                    self.cmd_window_duration = Some(cmd_dur);

                    log_info!("📅 [{}] Schedule updated: mode={}, window_in={}s",
                        self.device_id, mode, cmd_in);
                }
            }
        }

        if message_count > 0 {
            log_info!("🟢 Processed {} messages for {}", message_count, self.device_id);
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

    // NEW: Check if device is in CMD window
    fn is_in_cmd_window(&self) -> bool {
        self.current_mode.as_deref() == Some("cmd")
    }
}

impl ModuleHandler for MonitoringHandler {
    fn handle_key(&mut self, key_code: KeyCode, _module: &mut Module) -> Option<AppEvent> {
        match key_code {
            KeyCode::Char('r') => {
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

        // Export history to bindings for chart templates
        let history_json: Vec<serde_json::Value> = self.history.iter()
            .map(|(a, b, c)| {
                serde_json::json!({
                    "a": a,
                    "b": b,
                    "c": *c
                })
            })
            .collect();

        module.config.bindings.insert(
            "_chart_history".to_string(),
            serde_json::json!(history_json),
        );

        // Add connection status
        let is_connected = self.is_connected();
        module.config.bindings.insert(
            "is_connected".to_string(),
            serde_json::json!(is_connected),
        );

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

        if let Some(cmd_dur) = self.cmd_window_duration {
            module.config.bindings.insert(
                "cmd_window_duration".to_string(),
                serde_json::json!(cmd_dur),
            );
        }

        // User-friendly CMD window status
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
                serde_json::json!(" [Lost Connection]"),
            );

            if self.last_update.is_none() {
                log_warn!("Device {} has never sent telemetry", self.device_id);
            } else {
                log_warn!("Device {} connection lost ({}s since last update)",
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
