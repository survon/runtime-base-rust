mod new;
mod start_telemetry_listener;
mod process_messages;
mod toggle_valve;
mod is_in_cmd_window;
mod handle_key;
mod update_bindings;
mod trait_module_handler;

use std::any::Any;
use tokio::sync::mpsc;

use crate::{
    modules::{module_handler::ModuleHandler},
    util::io::bus::{MessageBus},
};

#[derive(Debug, Clone)]
enum HandlerMessage {
    StateChanged(bool),
    StatusUpdate(String),
    TelemetryReceived {
        valve_open: bool,
        sensor_value: f64
    },
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
    current_mode: Option<String>,
    cmd_window_opens_in: Option<u64>,
    cmd_window_duration: Option<u64>,
    discovery_manager: Option<std::sync::Arc<crate::util::io::discovery::DiscoveryManager>>,
}
