use crate::{
    log_info,
    modules::monitoring::handler::{
        HandlerMessage,
        MAX_HISTORY,
        MonitoringHandler,
    }
};

impl MonitoringHandler {
    pub(super) fn process_messages(&mut self) {
        let mut message_count = 0;
        while let Ok(msg) = self.message_rx.try_recv() {
            message_count += 1;
            match msg {
                HandlerMessage::TelemetryReceived { value_a, value_b, value_c, timestamp } => {
                    self.current_values = (value_a, value_b, value_c);
                    self.last_update = Some(timestamp);

                    self.history.push_back((value_a, value_b, value_c));

                    while self.history.len() > MAX_HISTORY {
                        self.history.pop_front();
                    }

                    log_info!("🟢 Updated values for {}: a={}, b={}, c={}, history_size={}",
                        self.device_id, value_a, value_b, value_c, self.history.len());
                }

                // NEW: Handle schedule updates
                HandlerMessage::ScheduleUpdate { mode, cmd_in, cmd_dur } => {
                    self.current_mode = Some(mode.clone());
                    self.cmd_window_opens_in = Some(cmd_in);
                    self.cmd_window_duration = Some(cmd_dur);

                    log_info!("📅 [{}] Schedule updated: mode={}, window_in={}s",
                        self.device_id, mode, cmd_in);
                }
            }
        }

        if message_count > 0 {
            log_info!("🟢 Processed {} messages for {}", message_count, self.device_id);
        }
    }
}
