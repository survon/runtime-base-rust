use std::collections::VecDeque;
use tokio::sync::mpsc;

use crate::util::io::bus::MessageBus;

use super::MonitoringHandler;

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
}
