//! High-performance message passing system for AlphaForge

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use dashmap::DashMap;
use tokio::sync::{mpsc, oneshot};
use serde::{Serialize, Deserialize};
use tracing::{debug, warn};

use crate::time::UnixNanos;
use crate::uuid::UUID4;
use crate::error::{AlphaForgeError, Result};

/// Message envelope for all system messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    pub id: UUID4,
    pub timestamp: UnixNanos,
    pub sender: String,
    pub recipient: Option<String>,
    pub correlation_id: Option<UUID4>,
    pub message_type: String,
    pub payload: Vec<u8>,
}

impl MessageEnvelope {
    /// Create a new message envelope
    pub fn new(
        sender: String,
        message_type: String, 
        payload: Vec<u8>,
    ) -> Self {
        Self {
            id: UUID4::new(),
            timestamp: crate::time::unix_nanos_now(),
            sender,
            recipient: None,
            correlation_id: None,
            message_type,
            payload,
        }
    }
    
    /// Create a response message
    pub fn create_response(
        &self,
        sender: String,
        message_type: String,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            id: UUID4::new(),
            timestamp: crate::time::unix_nanos_now(),
            sender,
            recipient: Some(self.sender.clone()),
            correlation_id: Some(self.id),
            message_type,
            payload,
        }
    }
}

/// Message bus patterns
#[derive(Debug, Clone)]
pub enum MessagePattern {
    /// Publish-Subscribe pattern
    PubSub { topic: String },
    /// Request-Response pattern
    RequestResponse { target: String },
    /// Point-to-Point messaging
    PointToPoint { target: String },
}

/// High-performance message bus implementation
pub struct MessageBus {
    // Publish-Subscribe subscriptions
    pub_sub_subs: Arc<DashMap<String, Vec<mpsc::UnboundedSender<MessageEnvelope>>>>,
    
    // Request-Response handlers
    req_resp_handlers: Arc<DashMap<String, mpsc::UnboundedSender<(MessageEnvelope, oneshot::Sender<MessageEnvelope>)>>>,
    
    // Point-to-Point endpoints
    p2p_endpoints: Arc<DashMap<String, mpsc::UnboundedSender<MessageEnvelope>>>,
    
    // Message statistics
    stats: Arc<MessageBusStats>,
}

impl Clone for MessageBus {
    fn clone(&self) -> Self {
        Self {
            pub_sub_subs: self.pub_sub_subs.clone(),
            req_resp_handlers: self.req_resp_handlers.clone(),
            p2p_endpoints: self.p2p_endpoints.clone(),
            stats: self.stats.clone(),
        }
    }
}

impl MessageBus {
    /// Create a new message bus
    pub fn new() -> Self {
        Self {
            pub_sub_subs: Arc::new(DashMap::new()),
            req_resp_handlers: Arc::new(DashMap::new()),
            p2p_endpoints: Arc::new(DashMap::new()),
            stats: Arc::new(MessageBusStats::default()),
        }
    }
    
    /// Subscribe to a topic (Pub/Sub pattern)
    pub fn subscribe(&self, topic: String) -> mpsc::UnboundedReceiver<MessageEnvelope> {
        let (tx, rx) = mpsc::unbounded_channel();
        
        self.pub_sub_subs
            .entry(topic.clone())
            .or_insert_with(Vec::new)
            .push(tx);
            
        debug!("Subscribed to topic: {}", topic);
        rx
    }
    
    /// Publish a message to a topic (Pub/Sub pattern) 
    pub async fn publish(&self, topic: String, envelope: MessageEnvelope) -> Result<()> {
        let start = std::time::Instant::now();
        
        if let Some(subscribers) = self.pub_sub_subs.get(&topic) {
            let mut delivered = 0;
            let mut failed = 0;
            
            for subscriber in subscribers.value() {
                match subscriber.send(envelope.clone()) {
                    Ok(()) => delivered += 1,
                    Err(_) => failed += 1, // Receiver dropped
                }
            }
            
            if failed > 0 {
                warn!("Failed to deliver to {} subscribers for topic: {}", failed, topic);
            }
            
            self.stats.record_publish(delivered, start.elapsed());
        }
        
        Ok(())
    }
    
    /// Send a request and wait for response (Request/Response pattern)
    pub async fn request(
        &self,
        target: String,
        envelope: MessageEnvelope,
        timeout: std::time::Duration,
    ) -> Result<MessageEnvelope> {
        let (response_tx, response_rx) = oneshot::channel();
        
        if let Some(handler) = self.req_resp_handlers.get(&target) {
            handler.send((envelope, response_tx))
                .map_err(|_| AlphaForgeError::MessageBus { 
                    msg: format!("No handler available for target: {}", target)
                })?;
                
            let response = tokio::time::timeout(timeout, response_rx)
                .await
                .map_err(|_| AlphaForgeError::MessageBus { 
                    msg: "Request timeout".to_string()
                })?
                .map_err(|_| AlphaForgeError::MessageBus { 
                    msg: "Response channel closed".to_string()
                })?;
                
            Ok(response)
        } else {
            Err(AlphaForgeError::MessageBus { 
                msg: format!("No handler registered for target: {}", target)
            })
        }
    }
    
    /// Register a request handler (Request/Response pattern)
    pub fn register_handler(
        &self,
        target: String,
    ) -> mpsc::UnboundedReceiver<(MessageEnvelope, oneshot::Sender<MessageEnvelope>)> {
        let (tx, rx) = mpsc::unbounded_channel();
        
        self.req_resp_handlers.insert(target.clone(), tx);
        debug!("Registered handler for target: {}", target);
        
        rx
    }
    
