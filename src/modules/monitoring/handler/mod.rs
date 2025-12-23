mod trait_module_handler;
mod new;
mod start_telemetry_listener;
mod process_messages;
mod is_connected;
mod time_since_last_update;
mod is_in_cmd_window;
mod update_bindings;

use std::{
    any::Any,
    collections::VecDeque,
    time::{Duration, Instant},
};

use tokio::sync::mpsc;

use crate::{
    modules::module_handler::ModuleHandler,
    util::io::bus::MessageBus,
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
    current_mode: Option<String>,        // "data" or "cmd"
    cmd_window_opens_in: Option<u64>,    // seconds until CMD window
    cmd_window_duration: Option<u64>,    // duration of CMD window
}
