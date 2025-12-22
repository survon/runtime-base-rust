// src/widgets/messages_window/actor.rs
use super::state::{MessagesState, MessagesIntent, MessagesEvent, MessagesStateMachine};
use crate::util::io::bus::{BusMessage, MessageBus};
use tokio::sync::mpsc;

pub struct MessagesActor {
    state: MessagesState,
    message_bus: MessageBus,
    intent_rx: mpsc::UnboundedReceiver<MessagesIntent>,
}

impl MessagesActor {
    pub fn new(message_bus: MessageBus) -> (Self, mpsc::UnboundedSender<MessagesIntent>) {
        let (intent_tx, intent_rx) = mpsc::unbounded_channel();

        let actor = Self {
            state: MessagesState::default(),
            message_bus,
            intent_rx,
        };

        (actor, intent_tx)
    }

    pub async fn run(mut self) {
        while let Some(intent) = self.intent_rx.recv().await {
            self.process_intent(intent).await;
        }
    }

    async fn process_intent(&mut self, intent: MessagesIntent) {
        // Pure state transition
        let (new_state, events) = MessagesStateMachine::transition(
            self.state.clone(),
            intent,
        );

        // Update internal state
        self.state = new_state;

        // Publish events to message bus
        for event in events {
            self.publish_event(event).await;
        }
    }

    async fn publish_event(&self, event: MessagesEvent) {
        let topic = match &event {
            MessagesEvent::StateChanged(_) => "messages.state",
            MessagesEvent::MessageAdded { .. } => "messages.added",
            MessagesEvent::Scrolled { .. } => "messages.scrolled",
        };

        let payload = serde_json::to_string(&event).unwrap();

        let _ = self.message_bus.publish(BusMessage::new(
            topic.to_string(),
            payload,
            "messages_window".to_string(),
        )).await;
    }
}
