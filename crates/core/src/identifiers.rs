//! AlphaForge Identifiers
//! 
//! Type-safe identifiers for trading system components.

use serde::{Serialize, Deserialize};
use std::fmt::{self, Display};
use std::str::FromStr;

/// Instrument identifier
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstrumentId {
    // Use a simpler representation for Copy trait
    pub id: u64,  // Use numeric ID for performance
}

impl InstrumentId {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
    
    pub fn from_symbol_venue(symbol: &str, venue: &str) -> Self {
        // Simple hash combination for demo purposes
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        symbol.hash(&mut hasher);
        venue.hash(&mut hasher);
        
        Self { id: hasher.finish() }
    }
}

impl Default for InstrumentId {
    fn default() -> Self {
        Self { id: 0 }
    }
}

impl Display for InstrumentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl FromStr for InstrumentId {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try to parse as numeric ID
        if let Ok(id) = s.parse::<u64>() {
            return Ok(InstrumentId { id });
        }
        
        // Otherwise, parse as symbol.venue format
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid instrument ID format: {}", s));
        }
        
        Ok(InstrumentId::from_symbol_venue(parts[0], parts[1]))
    }
}

/// Account identifier
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountId {
    pub value: String,
}

impl AccountId {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

impl Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Client order identifier
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientOrderId {
    pub value: String,
}

impl ClientOrderId {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

impl Display for ClientOrderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Position identifier
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct PositionId {
    pub value: String,
}

impl PositionId {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

impl Display for PositionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Strategy identifier
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub struct StrategyId {
    pub id: u64,
}

impl StrategyId {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

impl Display for StrategyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// Trader identifier
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraderId {
    pub value: String,
}

impl TraderId {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

impl Display for TraderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Venue identifier
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct VenueId {
    pub value: String,
}

impl VenueId {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

impl Display for VenueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Order identifier
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderId {
    pub id: u64,
}

impl OrderId {
    /// Create a new order ID
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self {
            id: COUNTER.fetch_add(1, Ordering::SeqCst),
        }
    }

    /// Create order ID with specific value
    pub fn from_u64(id: u64) -> Self {
        Self { id }
    }
}

impl Default for OrderId {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for OrderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// Venue-assigned order identifier
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct VenueOrderId {
    pub value: String,
}

impl VenueOrderId {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

impl Display for VenueOrderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instrument_id_creation() {
        let id = InstrumentId::from_symbol_venue("EURUSD", "IDEALPRO");
        assert_eq!(id.to_string(), id.id.to_string());
    }

    #[test]
    fn test_instrument_id_from_string() {
        let id: InstrumentId = "EURUSD.IDEALPRO".parse().unwrap();
        assert!(id.id != 0); // Should have some hash value
    }

    #[test]
    fn test_invalid_instrument_id_string() {
        let result: Result<InstrumentId, _> = "INVALID".parse();
        assert!(result.is_err());
    }
}
