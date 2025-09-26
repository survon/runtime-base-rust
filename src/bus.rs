use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use color_eyre::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusMessage {
    pub topic: String,
    pub payload: String,
    pub source: String,
    pub timestamp: u64,
}

impl BusMessage {
    pub fn new(topic: String, payload: String, source: String) -> Self {
        Self {
            topic,
            payload,
            source,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

pub type BusReceiver = mpsc::UnboundedReceiver<BusMessage>;
pub type BusSender = mpsc::UnboundedSender<BusMessage>;

#[derive(Debug)]
pub struct MessageBus {
    sender: BusSender,
    subscribers: HashMap<String, Vec<BusSender>>,
}

impl MessageBus {
    pub fn new() -> (Self, BusReceiver) {
        let (sender, receiver) = mpsc::unbounded_channel();

        (
            Self {
                sender,
                subscribers: HashMap::new(),
            },
            receiver,
        )
    }

    pub fn publish(&self, message: BusMessage) -> Result<()> {
        // Send to main receiver
        self.sender.send(message.clone())?;

        // Send to topic subscribers
        if let Some(subscribers) = self.subscribers.get(&message.topic) {
            for subscriber in subscribers {
                let _ = subscriber.send(message.clone());
            }
        }

        Ok(())
    }

    pub fn subscribe(&mut self, topic: String) -> BusReceiver {
        let (sender, receiver) = mpsc::unbounded_channel();

        self.subscribers
            .entry(topic)
            .or_insert_with(Vec::new)
            .push(sender);

        receiver
    }

    pub fn send_command(&self, topic: String, command: String, source: String) -> Result<()> {
        let message = BusMessage::new(topic, command, source);
        self.publish(message)
    }

    pub fn get_sender(&self) -> BusSender {
        self.sender.clone()
    }
}

// Helper for creating common message types
pub mod messages {
    use super::BusMessage;

    pub fn gate_close(source: String) -> BusMessage {
        BusMessage::new("com_input".to_string(), "close_gate".to_string(), source)
    }

    pub fn monitoring_update(topic: String, value: String, source: String) -> BusMessage {
        BusMessage::new(topic, value, source)
    }

    pub fn llm_query(query: String, source: String) -> BusMessage {
        BusMessage::new("llm_response".to_string(), query, source)
    }
}
