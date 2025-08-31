//! High-performance order book implementation

use std::collections::{BTreeMap, VecDeque};
use serde::{Serialize, Deserialize};
use rust_decimal::Decimal;
use tracing::debug;

use alphaforge_core::time::UnixNanos;
use crate::identifiers::InstrumentId;
use crate::enums::{OrderSide, BookAction};

/// High-precision price type with fixed-point arithmetic
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Price(i64);

impl Price {
    pub const PRECISION: u8 = 9; // 9 decimal places
    const MULTIPLIER: i64 = 1_000_000_000; // 10^9
    
    /// Create a new price from raw value and precision
    pub fn new(raw: i64, precision: u8) -> Result<Self, PriceError> {
        if precision > Self::PRECISION {
            return Err(PriceError::PrecisionTooHigh(precision));
        }
        if raw <= 0 {
            return Err(PriceError::NonPositive(raw));
        }
        
        let adjusted = raw * 10_i64.pow((Self::PRECISION - precision) as u32);
        Ok(Self(adjusted))
    }
    
    /// Create price from f64 value
    pub fn from_f64(value: f64, precision: u8) -> Result<Self, PriceError> {
        if !value.is_finite() || value <= 0.0 {
            return Err(PriceError::InvalidValue(value));
        }
        
        let multiplier = 10_f64.powi(precision as i32);
        let raw = (value * multiplier).round() as i64;
        Self::new(raw, precision)
    }
    
    /// Convert to f64
    pub fn as_f64(&self) -> f64 {
        self.0 as f64 / Self::MULTIPLIER as f64
    }
    
    /// Get raw internal value
    pub fn raw(&self) -> i64 {
        self.0
    }
    
    /// Convert to Decimal for high-precision arithmetic
    pub fn as_decimal(&self) -> Decimal {
        Decimal::new(self.0, Self::PRECISION as u32)
    }
    
    /// Zero-allocation arithmetic operations
    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }
    
    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }
    
    pub fn checked_mul_f64(self, factor: f64) -> Option<Self> {
        let result = (self.0 as f64 * factor).round() as i64;
        if result > 0 && result <= i64::MAX {
            Some(Self(result))
        } else {
            None
        }
    }
}

/// Quantity type for order sizes
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Quantity(u64);

impl Quantity {
    pub const PRECISION: u8 = 8; // 8 decimal places
    const MULTIPLIER: u64 = 100_000_000; // 10^8
    
    /// Create new quantity
    pub fn new(raw: u64, precision: u8) -> Result<Self, QuantityError> {
        if precision > Self::PRECISION {
            return Err(QuantityError::PrecisionTooHigh(precision));
        }
        
        let adjusted = raw * 10_u64.pow((Self::PRECISION - precision) as u32);
        Ok(Self(adjusted))
    }
    
    /// Create from f64
    pub fn from_f64(value: f64, precision: u8) -> Result<Self, QuantityError> {
        if !value.is_finite() || value < 0.0 {
            return Err(QuantityError::InvalidValue(value));
        }
        
        let multiplier = 10_f64.powi(precision as i32);
        let raw = (value * multiplier).round() as u64;
        Self::new(raw, precision)
    }
    
    /// Convert to f64
    pub fn as_f64(&self) -> f64 {
        self.0 as f64 / Self::MULTIPLIER as f64
    }
    
    /// Get raw value
    pub fn raw(&self) -> u64 {
        self.0
    }
    
    /// Zero-allocation arithmetic
    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }
    
    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }
}

/// Book order for order book representation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub struct BookOrder {
    pub side: OrderSide,
    pub price: Price,
    pub size: Quantity,
    pub order_id: u64,
}

impl BookOrder {
    /// Create a new book order
    pub fn new(side: OrderSide, price: Price, size: Quantity, order_id: u64) -> Self {
        Self {
            side,
            price,
            size,
            order_id,
        }
    }
}

