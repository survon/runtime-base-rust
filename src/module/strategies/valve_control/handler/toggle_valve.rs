use crate::{
    log_error,
    log_info,
    util::io::{
        ble_scheduler::CommandPriority,
        bus::BusMessage,
    }
};

use super::{HandlerMessage, ValveControlHandler};

impl ValveControlHandler {
    pub(in crate::module) fn toggle_valve(&mut self) {
        let new_state = !self.current_state;
        self.target_state = new_state;

        let action = if new_state { "open" } else { "close" };

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
}
