use crate::{log_debug, log_info};

use crate::util::io::ble_scheduler::extract_schedule_metadata;

use super::{HandlerMessage, ValveControlHandler};

impl ValveControlHandler {
    pub(in crate::module) fn start_telemetry_listener(&mut self, bus_topic: String) {
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
}
