use crate::{
    module::ModuleManager,
    util::io::{
        bus::MessageBus,
        get_all_event_message_topics,
    }
};

impl ModuleManager {
    /// Subscribe the module manager to app events it cares about
    pub async fn subscribe_to_events(&mut self, message_bus: &MessageBus) {
        let topics = get_all_event_message_topics();

        for topic in topics {
            let receiver = message_bus.subscribe(topic.to_string()).await;
            self.event_receivers.push(receiver);
        }
    }
}
