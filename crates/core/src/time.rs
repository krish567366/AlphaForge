//! High-performance time handling for AlphaForge
//! 
//! Provides unified time abstractions for backtesting and live trading modes.

use std::sync::atomic::{AtomicU64, Ordering};
use chrono::{DateTime, Utc, NaiveDateTime};

/// Nanoseconds since UNIX epoch (1970-01-01 00:00:00 UTC)
pub type UnixNanos = u64;

/// Atomic timestamp for lock-free time operations
#[derive(Debug, Default)]
pub struct AtomicTime {
    nanos: AtomicU64,
}

impl AtomicTime {
    /// Create new atomic time with current timestamp
    pub fn new() -> Self {
        Self {
            nanos: AtomicU64::new(unix_nanos_now()),
        }
    }
    
    /// Get current timestamp
    pub fn get(&self) -> UnixNanos {
        self.nanos.load(Ordering::Relaxed)
    }
    
    /// Update timestamp
    pub fn set(&self, timestamp: UnixNanos) {
        self.nanos.store(timestamp, Ordering::Relaxed);
    }
    
    /// Update to current time
    pub fn update_now(&self) {
        self.set(unix_nanos_now());
    }
}

/// Get current Unix timestamp in nanoseconds
pub fn unix_nanos_now() -> UnixNanos {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos() as u64
}

/// Convert UnixNanos to DateTime<Utc>
pub fn unix_nanos_to_datetime(nanos: UnixNanos) -> Result<DateTime<Utc>, String> {
    let secs = (nanos / 1_000_000_000) as i64;
    let nsecs = (nanos % 1_000_000_000) as u32;
    
    match DateTime::from_timestamp(secs, nsecs) {
        Some(dt) => Ok(dt),
        None => Err("Invalid timestamp".to_string()),
    }
}

/// Convert DateTime<Utc> to UnixNanos
pub fn datetime_to_unix_nanos(dt: DateTime<Utc>) -> UnixNanos {
    (dt.timestamp() as u64) * 1_000_000_000 + (dt.timestamp_subsec_nanos() as u64)
}

/// Precision time parsing for various formats
pub fn parse_datetime_string(s: &str) -> Result<UnixNanos, String> {
    // Try multiple common formats
    let formats = [
        "%Y-%m-%d %H:%M:%S%.f",      // ISO-like with microseconds
        "%Y-%m-%dT%H:%M:%S%.fZ",     // ISO 8601 with Z
        "%Y-%m-%dT%H:%M:%S%.f%z",    // ISO 8601 with timezone
        "%Y-%m-%d %H:%M:%S",         // Simple datetime
        "%Y-%m-%dT%H:%M:%SZ",        // ISO 8601 simple
    ];
    
    for format in &formats {
        if let Ok(dt) = DateTime::parse_from_str(s, format) {
            return Ok(datetime_to_unix_nanos(dt.with_timezone(&Utc)));
        }
    }
    
    // Try naive datetime and assume UTC
    for format in &formats {
        if let Ok(naive) = NaiveDateTime::parse_from_str(s, format) {
            let dt = DateTime::from_naive_utc_and_offset(naive, Utc);
            return Ok(datetime_to_unix_nanos(dt));
        }
    }
    
    Err("Unable to parse datetime string".to_string())
}

/// High-resolution timer for performance measurements
#[derive(Debug, Clone)]
pub struct PrecisionTimer {
    start: std::time::Instant,
}

impl PrecisionTimer {
    /// Start a new timer
    pub fn start() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }
    
    /// Get elapsed time in nanoseconds
    pub fn elapsed_nanos(&self) -> u64 {
        self.start.elapsed().as_nanos() as u64
    }
    
    /// Get elapsed time in microseconds
    pub fn elapsed_micros(&self) -> u64 {
        self.start.elapsed().as_micros() as u64
    }
    
    /// Get elapsed time in milliseconds
    pub fn elapsed_millis(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unix_nanos_conversion() {
        let now = unix_nanos_now();
        let dt = unix_nanos_to_datetime(now).expect("Failed to convert to datetime");
        let converted_back = datetime_to_unix_nanos(dt);
        
        // Allow for small precision loss
        assert!((now as i64 - converted_back as i64).abs() < 1000);
    }
    
    #[test]
    fn test_atomic_time() {
        let atomic_time = AtomicTime::new();
        let initial = atomic_time.get();
        
        std::thread::sleep(std::time::Duration::from_millis(1));
        atomic_time.update_now();
        
        let updated = atomic_time.get();
        assert!(updated > initial);
    }
    
    #[test]
    fn test_precision_timer() {
        let timer = PrecisionTimer::start();
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        let elapsed = timer.elapsed_nanos();
        assert!(elapsed > 1_000_000); // At least 1ms
    }
}
