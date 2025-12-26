use tokio::sync::mpsc;

use crate::util::io::{
    bus::MessageBus,
    discovery::DiscoveryManager,
};

use super::ValveControlHandler;

impl ValveControlHandler {
    pub fn new(
        message_bus: MessageBus,
        device_id: String,
        bus_topic: String,
        discovery_manager: Option<std::sync::Arc<DiscoveryManager>>,
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
}
