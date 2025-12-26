use crate::{
    log_warn,
    module::Module
};
use crate::module::strategies::monitoring::handler::MonitoringHandler;

impl MonitoringHandler {
    pub(in crate::module) fn _update_bindings(&mut self, module: &mut Module) {
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
}
