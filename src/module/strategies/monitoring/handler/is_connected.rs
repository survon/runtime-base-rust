use crate::module::strategies::monitoring::handler::{
    CONNECTION_TIMEOUT,
    MonitoringHandler
};

impl MonitoringHandler {
    pub(in crate::module) fn is_connected(&self) -> bool {
        if let Some(last_update) = self.last_update {
            last_update.elapsed() < CONNECTION_TIMEOUT
        } else {
            false
        }
    }
}
