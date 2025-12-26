use crate::{
    module::ModuleManager,
    util::io::bus::BusMessage,
};

impl ModuleManager {
    pub(super) fn handle_event_message(&mut self, message: &BusMessage) {

        // todo refactor this to be module domain not app domain..
        // todo the app handler for event messages is in app.rs.. that's where we handle RefreshModules
        match message.topic.strip_prefix("app.event.") {
            // Some("refresh_modules") => {
            //     self.refresh_modules(); // async...
            // }
            _ => {}
        }
    }
}
