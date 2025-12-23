use crate::modules::monitoring::handler::{
    CONNECTION_TIMEOUT,
    MonitoringHandler
};

impl MonitoringHandler {
    pub(super) fn is_connected(&self) -> bool {
        if let Some(last_update) = self.last_update {
            last_update.elapsed() < CONNECTION_TIMEOUT
        } else {
            false
        }
    }
}