/// High-performance order book with price-time priority
#[derive(Debug, Clone)]
pub struct OrderBook {
    pub instrument_id: InstrumentId,
    // BTreeMap for O(log n) price-level operations, VecDeque for O(1) time priority
    pub bids: BTreeMap<Price, VecDeque<BookOrder>>,
    pub asks: BTreeMap<Price, VecDeque<BookOrder>>,
    pub sequence: u64,
    pub ts_last: UnixNanos,
    pub count: usize,
    // Performance optimization: cache best levels
    best_bid_price: Option<Price>,
    best_ask_price: Option<Price>,
}

impl OrderBook {
    /// Create a new order book
    pub fn new(instrument_id: InstrumentId) -> Self {
        Self {
            instrument_id,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            sequence: 0,
            ts_last: 0,
            count: 0,
            best_bid_price: None,
            best_ask_price: None,
        }
    }
    
    /// Add an order to the book - O(log n) complexity
    pub fn add(&mut self, order: BookOrder, sequence: u64, ts_event: UnixNanos) {
        let order_side = order.side;
        let order_size = order.size;
        let order_price = order.price;
        
        self.sequence = sequence;
        self.ts_last = ts_event;
        
        let price_level = match order_side {
            OrderSide::Buy => self.bids.entry(order_price).or_insert_with(VecDeque::new),
            OrderSide::Sell => self.asks.entry(order_price).or_insert_with(VecDeque::new),
        };
        
        price_level.push_back(order);
        self.count += 1;
        
        // Update cached best prices
        self.update_best_prices();
        
        debug!(
            "Added order to book: {:?} {} @ {}", 
            order_side, order_size.as_f64(), order_price.as_f64()
        );
    }
    
    /// Remove an order from the book - O(log n) complexity
    pub fn remove(&mut self, order_id: u64, side: OrderSide, price: Price) -> Option<BookOrder> {
        let price_level = match side {
            OrderSide::Buy => self.bids.get_mut(&price)?,
            OrderSide::Sell => self.asks.get_mut(&price)?,
        };
        
        // Find and remove order (O(n) within price level)
        let position = price_level.iter().position(|o| o.order_id == order_id)?;
        let removed_order = price_level.remove(position)?;
        
        // Remove empty price level
        if price_level.is_empty() {
            match side {
                OrderSide::Buy => { self.bids.remove(&price); }
                OrderSide::Sell => { self.asks.remove(&price); }
            }
        }
        
        self.count -= 1;
        self.update_best_prices();
        
        Some(removed_order)
    }
    
    /// Get best bid price - O(1) complexity (cached)
    pub fn best_bid_price(&self) -> Option<Price> {
        self.best_bid_price
    }
    
    /// Get best ask price - O(1) complexity (cached)
    pub fn best_ask_price(&self) -> Option<Price> {
        self.best_ask_price
    }
    
    /// Get spread - O(1) complexity
    pub fn spread(&self) -> Option<Decimal> {
        match (self.best_bid_price, self.best_ask_price) {
            (Some(bid), Some(ask)) => Some(ask.as_decimal() - bid.as_decimal()),
            _ => None,
        }
    }
    
    /// Get market depth for a side
    pub fn depth(&self, side: OrderSide, depth_levels: usize) -> Vec<(Price, Quantity)> {
        let mut result = Vec::with_capacity(depth_levels);
        
        let iter: Box<dyn Iterator<Item = (&Price, &VecDeque<BookOrder>)>> = match side {
            OrderSide::Buy => Box::new(self.bids.iter().rev()), // Highest bid first
            OrderSide::Sell => Box::new(self.asks.iter()),       // Lowest ask first
        };
        
        for (price, orders) in iter.take(depth_levels) {
            let total_size: u64 = orders.iter().map(|o| o.size.raw()).sum();
            if let Ok(qty) = Quantity::new(total_size, Quantity::PRECISION) {
                result.push((*price, qty));
            }
        }
        
        result
    }
    
