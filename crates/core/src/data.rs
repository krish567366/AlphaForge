//! AlphaForge Data Types
//! 
//! Core data types for market data, orders, and trading events.

use serde::{Serialize, Deserialize};
use crate::identifiers::*;
use crate::time::UnixNanos;

/// Market data quote tick
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteTick {
    pub instrument_id: InstrumentId,
    pub bid_price: f64,
    pub ask_price: f64,
    pub bid_size: f64,
    pub ask_size: f64,
    pub ts_event: UnixNanos,
    pub ts_init: UnixNanos,
}

/// Market data trade tick
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeTick {
    pub instrument_id: InstrumentId,
    pub price: f64,
    pub size: f64,
    pub aggressor_side: AggressorSide,
    pub trade_id: String,
    pub ts_event: UnixNanos,
    pub ts_init: UnixNanos,
}

/// OHLCV bar data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bar {
    pub bar_type: BarType,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub ts_event: UnixNanos,
    pub ts_init: UnixNanos,
}

/// Bar type specification
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct BarType {
    pub instrument_id: InstrumentId,
    pub bar_spec: BarSpecification,
}

/// Bar specification with aggregation method
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct BarSpecification {
    pub step: u64,
    pub aggregation: BarAggregation,
}

/// Bar aggregation methods
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum BarAggregation {
    /// Time-based bars (duration in nanoseconds)
    Time(u64),
    /// Tick-based bars (number of ticks)
    Tick(u64),
    /// Volume-based bars (volume amount)
    Volume(u64),
    /// Dollar-based bars (dollar amount)
    Dollar(u64),
}

/// Aggressor side for trades
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AggressorSide {
    Buyer,
    Seller,
    NoAggressor,
}

/// Order book level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookOrder {
    pub side: OrderSide,
    pub price: f64,
    pub size: f64,
    pub order_id: Option<String>,
}

/// Order side
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Order book data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub instrument_id: InstrumentId,
    pub sequence: u64,
    pub ts_last: UnixNanos,
    pub count: usize,
}
