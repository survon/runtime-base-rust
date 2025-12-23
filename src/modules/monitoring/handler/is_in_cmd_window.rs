use super::MonitoringHandler;

impl MonitoringHandler {
    pub(super) fn is_in_cmd_window(&self) -> bool {
        self.current_mode.as_deref() == Some("cmd")
    }
}
