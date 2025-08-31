use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::Serialize;
use tokio::sync::mpsc;
use crate::message::MessageEnvelope;

/// Simple message bus for publish/subscribe messaging
pub struct MessageBus {
    /// Topic subscribers
    subscribers: Arc<RwLock<HashMap<String, Vec<mpsc::UnboundedSender<MessageEnvelope>>>>>,
    /// Message statistics
    message_count: Arc<std::sync::atomic::AtomicU64>,
}

impl MessageBus {
    /// Create a new message bus
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            message_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Publish a message to a topic
    pub fn publish<T: Serialize>(&self, topic: &str, message: &T) {
        let payload = match bincode::serialize(message) {
            Ok(data) => data,
            Err(_) => return, // Skip if serialization fails
        };

        let envelope = MessageEnvelope::new(
            "execution_engine".to_string(),
            topic.to_string(),
            payload,
        );

        let subscribers = self.subscribers.read().unwrap();
        if let Some(senders) = subscribers.get(topic) {
            for sender in senders {
                let _ = sender.send(envelope.clone());
            }
        }

        self.message_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Subscribe to a topic
    pub fn subscribe(&self, topic: &str) -> mpsc::UnboundedReceiver<MessageEnvelope> {
        let (tx, rx) = mpsc::unbounded_channel();
        
        let mut subscribers = self.subscribers.write().unwrap();
        subscribers.entry(topic.to_string()).or_insert_with(Vec::new).push(tx);
        
        rx
    }

    /// Get message count
    pub fn get_message_count(&self) -> u64 {
        self.message_count.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new()
    }
}
