use crate::log_debug;
use super::WastelandManagerHandler;

impl WastelandManagerHandler {
    pub(super) fn trigger_module_refresh(&self) {
        let bus = self.message_bus.clone();
        tokio::spawn(async move {
            log_debug!("Trigger module refresh");
            let _ = bus.publish_app_event("refresh_modules", "").await;
        });
    }
}
