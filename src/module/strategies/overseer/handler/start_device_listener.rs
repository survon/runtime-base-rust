use super::{HandlerMessage, OverseerHandler};

impl OverseerHandler {
    pub(in crate::module) fn start_device_listener(&mut self) {
        let tx = self.message_tx.clone();
        let bus = self.message_bus.clone();

        tokio::spawn(async move {
            let mut receiver = bus.subscribe("device_discovered".to_string()).await;

            while let Some(msg) = receiver.recv().await {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&msg.payload) {
                    if data
                        .get("requires_trust_decision")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                    {
                        let mac = data["mac_address"].as_str().unwrap_or("").to_string();
                        let name = data["name"].as_str().unwrap_or("Unknown").to_string();
                        let rssi = data["rssi"].as_i64().unwrap_or(0) as i16;

                        let _ = tx.send(HandlerMessage::DeviceDiscovered { mac, name, rssi });
                    }
                }
            }
        });
    }
}
