//! Identifier types for AlphaForge trading system

use std::fmt;
use serde::{Serialize, Deserialize};
use alphaforge_core::uuid::UUID4;

/// Instrument identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstrumentId {
    symbol: String,
    venue: String,
    raw: String, // Cached full identifier
}

impl InstrumentId {
    /// Create a new instrument ID from symbol.venue format
    pub fn new(identifier: &str) -> Result<Self, IdentifierError> {
        let parts: Vec<&str> = identifier.split('.').collect();
        if parts.len() != 2 {
            return Err(IdentifierError::InvalidFormat(identifier.to_string()));
        }
        
        let symbol = parts[0].to_uppercase();
        let venue = parts[1].to_uppercase();
        
        if symbol.is_empty() || venue.is_empty() {
            return Err(IdentifierError::EmptyComponent(identifier.to_string()));
        }
        
        let raw = format!("{}.{}", symbol, venue);
        
        Ok(Self {
            symbol,
            venue,
            raw,
        })
    }
    
    /// Get the symbol component
    pub fn symbol(&self) -> &str {
        &self.symbol
    }
    
    /// Get the venue component
    pub fn venue(&self) -> &str {
        &self.venue
    }
    
    /// Get the full identifier string
    pub fn value(&self) -> &str {
        &self.raw
    }
    
    /// Create from separate symbol and venue
    pub fn from_parts(symbol: &str, venue: &str) -> Result<Self, IdentifierError> {
        Self::new(&format!("{}.{}", symbol, venue))
    }
}

impl fmt::Display for InstrumentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

/// Account identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountId {
    issuer: String,
    number: String,
    raw: String,
}

impl AccountId {
    /// Create a new account ID
    pub fn new(issuer: &str, number: &str) -> Result<Self, IdentifierError> {
        if issuer.is_empty() || number.is_empty() {
            return Err(IdentifierError::EmptyComponent(format!("{}-{}", issuer, number)));
        }
        
        let raw = format!("{}-{}", issuer.to_uppercase(), number);
        
        Ok(Self {
            issuer: issuer.to_uppercase(),
            number: number.to_string(),
            raw,
        })
    }
    
    /// Get the issuer component
    pub fn issuer(&self) -> &str {
        &self.issuer
    }
    
    /// Get the account number
    pub fn number(&self) -> &str {
        &self.number
    }
    
    /// Get the full identifier
    pub fn value(&self) -> &str {
        &self.raw
    }
}

impl fmt::Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

/// Client order identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClientOrderId {
    value: String,
}

impl ClientOrderId {
    /// Create a new client order ID
    pub fn new(value: &str) -> Result<Self, IdentifierError> {
        if value.is_empty() {
            return Err(IdentifierError::EmptyComponent(value.to_string()));
        }
        
        if value.len() > 64 {
            return Err(IdentifierError::TooLong(value.to_string()));
        }
        
        Ok(Self {
            value: value.to_string(),
        })
    }
    
    /// Generate a new UUID-based client order ID
    pub fn generate() -> Self {
        Self {
            value: UUID4::new().to_string(),
        }
    }
    
    /// Get the identifier value
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for ClientOrderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Venue order identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VenueOrderId {
    value: String,
}

impl VenueOrderId {
    /// Create a new venue order ID
    pub fn new(value: &str) -> Result<Self, IdentifierError> {
        if value.is_empty() {
            return Err(IdentifierError::EmptyComponent(value.to_string()));
        }
        
        Ok(Self {
            value: value.to_string(),
        })
    }
    
    /// Get the identifier value
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for VenueOrderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Trade identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TradeId {
    value: String,
}

impl TradeId {
    /// Create a new trade ID
    pub fn new(value: &str) -> Result<Self, IdentifierError> {
        if value.is_empty() {
            return Err(IdentifierError::EmptyComponent(value.to_string()));
        }
        
        Ok(Self {
            value: value.to_string(),
        })
    }
    
    /// Get the identifier value
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for TradeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Position identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PositionId {
    value: String,
}

impl PositionId {
    /// Create a new position ID
    pub fn new(value: &str) -> Result<Self, IdentifierError> {
        if value.is_empty() {
            return Err(IdentifierError::EmptyComponent(value.to_string()));
        }
        
        Ok(Self {
            value: value.to_string(),
        })
    }
    
    /// Get the identifier value
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for PositionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Strategy identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StrategyId {
    value: String,
}

impl StrategyId {
    /// Create a new strategy ID
    pub fn new(value: &str) -> Result<Self, IdentifierError> {
        if value.is_empty() {
            return Err(IdentifierError::EmptyComponent(value.to_string()));
        }
        
        if value.len() > 64 {
            return Err(IdentifierError::TooLong(value.to_string()));
        }
        
        Ok(Self {
            value: value.to_string(),
        })
    }
    
    /// Get the identifier value
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for StrategyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Trader identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraderId {
    value: String,
}

impl TraderId {
    /// Create a new trader ID
    pub fn new(value: &str) -> Result<Self, IdentifierError> {
        if value.is_empty() {
            return Err(IdentifierError::EmptyComponent(value.to_string()));
        }
        
        if value.len() > 64 {
            return Err(IdentifierError::TooLong(value.to_string()));
        }
        
        Ok(Self {
            value: value.to_string(),
        })
    }
    
    /// Get the identifier value
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for TraderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Identifier error types
#[derive(Debug, thiserror::Error)]
pub enum IdentifierError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Empty component: {0}")]
    EmptyComponent(String),
    #[error("Too long: {0}")]
    TooLong(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_instrument_id_creation() {
        let id = InstrumentId::new("BTCUSD.BINANCE").unwrap();
        assert_eq!(id.symbol(), "BTCUSD");
        assert_eq!(id.venue(), "BINANCE");
        assert_eq!(id.value(), "BTCUSD.BINANCE");
        
        // Test case normalization
        let id2 = InstrumentId::new("btcusd.binance").unwrap();
        assert_eq!(id2.value(), "BTCUSD.BINANCE");
    }
    
    #[test]
    fn test_instrument_id_validation() {
        assert!(InstrumentId::new("INVALID").is_err());
        assert!(InstrumentId::new(".BINANCE").is_err());
        assert!(InstrumentId::new("BTCUSD.").is_err());
        assert!(InstrumentId::new("").is_err());
    }
    
    #[test]
    fn test_account_id_creation() {
        let id = AccountId::new("BINANCE", "123456").unwrap();
        assert_eq!(id.issuer(), "BINANCE");
        assert_eq!(id.number(), "123456");
        assert_eq!(id.value(), "BINANCE-123456");
    }
    
    #[test]
    fn test_client_order_id_generation() {
        let id1 = ClientOrderId::generate();
        let id2 = ClientOrderId::generate();
        
        assert_ne!(id1.value(), id2.value());
        assert_eq!(id1.value().len(), 36); // UUID format
    }
}
