use crate::identifiers::{OrderId, InstrumentId, StrategyId, VenueOrderId};
use crate::message_bus::MessageBus;
use crate::generic_cache::{GenericCache, GenericCacheConfig};
use crate::time::{AtomicTime, UnixNanos};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// ============================================================================
// ORDER TYPES AND ENUMS
// ============================================================================

/// Order side enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderSide {
    /// Buy order
    Buy,
    /// Sell order
    Sell,
}

/// Order type enumeration  
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderType {
    /// Market order - immediate execution at best available price
    Market,
    /// Limit order - execution only at specified price or better
    Limit,
    /// Stop order - becomes market order when stop price reached
    Stop,
    /// Stop-limit order - becomes limit order when stop price reached
    StopLimit,
}

/// Order status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderStatus {
    /// Order created but not yet submitted
    Initialized,
    /// Order submitted to exchange
    Submitted,
    /// Order accepted by exchange
    Accepted,
    /// Order partially filled
    PartiallyFilled,
    /// Order completely filled
    Filled,
    /// Order cancelled
    Cancelled,
    /// Order rejected by exchange
    Rejected,
    /// Order expired
    Expired,
}

/// Order time in force enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeInForce {
    /// Good Till Cancelled - remains active until explicitly cancelled
    GTC,
    /// Immediate Or Cancel - execute immediately, cancel remainder
    IOC,
    /// Fill Or Kill - execute completely immediately or cancel entirely
    FOK,
    /// Good Till Date - remains active until specified date
    GTD,
    /// Day order - expires at end of trading day
    DAY,
}

// ============================================================================
// ORDER STRUCTURE
// ============================================================================

/// Core order structure for trading operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// Unique order identifier
    pub order_id: OrderId,
    /// Strategy that created this order
    pub strategy_id: StrategyId,
    /// Instrument being traded
    pub instrument_id: InstrumentId,
    /// Order side (buy/sell)
    pub side: OrderSide,
    /// Order type (market/limit/etc)
    pub order_type: OrderType,
    /// Quantity to trade
    pub quantity: f64,
    /// Price for limit orders (None for market orders)
    pub price: Option<f64>,
    /// Stop price for stop orders
    pub stop_price: Option<f64>,
    /// Time in force
    pub time_in_force: TimeInForce,
    /// Current order status
    pub status: OrderStatus,
    /// Exchange-assigned order ID
    pub venue_order_id: Option<VenueOrderId>,
    /// Quantity filled so far
    pub filled_quantity: f64,
    /// Average fill price
    pub avg_fill_price: Option<f64>,
    /// Order creation timestamp
    pub created_time: UnixNanos,
    /// Last update timestamp
    pub updated_time: UnixNanos,
    /// Commission paid on fills
    pub commission: f64,
    /// Order tags/metadata
    pub tags: HashMap<String, String>,
}

impl Order {
    /// Create a new market order
    pub fn market(
        strategy_id: StrategyId,
        instrument_id: InstrumentId,
        side: OrderSide,
        quantity: f64,
    ) -> Self {
        let now = crate::time::unix_nanos_now();
        
        Self {
            order_id: OrderId::new(),
            strategy_id,
            instrument_id,
            side,
            order_type: OrderType::Market,
            quantity,
            price: None,
            stop_price: None,
            time_in_force: TimeInForce::IOC,
            status: OrderStatus::Initialized,
            venue_order_id: None,
            filled_quantity: 0.0,
            avg_fill_price: None,
            created_time: now,
            updated_time: now,
            commission: 0.0,
            tags: HashMap::new(),
        }
    }

    /// Create a new limit order
    pub fn limit(
        strategy_id: StrategyId,
        instrument_id: InstrumentId,
        side: OrderSide,
        quantity: f64,
        price: f64,
    ) -> Self {
        let now = crate::time::unix_nanos_now();
        
        Self {
            order_id: OrderId::new(),
            strategy_id,
            instrument_id,
            side,
            order_type: OrderType::Limit,
            quantity,
            price: Some(price),
            stop_price: None,
            time_in_force: TimeInForce::GTC,
            status: OrderStatus::Initialized,
            venue_order_id: None,
            filled_quantity: 0.0,
            avg_fill_price: None,
            created_time: now,
            updated_time: now,
            commission: 0.0,
            tags: HashMap::new(),
        }
    }

