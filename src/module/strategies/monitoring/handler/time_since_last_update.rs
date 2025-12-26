use std::time::Duration;

use super::MonitoringHandler;

impl MonitoringHandler {
    pub(in crate::module) fn time_since_last_update(&self) -> Option<Duration> {
        self.last_update.map(|t| t.elapsed())
    }
}
