use crate::module::ModuleManager;

impl ModuleManager {
    /// Poll for incoming events and handle them
    pub fn poll_events(&mut self) {
        // Collect messages first, then process them
        let mut messages = Vec::new();

        for receiver in &mut self.event_receivers {
            while let Ok(message) = receiver.try_recv() {
                messages.push(message);
            }
        }

        // Now process all collected messages
        for message in messages {
            self.handle_event_message(&message);
        }
    }
}
