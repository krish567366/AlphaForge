//! Error types for AlphaForge core components

use thiserror::Error;

/// Result type alias for AlphaForge operations
pub type Result<T> = std::result::Result<T, AlphaForgeError>;

/// Core error types for AlphaForge system
#[derive(Debug, Error, Clone)]
pub enum AlphaForgeError {
    #[error("Invalid configuration: {msg}")]
    InvalidConfiguration { msg: String },
    
    #[error("Network error: {msg}")]
    Network { msg: String },
    
    #[error("Serialization error: {msg}")]
    Serialization { msg: String },
    
    #[error("Time error: {msg}")]
    Time { msg: String },
    
    #[error("Validation error: {msg}")]
    Validation { msg: String },
    
    #[error("Component error: {msg}")]
    Component { msg: String },
    
    #[error("Message bus error: {msg}")]
    MessageBus { msg: String },
    
    #[error("Runtime error: {msg}")]
    Runtime { msg: String },
}

impl AlphaForgeError {
    /// Create a new configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::InvalidConfiguration { msg: msg.into() }
    }
    
    /// Create a new network error
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network { msg: msg.into() }
    }
    
    /// Create a new validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation { msg: msg.into() }
    }
    
    /// Create a new runtime error
    pub fn runtime(msg: impl Into<String>) -> Self {
        Self::Runtime { msg: msg.into() }
    }
}

// Conversion from common error types
impl From<serde_json::Error> for AlphaForgeError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization { msg: err.to_string() }
    }
}

impl From<rmp_serde::encode::Error> for AlphaForgeError {
    fn from(err: rmp_serde::encode::Error) -> Self {
        Self::Serialization { msg: err.to_string() }
    }
}

impl From<rmp_serde::decode::Error> for AlphaForgeError {
    fn from(err: rmp_serde::decode::Error) -> Self {
        Self::Serialization { msg: err.to_string() }
    }
}
