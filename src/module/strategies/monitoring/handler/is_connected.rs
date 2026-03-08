use crate::module::strategies::monitoring::handler::{MonitoringHandler, CONNECTION_TIMEOUT};

impl MonitoringHandler {
    pub(in crate::module) fn is_connected(&self) -> bool {
        if let Some(last_update) = self.last_update {
            last_update.elapsed() < CONNECTION_TIMEOUT
        } else {
            false
        }
    }
}
