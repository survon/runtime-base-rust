use std::time::Instant;

use crate::{
    log_debug, log_error, log_info, log_warn,
    util::io::ble_scheduler::extract_schedule_metadata
};
use crate::module::strategies::monitoring::handler::{
    HandlerMessage,
    MonitoringHandler,
};

impl MonitoringHandler {
    pub(in crate::module) fn start_telemetry_listener(&mut self, bus_topic: String) {
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
}
