use crate::log_info;

use super::{HandlerMessage, ValveControlHandler};

impl ValveControlHandler {
    pub(in crate::module) fn process_messages(&mut self) {
        while let Ok(msg) = self.message_rx.try_recv() {
            match msg {
                HandlerMessage::StateChanged(new_state) => {
                    self.current_state = new_state;
                    self.status_message = Some(
                        if new_state { "✓ Valve opened".to_string() } else { "✓ Valve closed".to_string() }
                    );
                }
                HandlerMessage::StatusUpdate(status) => {
                    self.status_message = Some(status);
                }
                HandlerMessage::TelemetryReceived { valve_open, sensor_value } => {
                    self.current_state = valve_open;
                    if valve_open == self.target_state {
                        self.status_message = None;
                    }
                }
                HandlerMessage::ScheduleUpdate { mode, cmd_in, cmd_dur } => {
                    self.current_mode = Some(mode.clone());
                    self.cmd_window_opens_in = Some(cmd_in);
                    self.cmd_window_duration = Some(cmd_dur);

                    log_info!("📅 [{}] Valve schedule updated: mode={}, window_in={}s",
                        self.device_id, mode, cmd_in);
                }
            }
        }
    }
}
