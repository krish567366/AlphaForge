//! UUID utilities for AlphaForge

use std::fmt;
use serde::{Serialize, Deserialize};

/// UUID v4 implementation optimized for performance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UUID4 {
    bytes: [u8; 16],
}

impl UUID4 {
    /// Generate a new UUID v4
    pub fn new() -> Self {
        let mut bytes = [0u8; 16];
        
        // Use system randomness
        #[cfg(target_os = "linux")]
        {
            use std::fs::File;
            use std::io::Read;
            if let Ok(mut f) = File::open("/dev/urandom") {
                let _ = f.read_exact(&mut bytes);
            }
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            // Fallback to std random
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            use std::time::{SystemTime, UNIX_EPOCH};
            
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            let mut hasher = DefaultHasher::new();
            now.hash(&mut hasher);
            std::thread::current().id().hash(&mut hasher);
            
            let hash = hasher.finish();
            bytes[0..8].copy_from_slice(&hash.to_le_bytes());
            
            // Second hash for remaining bytes
            let mut hasher2 = DefaultHasher::new();
            hash.hash(&mut hasher2);
            std::ptr::addr_of!(bytes).hash(&mut hasher2);
            let hash2 = hasher2.finish();
            bytes[8..16].copy_from_slice(&hash2.to_le_bytes());
        }
        
        // Set version (4) and variant bits
        bytes[6] = (bytes[6] & 0x0f) | 0x40; // Version 4
        bytes[8] = (bytes[8] & 0x3f) | 0x80; // Variant bits
        
        Self { bytes }
    }
    
    /// Create from byte array
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self { bytes }
    }
    
    /// Get raw bytes
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.bytes
    }
    
    /// Convert to hyphenated string
    pub fn to_string(&self) -> String {
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3],
            self.bytes[4], self.bytes[5],
            self.bytes[6], self.bytes[7],
            self.bytes[8], self.bytes[9],
            self.bytes[10], self.bytes[11], self.bytes[12], self.bytes[13], self.bytes[14], self.bytes[15]
        )
    }
    
    /// Parse from hyphenated string
    pub fn parse(s: &str) -> Result<Self, uuid::Error> {
        if s.len() != 36 {
            return Err(uuid::Error::InvalidLength);
        }
        
        let mut bytes = [0u8; 16];
        let mut byte_idx = 0;
        
        for (i, chunk) in s.split('-').enumerate() {
            match i {
                0 => { // 8 chars
                    if chunk.len() != 8 { return Err(uuid::Error::InvalidFormat); }
                    for j in (0..8).step_by(2) {
                        bytes[byte_idx] = u8::from_str_radix(&chunk[j..j+2], 16)
                            .map_err(|_| uuid::Error::InvalidCharacter)?;
                        byte_idx += 1;
                    }
                }
                1 | 2 => { // 4 chars each
                    if chunk.len() != 4 { return Err(uuid::Error::InvalidFormat); }
                    for j in (0..4).step_by(2) {
                        bytes[byte_idx] = u8::from_str_radix(&chunk[j..j+2], 16)
                            .map_err(|_| uuid::Error::InvalidCharacter)?;
                        byte_idx += 1;
                    }
                }
                3 => { // 4 chars
                    if chunk.len() != 4 { return Err(uuid::Error::InvalidFormat); }
                    for j in (0..4).step_by(2) {
                        bytes[byte_idx] = u8::from_str_radix(&chunk[j..j+2], 16)
                            .map_err(|_| uuid::Error::InvalidCharacter)?;
                        byte_idx += 1;
                    }
                }
                4 => { // 12 chars
                    if chunk.len() != 12 { return Err(uuid::Error::InvalidFormat); }
                    for j in (0..12).step_by(2) {
                        bytes[byte_idx] = u8::from_str_radix(&chunk[j..j+2], 16)
                            .map_err(|_| uuid::Error::InvalidCharacter)?;
                        byte_idx += 1;
                    }
                }
                _ => return Err(uuid::Error::InvalidFormat),
            }
        }
        
        Ok(Self { bytes })
    }
}

impl Default for UUID4 {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for UUID4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// UUID error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid UUID length")]
    InvalidLength,
    #[error("Invalid UUID format")]
    InvalidFormat,
    #[error("Invalid character in UUID")]
    InvalidCharacter,
}

// Create a module alias for compatibility
pub mod uuid {
    pub use super::Error;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_uuid_generation() {
        let uuid1 = UUID4::new();
        let uuid2 = UUID4::new();
        
        assert_ne!(uuid1, uuid2);
        assert_eq!(uuid1.bytes.len(), 16);
        
        // Check version bits
        assert_eq!(uuid1.bytes[6] & 0xf0, 0x40); // Version 4
        assert_eq!(uuid1.bytes[8] & 0xc0, 0x80); // Variant bits
    }
    
    #[test]
    fn test_uuid_string_conversion() {
        let uuid = UUID4::new();
        let uuid_str = uuid.to_string();
        
        assert_eq!(uuid_str.len(), 36);
        assert_eq!(uuid_str.chars().filter(|&c| c == '-').count(), 4);
        
        let parsed = UUID4::parse(&uuid_str).unwrap();
        assert_eq!(uuid, parsed);
    }
    
    #[test]
    fn test_uuid_performance() {
        let start = std::time::Instant::now();
        let count = 100_000;
        
        for _ in 0..count {
            let _uuid = UUID4::new();
        }
        
        let elapsed = start.elapsed();
        let ops_per_sec = count as f64 / elapsed.as_secs_f64();
        
        println!("UUID generation: {:.0} ops/sec", ops_per_sec);
        assert!(ops_per_sec > 100_000.0); // Should be >100k ops/sec
    }
}
