//! AlphaForge Core
//! 
//! High-performance core data structures and utilities for algorithmic trading.
//! 
//! This crate provides the foundational types, time handling, messaging system,
//! and performance-critical utilities that power the AlphaForge trading platform.

pub mod error;
pub mod message;
pub mod message_bus;
pub mod time;
pub mod uuid;
pub mod cache;
pub mod generic_cache;
pub mod data;
pub mod data_engine;
pub mod identifiers;
pub mod strategy_engine;
pub mod execution_engine;

// Re-export commonly used types
pub use error::{AlphaForgeError, Result};
pub use time::{UnixNanos, AtomicTime};
pub use uuid::UUID4;
pub use data_engine::{DataEngine, DataEngineConfig, DataEngineStatistics};

/// AlphaForge version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// User agent string for HTTP requests  
pub const USER_AGENT: &str = concat!("AlphaForge/", env!("CARGO_PKG_VERSION"));