    /// Send point-to-point message
    pub async fn send(&self, target: String, envelope: MessageEnvelope) -> Result<()> {
        if let Some(endpoint) = self.p2p_endpoints.get(&target) {
            endpoint.send(envelope)
                .map_err(|_| AlphaForgeError::MessageBus { 
                    msg: format!("Failed to send to target: {}", target)
                })?;
            Ok(())
        } else {
            Err(AlphaForgeError::MessageBus { 
                msg: format!("No endpoint registered for target: {}", target)
            })
        }
    }
    
    /// Register point-to-point endpoint
    pub fn register_endpoint(&self, target: String) -> mpsc::UnboundedReceiver<MessageEnvelope> {
        let (tx, rx) = mpsc::unbounded_channel();
        
        self.p2p_endpoints.insert(target.clone(), tx);
        debug!("Registered endpoint: {}", target);
        
        rx
    }
    
    /// Get message bus statistics
    pub fn stats(&self) -> MessageBusStats {
        self.stats.snapshot()
    }
    
    /// Pattern matching for wildcard subscriptions
    pub fn subscribe_pattern(&self, pattern: String) -> mpsc::UnboundedReceiver<MessageEnvelope> {
        // TODO: Implement wildcard pattern matching
        // For now, exact match only
        self.subscribe(pattern)
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Message bus performance statistics
#[derive(Debug, Default)]
pub struct MessageBusStats {
    pub total_messages_sent: AtomicU64,
    pub total_messages_delivered: AtomicU64,
    pub total_publish_time_nanos: AtomicU64,
    pub publish_count: AtomicU64,
}

impl MessageBusStats {
    /// Record a publish operation
    pub fn record_publish(&self, delivered: usize, elapsed: std::time::Duration) {
        self.total_messages_delivered.fetch_add(delivered as u64, Ordering::Relaxed);
        self.total_publish_time_nanos.fetch_add(elapsed.as_nanos() as u64, Ordering::Relaxed);
        self.publish_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get average publish latency in nanoseconds
    pub fn avg_publish_latency_nanos(&self) -> f64 {
        let total_time = self.total_publish_time_nanos.load(Ordering::Relaxed);
        let count = self.publish_count.load(Ordering::Relaxed);
        
        if count > 0 {
            total_time as f64 / count as f64
        } else {
            0.0
        }
    }
    
    /// Get messages per second throughput
    pub fn messages_per_second(&self, duration_secs: f64) -> f64 {
        let delivered = self.total_messages_delivered.load(Ordering::Relaxed);
        delivered as f64 / duration_secs
    }
    
    /// Get snapshot of current statistics
    pub fn snapshot(&self) -> Self {
        Self {
            total_messages_sent: std::sync::atomic::AtomicU64::new(
                self.total_messages_sent.load(Ordering::Relaxed)
            ),
            total_messages_delivered: std::sync::atomic::AtomicU64::new(
                self.total_messages_delivered.load(Ordering::Relaxed)
            ),
            total_publish_time_nanos: std::sync::atomic::AtomicU64::new(
                self.total_publish_time_nanos.load(Ordering::Relaxed)
            ),
            publish_count: std::sync::atomic::AtomicU64::new(
                self.publish_count.load(Ordering::Relaxed)
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};
    
    #[tokio::test]
    async fn test_pub_sub_messaging() {
        let bus = MessageBus::new();
        let mut rx = bus.subscribe("test.topic".to_string());
        
        let envelope = MessageEnvelope::new(
            "test_sender".to_string(),
            "TestMessage".to_string(),
            b"test payload".to_vec(),
        );
        
        bus.publish("test.topic".to_string(), envelope.clone()).await.unwrap();
        
        let received = rx.recv().await.unwrap();
        assert_eq!(received.message_type, "TestMessage");
        assert_eq!(received.payload, b"test payload");
    }
    
    #[tokio::test]
    async fn test_request_response_messaging() {
        let bus = MessageBus::new();
        let mut handler_rx = bus.register_handler("test.service".to_string());
        
        // Spawn handler task
        let bus_clone = bus.clone();
        tokio::spawn(async move {
            if let Some((request, response_tx)) = handler_rx.recv().await {
                let response = MessageEnvelope::new(
                    "test.service".to_string(),
                    "TestResponse".to_string(),
                    b"response payload".to_vec(),
                );
                let _ = response_tx.send(response);
            }
        });
        
        let request = MessageEnvelope::new(
            "test_client".to_string(),
            "TestRequest".to_string(),
            b"request payload".to_vec(),
        );
        
        let response = bus.request(
            "test.service".to_string(),
            request,
            Duration::from_secs(1),
        ).await.unwrap();
        
        assert_eq!(response.message_type, "TestResponse");
        assert_eq!(response.payload, b"response payload");
    }
    
    #[tokio::test]
    async fn test_message_bus_performance() {
        let bus = MessageBus::new();
        let _rx = bus.subscribe("perf.test".to_string());
        
        let start = std::time::Instant::now();
        let message_count = 10000;
        
        for i in 0..message_count {
            let envelope = MessageEnvelope::new(
                "perf_sender".to_string(),
                "PerfTest".to_string(),
                format!("message_{}", i).into_bytes(),
            );
            
            bus.publish("perf.test".to_string(), envelope).await.unwrap();
        }
        
        let elapsed = start.elapsed();
        let throughput = message_count as f64 / elapsed.as_secs_f64();
        
        println!("Message bus throughput: {:.0} msgs/sec", throughput);
        assert!(throughput > 100_000.0); // Should handle >100k msgs/sec
    }
}
