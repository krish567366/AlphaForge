//! Core enumerations for AlphaForge trading system

use serde::{Serialize, Deserialize};

/// Order side enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum OrderSide {
    /// Buy order (bid)
    Buy = 1,
    /// Sell order (ask)  
    Sell = 2,
}

impl OrderSide {
    /// Get the opposite side
    pub fn opposite(self) -> Self {
        match self {
            Self::Buy => Self::Sell,
            Self::Sell => Self::Buy,
        }
    }
}

impl std::fmt::Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Buy => write!(f, "BUY"),
            Self::Sell => write!(f, "SELL"),
        }
    }
}

/// Order type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum OrderType {
    /// Market order - executes at current market price
    Market = 1,
    /// Limit order - executes at specified price or better
    Limit = 2,
    /// Stop order - becomes market order when stop price hit
    Stop = 3,
    /// Stop-limit order - becomes limit order when stop price hit
    StopLimit = 4,
    /// Trailing stop order
    TrailingStop = 5,
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Market => write!(f, "MARKET"),
            Self::Limit => write!(f, "LIMIT"),
            Self::Stop => write!(f, "STOP"),
            Self::StopLimit => write!(f, "STOP_LIMIT"),
            Self::TrailingStop => write!(f, "TRAILING_STOP"),
        }
    }
}

/// Order status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum OrderStatus {
    /// Order is submitted but not yet accepted
    Submitted = 1,
    /// Order is accepted by the exchange
    Accepted = 2,
    /// Order is cancelled
    Cancelled = 3,
    /// Order is expired
    Expired = 4,
    /// Order is triggered (for stop orders)
    Triggered = 5,
    /// Order is partially filled
    PartiallyFilled = 6,
    /// Order is completely filled
    Filled = 7,
    /// Order is rejected
    Rejected = 8,
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Submitted => write!(f, "SUBMITTED"),
            Self::Accepted => write!(f, "ACCEPTED"),
            Self::Cancelled => write!(f, "CANCELLED"),
            Self::Expired => write!(f, "EXPIRED"),
            Self::Triggered => write!(f, "TRIGGERED"),
            Self::PartiallyFilled => write!(f, "PARTIALLY_FILLED"),
            Self::Filled => write!(f, "FILLED"),
            Self::Rejected => write!(f, "REJECTED"),
        }
    }
}

/// Time in force enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum TimeInForce {
    /// Good Till Cancelled
    GTC = 1,
    /// Immediate Or Cancel
    IOC = 2,
    /// Fill Or Kill
    FOK = 3,
    /// Good Till Date
    GTD = 4,
    /// Day order
    DAY = 5,
}

impl std::fmt::Display for TimeInForce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GTC => write!(f, "GTC"),
            Self::IOC => write!(f, "IOC"),
            Self::FOK => write!(f, "FOK"),
            Self::GTD => write!(f, "GTD"),
            Self::DAY => write!(f, "DAY"),
        }
    }
}

/// Book action for order book updates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum BookAction {
    /// Add order to book
    Add = 1,
    /// Update existing order
    Update = 2,
    /// Delete order from book
    Delete = 3,
    /// Clear all orders at price level
    Clear = 4,
}

impl std::fmt::Display for BookAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "ADD"),
            Self::Update => write!(f, "UPDATE"),
            Self::Delete => write!(f, "DELETE"),
            Self::Clear => write!(f, "CLEAR"),
        }
    }
}

/// Aggressor side for trade ticks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum AggressorSide {
    /// No aggressor (auction/crossing)
    NoAggressor = 0,
    /// Buyer was aggressor
    Buyer = 1,
    /// Seller was aggressor
    Seller = 2,
}

impl std::fmt::Display for AggressorSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoAggressor => write!(f, "NO_AGGRESSOR"),
            Self::Buyer => write!(f, "BUYER"),
            Self::Seller => write!(f, "SELLER"),
        }
    }
}

/// Instrument class enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum InstrumentClass {
    /// Spot instrument
    Spot = 1,
    /// Forward contract
    Forward = 2,
    /// Future contract
    Future = 3,
    /// Option contract
    Option = 4,
    /// Perpetual swap
    Perpetual = 5,
    /// Contract for difference
    CFD = 6,
    /// Bond
    Bond = 7,
    /// Cryptocurrency
    Crypto = 8,
}

impl std::fmt::Display for InstrumentClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spot => write!(f, "SPOT"),
            Self::Forward => write!(f, "FORWARD"),
            Self::Future => write!(f, "FUTURE"),
            Self::Option => write!(f, "OPTION"),
            Self::Perpetual => write!(f, "PERPETUAL"),
            Self::CFD => write!(f, "CFD"),
            Self::Bond => write!(f, "BOND"),
            Self::Crypto => write!(f, "CRYPTO"),
        }
    }
}

/// Component state enumeration for lifecycle management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ComponentState {
    /// Component is initializing
    Initializing = 1,
    /// Component is initialized but not started
    Initialized = 2,
    /// Component is starting
    Starting = 3,
    /// Component is running
    Running = 4,
    /// Component is stopping
    Stopping = 5,
    /// Component is stopped
    Stopped = 6,
    /// Component is resuming from stopped state
    Resuming = 7,
    /// Component is in error state
    Error = 8,
    /// Component is disposed
    Disposed = 9,
}

impl std::fmt::Display for ComponentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initializing => write!(f, "INITIALIZING"),
            Self::Initialized => write!(f, "INITIALIZED"),
            Self::Starting => write!(f, "STARTING"),
            Self::Running => write!(f, "RUNNING"),
            Self::Stopping => write!(f, "STOPPING"),
            Self::Stopped => write!(f, "STOPPED"),
            Self::Resuming => write!(f, "RESUMING"),
            Self::Error => write!(f, "ERROR"),
            Self::Disposed => write!(f, "DISPOSED"),
        }
    }
}