    /// Check if order is active (can be filled)
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            OrderStatus::Submitted | OrderStatus::Accepted | OrderStatus::PartiallyFilled
        )
    }

    /// Check if order is complete (no longer active)
    pub fn is_complete(&self) -> bool {
        matches!(
            self.status,
            OrderStatus::Filled | OrderStatus::Cancelled | OrderStatus::Rejected | OrderStatus::Expired
        )
    }

    /// Get remaining quantity to be filled
    pub fn remaining_quantity(&self) -> f64 {
        self.quantity - self.filled_quantity
    }

    /// Check if order is fully filled
    pub fn is_filled(&self) -> bool {
        self.filled_quantity >= self.quantity
    }
}

// ============================================================================
// FILL STRUCTURE
// ============================================================================

/// Order fill/execution information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    /// Associated order ID
    pub order_id: OrderId,
    /// Exchange fill ID
    pub fill_id: String,
    /// Fill price
    pub price: f64,
    /// Fill quantity
    pub quantity: f64,
    /// Fill timestamp
    pub timestamp: UnixNanos,
    /// Commission for this fill
    pub commission: f64,
    /// Commission currency
    pub commission_currency: String,
}

// ============================================================================
// ORDER EVENTS
// ============================================================================

/// Order event types for message bus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderEvent {
    /// Order submitted to exchange
    OrderSubmitted {
        order: Order,
        timestamp: UnixNanos,
    },
    /// Order accepted by exchange
    OrderAccepted {
        order_id: OrderId,
        venue_order_id: VenueOrderId,
        timestamp: UnixNanos,
    },
    /// Order rejected by exchange
    OrderRejected {
        order_id: OrderId,
        reason: String,
        timestamp: UnixNanos,
    },
    /// Order filled (partial or complete)
    OrderFilled {
        order_id: OrderId,
        fill: Fill,
        timestamp: UnixNanos,
    },
    /// Order cancelled
    OrderCancelled {
        order_id: OrderId,
        timestamp: UnixNanos,
    },
    /// Order modified
    OrderModified {
        order_id: OrderId,
        modified_order: Order,
        timestamp: UnixNanos,
    },
}

// ============================================================================
// EXECUTION ENGINE
// ============================================================================

/// High-performance live execution engine for order management
pub struct ExecutionEngine {
    /// Message bus for event communication
    message_bus: Arc<MessageBus>,
    /// High-speed order cache
    order_cache: Arc<GenericCache<Order>>,
    /// Active orders by ID
    active_orders: Arc<RwLock<HashMap<OrderId, Order>>>,
    /// Orders by strategy
    strategy_orders: Arc<RwLock<HashMap<StrategyId, Vec<OrderId>>>>,
    /// Exchange adapters
    exchange_adapters: Arc<RwLock<HashMap<String, Box<dyn ExchangeAdapter>>>>,
    /// Order routing configuration
    routing_config: Arc<RwLock<HashMap<InstrumentId, String>>>,
    /// Execution statistics
    stats: Arc<RwLock<ExecutionStats>>,
    /// Atomic time for timestamps
    clock: Arc<AtomicTime>,
}

/// Execution performance statistics
#[derive(Debug, Default)]
pub struct ExecutionStats {
    /// Total orders submitted
    pub orders_submitted: u64,
    /// Total orders filled
    pub orders_filled: u64,
    /// Total orders cancelled
    pub orders_cancelled: u64,
    /// Total orders rejected
    pub orders_rejected: u64,
    /// Total fill volume
    pub total_fill_volume: f64,
    /// Total commission paid
    pub total_commission: f64,
    /// Average execution latency (nanoseconds)
    pub avg_execution_latency_ns: u64,
}

