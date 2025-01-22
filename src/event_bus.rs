use std::sync::Arc;
use dashmap::DashMap;
use serde_json::{Value, json};

/// Represents an event that flows through the EventBus.
#[derive(Debug, Clone)]
pub struct Event {
    pub category: String,
    pub event_type: String,
    pub payload: Value,
    pub metadata: EventMetadata,
}

/// Metadata associated with an Event.
#[derive(Debug, Clone)]
pub struct EventMetadata {
    pub source: String,
    pub timestamp: u64,
}

/// EventBus to manage publish-subscribe interactions between modules.
pub struct EventBus {
    subscribers: DashMap<String, Vec<Box<dyn Fn(&Event) + Send + Sync>>>,
}

impl EventBus {
    /// Creates a new instance of the EventBus.
    pub fn new() -> Self {
        Self {
            subscribers: DashMap::new(),
        }
    }

    /// Subscribe to an event type with a callback.
    ///
    /// # Arguments
    /// - `event_type`: The event type to subscribe to.
    /// - `callback`: A closure that will handle events of the specified type.
    pub fn subscribe<F>(&self, event_type: &str, callback: F)
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        self.subscribers
            .entry(event_type.to_string())
            .or_default()
            .push(Box::new(callback));
    }

    /// Publish an event to all subscribers of the event type.
    ///
    /// # Arguments
    /// - `event`: The event to publish.
    pub fn publish(&self, event: &Event) {
        if let Some(callbacks) = self.subscribers.get(&event.event_type) {
            for callback in callbacks.iter() {
                callback(event);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_event_bus_publish_and_subscribe() {
        let event_bus = EventBus::new();
        let (tx, mut rx) = mpsc::channel(1);

        event_bus.subscribe("example_event", move |event| {
            let tx = tx.clone();
            let event = event.clone();
            tokio::spawn(async move {
                let _ = tx.send(event).await;
            });
        });

        let test_event = Event {
            category: "test".to_string(),
            event_type: "example_event".to_string(),
            payload: json!({ "message": "Hello, EventBus!" }),
            metadata: EventMetadata {
                source: "test_source".to_string(),
                timestamp: 1234567890,
            },
        };

        event_bus.publish(&test_event);

        if let Some(received_event) = rx.recv().await {
            assert_eq!(received_event.category, "test");
            assert_eq!(received_event.event_type, "example_event");
            assert_eq!(
                received_event.payload,
                json!({ "message": "Hello, EventBus!" })
            );
        }
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let event_bus = EventBus::new();
        let (tx1, mut rx1) = mpsc::channel(1);
        let (tx2, mut rx2) = mpsc::channel(1);

        // First subscriber
        event_bus.subscribe("example_event", move |event| {
            let tx = tx1.clone();
            let event = event.clone();
            tokio::spawn(async move {
                let _ = tx.send(event).await;
            });
        });

        // Second subscriber
        event_bus.subscribe("example_event", move |event| {
            let tx = tx2.clone();
            let event = event.clone();
            tokio::spawn(async move {
                let _ = tx.send(event).await;
            });
        });

        let test_event = Event {
            category: "test".to_string(),
            event_type: "example_event".to_string(),
            payload: json!({ "message": "Hello, multiple subscribers!" }),
            metadata: EventMetadata {
                source: "test_source".to_string(),
                timestamp: 1234567890,
            },
        };

        event_bus.publish(&test_event);

        // Verify that both subscribers received the event
        let received_event1 = rx1.recv().await.unwrap();
        let received_event2 = rx2.recv().await.unwrap();

        assert_eq!(received_event1.event_type, "example_event");
        assert_eq!(received_event2.event_type, "example_event");
    }
}