    /// Check if order crosses the spread (would execute immediately)
    pub fn would_cross_spread(&self, side: OrderSide, price: Price) -> bool {
        match side {
            OrderSide::Buy => {
                if let Some(best_ask) = self.best_ask_price {
                    price >= best_ask
                } else {
                    false
                }
            }
            OrderSide::Sell => {
                if let Some(best_bid) = self.best_bid_price {
                    price <= best_bid
                } else {
                    false
                }
            }
        }
    }
    
    /// Clear all orders from the book
    pub fn clear(&mut self) {
        self.bids.clear();
        self.asks.clear();
        self.count = 0;
        self.best_bid_price = None;
        self.best_ask_price = None;
        self.sequence += 1;
        self.ts_last = alphaforge_core::time::unix_nanos_now();
    }
    
    /// Update cached best prices
    fn update_best_prices(&mut self) {
        self.best_bid_price = self.bids.keys().next_back().copied();
        self.best_ask_price = self.asks.keys().next().copied();
    }
    
    /// Validate book integrity (for testing)
    pub fn validate_integrity(&self) -> bool {
        // Check that bids are in descending order
        let mut prev_bid_price = None;
        for &price in self.bids.keys() {
            if let Some(prev) = prev_bid_price {
                if price >= prev {
                    return false; // Bids should be descending
                }
            }
            prev_bid_price = Some(price);
        }
        
        // Check that asks are in ascending order
        let mut prev_ask_price = None;
        for &price in self.asks.keys() {
            if let Some(prev) = prev_ask_price {
                if price <= prev {
                    return false; // Asks should be ascending
                }
            }
            prev_ask_price = Some(price);
        }
        
        true
    }
}

/// Order book delta for efficient updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookDelta {
    pub instrument_id: InstrumentId,
    pub action: BookAction,
    pub order: BookOrder,
    pub sequence: u64,
    pub ts_event: UnixNanos,
}

impl OrderBookDelta {
    /// Create a new order book delta
    pub fn new(
        instrument_id: InstrumentId,
        action: BookAction,
        order: BookOrder,
        sequence: u64,
        ts_event: UnixNanos,
    ) -> Self {
        Self {
            instrument_id,
            action,
            order,
            sequence,
            ts_event,
        }
    }
}

/// Price error types
#[derive(Debug, thiserror::Error)]
pub enum PriceError {
    #[error("Precision too high: {0}")]
    PrecisionTooHigh(u8),
    #[error("Non-positive value: {0}")]
    NonPositive(i64),
    #[error("Invalid value: {0}")]
    InvalidValue(f64),
}

/// Quantity error types
#[derive(Debug, thiserror::Error)]
pub enum QuantityError {
    #[error("Precision too high: {0}")]
    PrecisionTooHigh(u8),
    #[error("Invalid value: {0}")]
    InvalidValue(f64),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identifiers::InstrumentId;
    use crate::enums::OrderSide;
    
    #[test]
    fn test_price_creation() {
        let price = Price::from_f64(123.456, 3).unwrap();
        assert_eq!(price.as_f64(), 123.456);
        
        let price2 = Price::new(123456, 3).unwrap();
        assert_eq!(price2.as_f64(), 123.456);
    }
    
    #[test]
    fn test_price_arithmetic() {
        let price1 = Price::from_f64(100.0, 2).unwrap();
        let price2 = Price::from_f64(50.0, 2).unwrap();
        
        let sum = price1.checked_add(price2).unwrap();
        assert_eq!(sum.as_f64(), 150.0);
        
        let diff = price1.checked_sub(price2).unwrap();
        assert_eq!(diff.as_f64(), 50.0);
    }
    
    #[test]
    fn test_quantity_creation() {
        let qty = Quantity::from_f64(1000.5, 1).unwrap();
        assert_eq!(qty.as_f64(), 1000.5);
    }
    
