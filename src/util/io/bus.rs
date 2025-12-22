use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use std::collections::HashMap;
use color_eyre::Result;
use crate::log_debug;

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

#[derive(Debug, Clone)]
pub struct MessageBus {
    sender: BusSender,
    // Use Arc<RwLock> so the bus can be cloned and subscribers can be modified
    subscribers: Arc<RwLock<HashMap<String, Vec<BusSender>>>>,
}

impl MessageBus {
    pub fn new() -> (Self, BusReceiver) {
        let (sender, receiver) = mpsc::unbounded_channel();

        (
            Self {
                sender,
                subscribers: Arc::new(RwLock::new(HashMap::new())),
            },
            receiver,
        )
    }

    pub async fn publish(&self, message: BusMessage) -> Result<()> {
        // Send to main receiver
        self.sender.send(message.clone())?;

        // Send to topic subscribers
        let subscribers = self.subscribers.read().await;
        if let Some(subs) = subscribers.get(&message.topic) {
            for subscriber in subs {
                let _ = subscriber.send(message.clone());
            }
        }

        Ok(())
    }

    pub async fn subscribe(&self, topic: String) -> BusReceiver {
        let (sender, receiver) = mpsc::unbounded_channel();

        let mut subscribers = self.subscribers.write().await;
        subscribers
            .entry(topic)
            .or_insert_with(Vec::new)
            .push(sender);

        receiver
    }

    /// Publish an app event to the bus with a standardized topic format
    pub async fn publish_app_event(&self, event_name: &str, payload: &str) -> Result<()> {
        let topic = format!("app.event.{}", event_name);
        log_debug!("topic: {}", &topic);
        log_debug!("payload: {}", &payload);
        let message = BusMessage::new(topic, payload.to_string(), "survon_tui".to_string());
        self.publish(message).await
    }

    pub fn send_command(&self, topic: String, command: String, source: String) -> Result<()> {
        let message = BusMessage::new(topic, command, source);
        // This needs to be sync, so just send to main receiver
        self.sender.send(message)?;
        Ok(())
    }

    pub fn get_sender(&self) -> BusSender {
        self.sender.clone()
    }
}

/// Usage:
/// subscribe_app_events!(
///     self.event_receivers,
///     &message_bus,
///     ["increment", "decrement", "select"]
/// ).await;
#[macro_export]
macro_rules! subscribe_app_events {
    ($receivers:expr, $bus:expr, [$($event:literal),* $(,)?]) => {{
        async {
            $(
                let receiver = $bus.subscribe(format!("app.event.{}", $event)).await;
                $receivers.push(receiver);
            )*
        }
    }};
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