impl ExecutionEngine {
    /// Create a new execution engine
    pub fn new(message_bus: Arc<MessageBus>) -> Self {
        let cache_config = GenericCacheConfig {
            max_size: 10000,
            ttl_seconds: Some(3600), // 1 hour TTL for orders
            enable_statistics: true,
        };

        Self {
            message_bus,
            order_cache: Arc::new(GenericCache::new(cache_config)),
            active_orders: Arc::new(RwLock::new(HashMap::new())),
            strategy_orders: Arc::new(RwLock::new(HashMap::new())),
            exchange_adapters: Arc::new(RwLock::new(HashMap::new())),
            routing_config: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ExecutionStats::default())),
            clock: Arc::new(AtomicTime::new()),
        }
    }

    /// Submit order for execution
    pub async fn submit_order(&self, mut order: Order) -> Result<OrderId, ExecutionError> {
        let submit_time = self.clock.get();
        order.status = OrderStatus::Submitted;
        order.updated_time = submit_time;

        let order_id = order.order_id;

        // Cache the order
        self.order_cache.put(order_id.to_string(), order.clone());

        // Add to active orders
        {
            let mut active_orders = self.active_orders.write().unwrap();
            active_orders.insert(order_id, order.clone());
        }

        // Track by strategy
        {
            let mut strategy_orders = self.strategy_orders.write().unwrap();
            strategy_orders
                .entry(order.strategy_id)
                .or_insert_with(Vec::new)
                .push(order_id);
        }

        // Route to appropriate exchange
        let exchange_name = self.get_exchange_for_instrument(&order.instrument_id)?;
        
        {
            let adapters = self.exchange_adapters.read().unwrap();
            if let Some(adapter) = adapters.get(&exchange_name) {
                // Submit to exchange adapter (async)
                tokio::spawn({
                    let adapter = adapter.clone_box();
                    let order = order.clone();
                    async move {
                        if let Err(e) = adapter.submit_order(order).await {
                            eprintln!("Failed to submit order to exchange: {}", e);
                        }
                    }
                });
            } else {
                return Err(ExecutionError::ExchangeNotFound(exchange_name));
            }
        }

        // Update statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.orders_submitted += 1;
        }

        // Publish order submitted event
        let event = OrderEvent::OrderSubmitted {
            order: order.clone(),
            timestamp: submit_time,
        };
        
        self.message_bus.publish("orders.submitted", &event);

        Ok(order_id)
    }

    /// Cancel an active order
    pub async fn cancel_order(&self, order_id: OrderId) -> Result<(), ExecutionError> {
        let cancel_time = self.clock.get();

        // Get order from active orders
        let order = {
            let active_orders = self.active_orders.read().unwrap();
            active_orders.get(&order_id).cloned()
        };

        let mut order = order.ok_or(ExecutionError::OrderNotFound(order_id))?;

        if !order.is_active() {
            return Err(ExecutionError::OrderNotActive(order_id));
        }

        // Route to appropriate exchange for cancellation
        let exchange_name = self.get_exchange_for_instrument(&order.instrument_id)?;
        
        {
            let adapters = self.exchange_adapters.read().unwrap();
            if let Some(adapter) = adapters.get(&exchange_name) {
                if let Err(e) = adapter.cancel_order(order_id).await {
                    return Err(ExecutionError::ExchangeError(e.to_string()));
                }
            } else {
                return Err(ExecutionError::ExchangeNotFound(exchange_name));
            }
        }

        // Update order status
        order.status = OrderStatus::Cancelled;
        order.updated_time = cancel_time;

        // Update cache
        self.order_cache.put(order_id.to_string(), order.clone());

        // Remove from active orders
        {
            let mut active_orders = self.active_orders.write().unwrap();
            active_orders.remove(&order_id);
        }

        // Update statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.orders_cancelled += 1;
        }

        // Publish cancellation event
        let event = OrderEvent::OrderCancelled {
            order_id,
            timestamp: cancel_time,
        };
        
        self.message_bus.publish("orders.cancelled", &event);

        Ok(())
    }

    /// Handle order fill from exchange
    pub fn handle_fill(&self, fill: Fill) -> Result<(), ExecutionError> {
        let fill_time = self.clock.get();

        // Get order from active orders
        let order = {
            let active_orders = self.active_orders.read().unwrap();
            active_orders.get(&fill.order_id).cloned()
        };

        let mut order = order.ok_or(ExecutionError::OrderNotFound(fill.order_id))?;

        // Update order with fill information
        let prev_filled = order.filled_quantity;
        order.filled_quantity += fill.quantity;
        order.commission += fill.commission;
        order.updated_time = fill_time;

        // Update average fill price
        if let Some(avg_price) = order.avg_fill_price {
            let total_value = avg_price * prev_filled + fill.price * fill.quantity;
            order.avg_fill_price = Some(total_value / order.filled_quantity);
        } else {
            order.avg_fill_price = Some(fill.price);
        }

        // Update order status
        if order.is_filled() {
            order.status = OrderStatus::Filled;
        } else {
            order.status = OrderStatus::PartiallyFilled;
        }

        // Update cache
        self.order_cache.put(fill.order_id.to_string(), order.clone());

        // Update active orders or remove if filled
        if order.is_complete() {
            let mut active_orders = self.active_orders.write().unwrap();
            active_orders.remove(&fill.order_id);
        } else {
            let mut active_orders = self.active_orders.write().unwrap();
            active_orders.insert(fill.order_id, order.clone());
        }

        // Update statistics
        {
            let mut stats = self.stats.write().unwrap();
            if order.status == OrderStatus::Filled {
                stats.orders_filled += 1;
            }
            stats.total_fill_volume += fill.quantity;
            stats.total_commission += fill.commission;
        }

        // Publish fill event
        let event = OrderEvent::OrderFilled {
            order_id: fill.order_id,
            fill: fill.clone(),
            timestamp: fill_time,
        };
        
        self.message_bus.publish("orders.filled", &event);

        Ok(())
    }

    /// Get execution statistics
    pub fn get_statistics(&self) -> ExecutionStats {
        let stats = self.stats.read().unwrap();
        ExecutionStats {
            orders_submitted: stats.orders_submitted,
            orders_filled: stats.orders_filled,
            orders_cancelled: stats.orders_cancelled,
            orders_rejected: stats.orders_rejected,
            total_fill_volume: stats.total_fill_volume,
            total_commission: stats.total_commission,
            avg_execution_latency_ns: stats.avg_execution_latency_ns,
        }
    }

    /// Get orders for a strategy
    pub fn get_strategy_orders(&self, strategy_id: StrategyId) -> Vec<Order> {
        let strategy_orders = self.strategy_orders.read().unwrap();
        if let Some(order_ids) = strategy_orders.get(&strategy_id) {
            order_ids
                .iter()
                .filter_map(|id| self.order_cache.get(&id.to_string()))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get active orders count
    pub fn get_active_orders_count(&self) -> usize {
        let active_orders = self.active_orders.read().unwrap();
        active_orders.len()
    }

    /// Register exchange adapter
    pub fn register_exchange_adapter(
        &self,
        name: String,
        adapter: Box<dyn ExchangeAdapter>,
    ) {
        let mut adapters = self.exchange_adapters.write().unwrap();
        adapters.insert(name, adapter);
    }

    /// Configure instrument routing
    pub fn configure_routing(&self, instrument_id: InstrumentId, exchange_name: String) {
        let mut routing = self.routing_config.write().unwrap();
        routing.insert(instrument_id, exchange_name);
    }

    /// Get exchange for instrument
    fn get_exchange_for_instrument(&self, instrument_id: &InstrumentId) -> Result<String, ExecutionError> {
        let routing = self.routing_config.read().unwrap();
        routing
            .get(instrument_id)
            .cloned()
            .ok_or_else(|| ExecutionError::NoRoutingConfigured(*instrument_id))
    }
}

// ============================================================================
// EXCHANGE ADAPTER TRAIT
// ============================================================================

/// Trait for exchange adapters
#[async_trait::async_trait]
pub trait ExchangeAdapter: Send + Sync {
    /// Submit order to exchange
    async fn submit_order(&self, order: Order) -> Result<VenueOrderId, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Cancel order on exchange
    async fn cancel_order(&self, order_id: OrderId) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// Modify order on exchange
    async fn modify_order(&self, order_id: OrderId, new_quantity: f64, new_price: Option<f64>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// Clone the adapter (for async usage)
    fn clone_box(&self) -> Box<dyn ExchangeAdapter>;
}

// ============================================================================
// ERROR TYPES
// ============================================================================

/// Execution engine errors
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("Order not found: {0}")]
    OrderNotFound(OrderId),
    
    #[error("Order not active: {0}")]
    OrderNotActive(OrderId),
    
    #[error("Exchange not found: {0}")]
    ExchangeNotFound(String),
    
    #[error("No routing configured for instrument: {0}")]
    NoRoutingConfigured(InstrumentId),
    
    #[error("Exchange error: {0}")]
    ExchangeError(String),
    
    #[error("Invalid order parameters: {0}")]
    InvalidOrderParameters(String),
    
    #[error("Risk check failed: {0}")]
    RiskCheckFailed(String),
    
    #[error("Insufficient funds")]
    InsufficientFunds,
    
    #[error("Market closed")]
    MarketClosed,
    
    #[error("Order timeout")]
    OrderTimeout,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identifiers::{InstrumentId, StrategyId};
    use std::str::FromStr;

    #[tokio::test]
    async fn test_order_creation() {
        let strategy_id = StrategyId::new(1);
        let instrument_id = InstrumentId::from_str("BTCUSD.BINANCE").unwrap();
        
        let order = Order::market(strategy_id, instrument_id, OrderSide::Buy, 0.1);
        
        assert_eq!(order.strategy_id, strategy_id);
        assert_eq!(order.instrument_id, instrument_id);
        assert_eq!(order.side, OrderSide::Buy);
        assert_eq!(order.quantity, 0.1);
        assert_eq!(order.order_type, OrderType::Market);
        assert_eq!(order.status, OrderStatus::Initialized);
    }

    #[tokio::test]
    async fn test_execution_engine_creation() {
        let message_bus = Arc::new(MessageBus::new());
        let engine = ExecutionEngine::new(message_bus);
        
        assert_eq!(engine.get_active_orders_count(), 0);
        
        let stats = engine.get_statistics();
        assert_eq!(stats.orders_submitted, 0);
    }

    #[test]
    fn test_order_states() {
        let strategy_id = StrategyId::new(1);
        let instrument_id = InstrumentId::from_str("ETHUSD.COINBASE").unwrap();
        
        let mut order = Order::limit(strategy_id, instrument_id, OrderSide::Sell, 1.0, 3000.0);
        
        assert!(order.is_active() == false); // Initialized is not active
        assert!(order.is_complete() == false);
        
        order.status = OrderStatus::Accepted;
        assert!(order.is_active());
        assert!(order.is_complete() == false);
        
        order.status = OrderStatus::Filled;
        assert!(order.is_active() == false);
        assert!(order.is_complete());
    }

    #[test]
    fn test_order_fill_calculations() {
        let strategy_id = StrategyId::new(1);
        let instrument_id = InstrumentId::from_str("ADAUSD.KRAKEN").unwrap();
        
        let mut order = Order::limit(strategy_id, instrument_id, OrderSide::Buy, 100.0, 1.5);
        
        assert_eq!(order.remaining_quantity(), 100.0);
        assert!(!order.is_filled());
        
        // Partial fill
        order.filled_quantity = 30.0;
        assert_eq!(order.remaining_quantity(), 70.0);
        assert!(!order.is_filled());
        
        // Complete fill
        order.filled_quantity = 100.0;
        assert_eq!(order.remaining_quantity(), 0.0);
        assert!(order.is_filled());
    }
}