    #[test]
    fn test_order_book_basic_operations() {
        let instrument_id = InstrumentId::new("BTCUSD.BINANCE").unwrap();
        let mut book = OrderBook::new(instrument_id);
        
        // Add some orders
        let bid_order = BookOrder::new(
            OrderSide::Buy,
            Price::from_f64(50000.0, 2).unwrap(),
            Quantity::from_f64(1.0, 2).unwrap(),
            1,
        );
        
        let ask_order = BookOrder::new(
            OrderSide::Sell,
            Price::from_f64(50100.0, 2).unwrap(),
            Quantity::from_f64(1.0, 2).unwrap(),
            2,
        );
        
        book.add(bid_order, 1, alphaforge_core::time::unix_nanos_now());
        book.add(ask_order, 2, alphaforge_core::time::unix_nanos_now());
        
        // Test best prices
        assert_eq!(book.best_bid_price(), Some(Price::from_f64(50000.0, 2).unwrap()));
        assert_eq!(book.best_ask_price(), Some(Price::from_f64(50100.0, 2).unwrap()));
        
        // Test spread
        let spread = book.spread().unwrap();
        assert_eq!(spread, Decimal::new(100_00, 2)); // $100.00 spread
        
        // Test integrity
        assert!(book.validate_integrity());
    }
    
    #[test]
    fn test_order_book_price_time_priority() {
        let instrument_id = InstrumentId::new("ETHUSD.BINANCE").unwrap();
        let mut book = OrderBook::new(instrument_id);
        
        let price = Price::from_f64(3000.0, 2).unwrap();
        
        // Add multiple orders at same price (should maintain time priority)
        for i in 1..=3 {
            let order = BookOrder::new(
                OrderSide::Buy,
                price,
                Quantity::from_f64(1.0, 1).unwrap(),
                i,
            );
            book.add(order, i, alphaforge_core::time::unix_nanos_now());
        }
        
        // Check time priority maintained
        let bid_level = book.bids.get(&price).unwrap();
        assert_eq!(bid_level.len(), 3);
        assert_eq!(bid_level[0].order_id, 1); // First order first
        assert_eq!(bid_level[1].order_id, 2);
        assert_eq!(bid_level[2].order_id, 3); // Last order last
    }
    
    #[test]
    fn test_order_book_cross_spread_detection() {
        let instrument_id = InstrumentId::new("ADAUSD.BINANCE").unwrap();
        let mut book = OrderBook::new(instrument_id);
        
        // Set up spread: bid $1.00, ask $1.02
        let bid = BookOrder::new(
            OrderSide::Buy,
            Price::from_f64(1.00, 2).unwrap(),
            Quantity::from_f64(1000.0, 0).unwrap(),
            1,
        );
        let ask = BookOrder::new(
            OrderSide::Sell,
            Price::from_f64(1.02, 2).unwrap(),
            Quantity::from_f64(1000.0, 0).unwrap(),
            2,
        );
        
        book.add(bid, 1, alphaforge_core::time::unix_nanos_now());
        book.add(ask, 2, alphaforge_core::time::unix_nanos_now());
        
        // Test crossing orders
        assert!(!book.would_cross_spread(OrderSide::Buy, Price::from_f64(1.01, 2).unwrap()));
        assert!(book.would_cross_spread(OrderSide::Buy, Price::from_f64(1.02, 2).unwrap()));
        assert!(book.would_cross_spread(OrderSide::Buy, Price::from_f64(1.03, 2).unwrap()));
        
        assert!(!book.would_cross_spread(OrderSide::Sell, Price::from_f64(1.01, 2).unwrap()));
        assert!(book.would_cross_spread(OrderSide::Sell, Price::from_f64(1.00, 2).unwrap()));
        assert!(book.would_cross_spread(OrderSide::Sell, Price::from_f64(0.99, 2).unwrap()));
    }
}
