use std::sync::Arc;
use dashmap::DashMap;
use serde_json::{Value,json};

#[derive(Debug, Clone)]
pub struct Event {
    pub category: String,
    pub event_type: String,
    pub payload: Value,
    pub metadata: EventMetadata,
}

#[derive(Debug, Clone)]
pub struct EventMetadata {
    pub source: String,
    pub timestamp: u64,
}

pub struct EventBus {
    subscribers: DashMap<String, Vec<Box<dyn Fn(&Event) + Send + Sync>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: DashMap::new(),
        }
    }

    pub fn subscribe<F>(&self, event_type: &str, callback: F)
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        self.subscribers
            .entry(event_type.to_string())
            .or_default()
            .push(Box::new(callback));
    }

    pub fn publish(&self, event: &Event) {
        if let Some(callbacks) = self.subscribers.get(&event.event_type) {
            for callback in callbacks.iter() {
                callback(event);
            }
        }
    }
}

#[tokio::test]
async fn test_event_bus_publish_and_subscribe() {
    let event_bus = EventBus::new();
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

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
    }
}

