use crate::module::Module;
use crate::module::strategies::valve_control::handler::ValveControlHandler;

impl ValveControlHandler {
    pub(in crate::module) fn _update_bindings(&mut self, module: &mut Module) {
        self.process_messages();

        module.config.bindings.insert(
            "state".to_string(),
            serde_json::json!(self.current_state),
        );

        if let Some(status) = &self.status_message {
            module.config.bindings.insert(
                "status_message".to_string(),
                serde_json::json!(status),
            );
        }

        let description = if self.current_state {
            "Valve is OPEN - Flow active"
        } else {
            "Valve is CLOSED - Flow stopped"
        };

        module.config.bindings.insert(
            "description".to_string(),
            serde_json::json!(description),
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
    }
}
