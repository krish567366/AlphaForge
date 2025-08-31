//! Generic key-value cache for PyO3 integration
//! 
//! High-performance generic cache that can work with any serializable data types.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

/// Configuration for generic cache
#[derive(Debug, Clone)]
pub struct GenericCacheConfig {
    pub max_size: usize,
    pub ttl_seconds: Option<u64>,
    pub enable_statistics: bool,
}

impl Default for GenericCacheConfig {
    fn default() -> Self {
        Self {
            max_size: 10_000,
            ttl_seconds: None,
            enable_statistics: true,
        }
    }
}

/// Cache entry with expiration support
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub access_count: u64,
}

impl<T> CacheEntry<T> {
    pub fn new(value: T, ttl_seconds: Option<u64>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            value,
            created_at: now,
            expires_at: ttl_seconds.map(|ttl| now + ttl),
            access_count: 0,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now > expires_at
        } else {
            false
        }
    }
    
    pub fn touch(&mut self) {
        self.access_count += 1;
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct GenericCacheStatistics {
    pub hits: u64,
    pub misses: u64,
    pub inserts: u64,
    pub evictions: u64,
    pub memory_usage: usize,
}

impl GenericCacheStatistics {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

/// High-performance generic cache
#[derive(Debug)]
pub struct GenericCache<T> {
    config: GenericCacheConfig,
    data: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
    stats: Arc<RwLock<GenericCacheStatistics>>,
}

impl<T: Clone> GenericCache<T> {
    pub fn new(config: GenericCacheConfig) -> Self {
        Self {
            config,
            data: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(GenericCacheStatistics::default())),
        }
    }
    
    pub fn get(&self, key: &str) -> Option<T> {
        let mut data = self.data.write().unwrap();
        
        if let Some(entry) = data.get_mut(key) {
            if entry.is_expired() {
                data.remove(key);
                if self.config.enable_statistics {
                    let mut stats = self.stats.write().unwrap();
                    stats.misses += 1;
                    stats.evictions += 1;
                }
                return None;
            }
            
            entry.touch();
            if self.config.enable_statistics {
                let mut stats = self.stats.write().unwrap();
                stats.hits += 1;
            }
            Some(entry.value.clone())
        } else {
            if self.config.enable_statistics {
                let mut stats = self.stats.write().unwrap();
                stats.misses += 1;
            }
            None
        }
    }
    
    pub fn put(&self, key: String, value: T) -> bool {
        let mut data = self.data.write().unwrap();
        
        // Check size limit and evict if necessary (simple random eviction for now)
        while data.len() >= self.config.max_size {
            if let Some((oldest_key, _)) = data.iter().next() {
                let oldest_key = oldest_key.clone();
                data.remove(&oldest_key);
                if self.config.enable_statistics {
                    let mut stats = self.stats.write().unwrap();
                    stats.evictions += 1;
                }
            } else {
                break;
            }
        }
        
        let was_new = !data.contains_key(&key);
        let entry = CacheEntry::new(value, self.config.ttl_seconds);
        data.insert(key, entry);
        
        if self.config.enable_statistics && was_new {
            let mut stats = self.stats.write().unwrap();
            stats.inserts += 1;
        }
        
        true
    }
    
    pub fn contains(&self, key: &str) -> bool {
        let data = self.data.read().unwrap();
        if let Some(entry) = data.get(key) {
            !entry.is_expired()
        } else {
            false
        }
    }
    
    pub fn remove(&self, key: &str) -> bool {
        let mut data = self.data.write().unwrap();
        data.remove(key).is_some()
    }
    
    pub fn clear(&self) {
        let mut data = self.data.write().unwrap();
        data.clear();
        
        if self.config.enable_statistics {
            let mut stats = self.stats.write().unwrap();
            *stats = GenericCacheStatistics::default();
        }
    }
    
    pub fn size(&self) -> usize {
        let data = self.data.read().unwrap();
        data.len()
    }
    
    pub fn keys(&self) -> Vec<String> {
        let data = self.data.read().unwrap();
        data.keys().cloned().collect()
    }
    
    pub fn statistics(&self) -> Option<GenericCacheStatistics> {
        if self.config.enable_statistics {
            let stats = self.stats.read().unwrap();
            Some(stats.clone())
        } else {
            None
        }
    }
    
    pub fn reset_statistics(&self) {
        if self.config.enable_statistics {
            let mut stats = self.stats.write().unwrap();
            *stats = GenericCacheStatistics::default();
        }
    }
}
