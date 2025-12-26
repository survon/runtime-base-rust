use crate::log_debug;
use super::OverseerHandler;

impl OverseerHandler {
    pub(in crate::module) fn trigger_module_refresh(&self) {
        let bus = self.message_bus.clone();
        tokio::spawn(async move {
            log_debug!("Trigger module refresh");
            let _ = bus.publish_app_event("refresh_modules", "").await;
        });
    }
}
