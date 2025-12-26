use super::{HandlerMessage, OverseerHandler};

impl OverseerHandler {
    pub(in crate::module) fn handle_trust_device(&mut self) {
        if self.selected_index < self.pending_devices.len() {
            let (mac, name, _rssi) = &self.pending_devices[self.selected_index];
            let mac_clone = mac.clone();
            let name_clone = name.clone();
            let tx = self.message_tx.clone();

            self.status_message = Some(format!("⏳ Trusting {}...", name_clone));

            if let Some(discovery) = &self.discovery_manager {
                let discovery_clone = discovery.clone();
                tokio::spawn(async move {
                    let _ = tx.send(HandlerMessage::OperationInProgress(format!(
                        "Connecting to {}...",
                        name_clone
                    )));

                    match discovery_clone.trust_device(mac_clone.clone()).await {
                        Ok(_) => {
                            let _ = tx.send(HandlerMessage::DeviceTrusted(mac_clone));
                        }
                        Err(e) => {
                            let _ = tx.send(HandlerMessage::OperationInProgress(format!(
                                "❌ Failed to trust device: {}",
                                e
                            )));
                        }
                    }
                });
            }
        }
    }
}
