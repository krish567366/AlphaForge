//! AlphaForge Cache Subsystem
//! 
//! High-performance in-memory cache with optional persistence for market and execution data.
//! Implements O(1) lookups with AHashMap and LRU eviction for memory management.

use std::collections::VecDeque;
use ahash::AHashMap;
use serde::{Serialize, Deserialize};
use parking_lot::RwLock;
use tracing::{debug, info};

use crate::time::UnixNanos;
use crate::identifiers::*;
use crate::data::*;

/// High-performance cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of items to cache per data type
    pub max_items_per_type: usize,
    /// Enable persistent backing store
    pub enable_persistence: bool,
    /// Cache eviction policy
    pub eviction_policy: EvictionPolicy,
    /// Flush interval for persistence (milliseconds)
    pub flush_interval_ms: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_items_per_type: 100_000,
            enable_persistence: false,
            eviction_policy: EvictionPolicy::LRU,
            flush_interval_ms: 1000,
        }
    }
}

/// Cache eviction policies
#[derive(Debug, Clone, Copy)]
pub enum EvictionPolicy {
    /// Least Recently Used
    LRU,
    /// First In, First Out
    FIFO,
    /// Least Frequently Used
    LFU,
}

/// Cache index for complex queries
#[derive(Debug, Default)]
pub struct CacheIndex {
    /// Instrument ID to symbol mapping
    pub instruments_by_symbol: AHashMap<String, InstrumentId>,
    /// Venue to instruments mapping
    pub instruments_by_venue: AHashMap<String, Vec<InstrumentId>>,
    /// Currency pairs index
    pub currency_pairs: AHashMap<(String, String), Vec<InstrumentId>>,
}

/// Database adapter trait for persistence
pub trait CacheDatabaseAdapter: Send + Sync {
    fn write_batch(&self, data: &[CacheEntry]) -> Result<(), CacheError>;
    fn read_by_key(&self, key: &str) -> Result<Option<CacheEntry>, CacheError>;
    fn flush(&self) -> Result<(), CacheError>;
}

/// Cache entry for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub data_type: String,
    pub data: Vec<u8>,
    pub timestamp: UnixNanos,
    pub access_count: u64,
}

/// Cache errors
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Cache is full, eviction failed")]
    CacheFull,
    #[error("Key not found: {key}")]
    KeyNotFound { key: String },
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("Database error: {0}")]
    Database(String),
}

/// High-performance in-memory cache as specified in copilot instructions
pub struct Cache {
    config: CacheConfig,
    index: RwLock<CacheIndex>,
    database: Option<Box<dyn CacheDatabaseAdapter>>,
    
    // Core market data - O(1) lookups with AHashMap
    currencies: RwLock<AHashMap<String, Currency>>,
    instruments: RwLock<AHashMap<InstrumentId, InstrumentAny>>,
    books: RwLock<AHashMap<InstrumentId, OrderBook>>,
    quotes: RwLock<AHashMap<InstrumentId, VecDeque<QuoteTick>>>,
    trades: RwLock<AHashMap<InstrumentId, VecDeque<TradeTick>>>,
    bars: RwLock<AHashMap<BarType, VecDeque<Bar>>>,
    
    // Execution data
    accounts: RwLock<AHashMap<String, Account>>,
    orders: RwLock<AHashMap<String, Order>>,
    positions: RwLock<AHashMap<String, Position>>,
    
    // Performance metrics
    stats: CacheStats,
}

/// Cache performance statistics
#[derive(Debug, Default)]
pub struct CacheStats {
    pub hits: std::sync::atomic::AtomicU64,
    pub misses: std::sync::atomic::AtomicU64,
    pub evictions: std::sync::atomic::AtomicU64,
    pub writes: std::sync::atomic::AtomicU64,
}

impl CacheStats {
    pub fn hit_ratio(&self) -> f64 {
        let hits = self.hits.load(std::sync::atomic::Ordering::Relaxed);
        let misses = self.misses.load(std::sync::atomic::Ordering::Relaxed);
        let total = hits + misses;
        
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }
}

impl Cache {
    /// Create new cache instance
    pub fn new(config: CacheConfig) -> Self {
        info!("Initializing AlphaForge cache with config: {:?}", config);
        
        Self {
            config,
            index: RwLock::new(CacheIndex::default()),
            database: None,
            currencies: RwLock::new(AHashMap::with_capacity(200)), // ~200 currencies
            instruments: RwLock::new(AHashMap::with_capacity(10_000)), // 10k instruments
            books: RwLock::new(AHashMap::with_capacity(1_000)), // 1k order books
            quotes: RwLock::new(AHashMap::with_capacity(1_000)),
            trades: RwLock::new(AHashMap::with_capacity(1_000)),
            bars: RwLock::new(AHashMap::with_capacity(1_000)),
            accounts: RwLock::new(AHashMap::with_capacity(100)),
            orders: RwLock::new(AHashMap::with_capacity(100_000)),
            positions: RwLock::new(AHashMap::with_capacity(10_000)),
            stats: CacheStats::default(),
        }
    }
    
    /// Add currency to cache - O(1) operation
    pub fn add_currency(&self, currency: Currency) -> Result<(), CacheError> {
        let code = currency.code.clone(); // Clone before moving
        let mut currencies = self.currencies.write();
        currencies.insert(currency.code.clone(), currency);
        self.stats.writes.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        debug!("Cached currency: {}", code);
        Ok(())
    }
    
    /// Get currency from cache - O(1) lookup
    pub fn get_currency(&self, code: &str) -> Option<Currency> {
        let currencies = self.currencies.read();
        if let Some(currency) = currencies.get(code) {
            self.stats.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Some(currency.clone())
        } else {
            self.stats.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            None
        }
    }
    
    /// Add instrument to cache with automatic indexing
    pub fn add_instrument(&self, instrument: InstrumentAny) -> Result<(), CacheError> {
        let instrument_id = instrument.id();
        let symbol = instrument.symbol().to_string();
        let venue = instrument.venue().to_string();
        
        // Update main cache
        let mut instruments = self.instruments.write();
        instruments.insert(instrument_id, instrument);
        
        // Update index
        let mut index = self.index.write();
        index.instruments_by_symbol.insert(symbol, instrument_id);
        index.instruments_by_venue
            .entry(venue)
            .or_insert_with(Vec::new)
            .push(instrument_id);
        
        self.stats.writes.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        debug!("Cached instrument: {}", instrument_id);
        Ok(())
    }
    
    /// Get instrument from cache - O(1) lookup
    pub fn get_instrument(&self, instrument_id: &InstrumentId) -> Option<InstrumentAny> {
        let instruments = self.instruments.read();
        if let Some(instrument) = instruments.get(instrument_id) {
            self.stats.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Some(instrument.clone())
        } else {
            self.stats.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            None
        }
    }
    
    /// Add order book to cache
    pub fn add_order_book(&self, book: OrderBook) -> Result<(), CacheError> {
        let instrument_id = book.instrument_id;
        let mut books = self.books.write();
        books.insert(instrument_id, book);
        self.stats.writes.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        debug!("Cached order book: {}", instrument_id);
        Ok(())
    }
    
    /// Get order book from cache - O(1) lookup
    pub fn get_order_book(&self, instrument_id: &InstrumentId) -> Option<OrderBook> {
        let books = self.books.read();
        if let Some(book) = books.get(instrument_id) {
            self.stats.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Some(book.clone())
        } else {
            self.stats.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            None
        }
    }
    
    /// Add quote tick with automatic deque management
    pub fn add_quote_tick(&self, tick: QuoteTick) -> Result<(), CacheError> {
        let instrument_id = tick.instrument_id;
        let mut quotes = self.quotes.write();
        
        let quote_deque = quotes.entry(instrument_id).or_insert_with(VecDeque::new);
        quote_deque.push_back(tick);
        
        // Implement LRU eviction if queue is too long
        if quote_deque.len() > self.config.max_items_per_type {
            quote_deque.pop_front();
            self.stats.evictions.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        
        self.stats.writes.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
    
    /// Get recent quotes for instrument
    pub fn get_quotes(&self, instrument_id: &InstrumentId, limit: Option<usize>) -> Vec<QuoteTick> {
        let quotes = self.quotes.read();
        if let Some(quote_deque) = quotes.get(instrument_id) {
            self.stats.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            
            let limit = limit.unwrap_or(quote_deque.len());
            quote_deque.iter()
                .rev()
                .take(limit)
                .cloned()
                .collect()
        } else {
            self.stats.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Vec::new()
        }
    }
    
    /// Add trade tick with automatic deque management  
    pub fn add_trade_tick(&self, tick: TradeTick) -> Result<(), CacheError> {
        let instrument_id = tick.instrument_id;
        let mut trades = self.trades.write();
        
        let trade_deque = trades.entry(instrument_id).or_insert_with(VecDeque::new);
        trade_deque.push_back(tick);
        
        // Implement LRU eviction if queue is too long
        if trade_deque.len() > self.config.max_items_per_type {
            trade_deque.pop_front();
            self.stats.evictions.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        
        self.stats.writes.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
    
    /// Get recent trades for instrument
    pub fn get_trades(&self, instrument_id: &InstrumentId, limit: Option<usize>) -> Vec<TradeTick> {
        let trades = self.trades.read();
        if let Some(trade_deque) = trades.get(instrument_id) {
            self.stats.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            
            let limit = limit.unwrap_or(trade_deque.len());
            trade_deque.iter()
                .rev()
                .take(limit)
                .cloned()
                .collect()
        } else {
            self.stats.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Vec::new()
        }
    }
    
    /// Get cache statistics for monitoring
    pub fn get_stats(&self) -> CacheStatistics {
        CacheStatistics {
            hit_ratio: self.stats.hit_ratio(),
            total_hits: self.stats.hits.load(std::sync::atomic::Ordering::Relaxed),
            total_misses: self.stats.misses.load(std::sync::atomic::Ordering::Relaxed),
            total_writes: self.stats.writes.load(std::sync::atomic::Ordering::Relaxed),
            total_evictions: self.stats.evictions.load(std::sync::atomic::Ordering::Relaxed),
            currencies_count: self.currencies.read().len(),
            instruments_count: self.instruments.read().len(),
            books_count: self.books.read().len(),
            quotes_count: self.quotes.read().values().map(|q| q.len()).sum(),
            trades_count: self.trades.read().values().map(|t| t.len()).sum(),
        }
    }
    
    /// Clear all cached data
    pub fn clear(&self) {
        info!("Clearing cache");
        self.currencies.write().clear();
        self.instruments.write().clear(); 
        self.books.write().clear();
        self.quotes.write().clear();
        self.trades.write().clear();
        self.bars.write().clear();
        self.accounts.write().clear();
        self.orders.write().clear();
        self.positions.write().clear();
        *self.index.write() = CacheIndex::default();
    }
}

/// Cache statistics for monitoring and observability
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub hit_ratio: f64,
    pub total_hits: u64,
    pub total_misses: u64,
    pub total_writes: u64,
    pub total_evictions: u64,
    pub currencies_count: usize,
    pub instruments_count: usize,
    pub books_count: usize,
    pub quotes_count: usize,
    pub trades_count: usize,
}

// Placeholder types - these would be implemented in their respective modules
#[derive(Debug, Clone)]
pub struct Currency {
    pub code: String,
    pub precision: u8,
    pub iso4217: u16,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct InstrumentAny {
    // Placeholder - actual implementation would be an enum of different instrument types
}

impl InstrumentAny {
    pub fn id(&self) -> InstrumentId {
        // For now, return a placeholder ID
        // In a real implementation, this would extract the ID from the enum variant
        InstrumentId::new(1)
    }
    
    pub fn symbol(&self) -> &str {
        // Placeholder implementation
        "PLACEHOLDER"
    }
    
    pub fn venue(&self) -> &str {
        // Placeholder implementation  
        "PLACEHOLDER"
    }
}

#[derive(Debug, Clone)]
pub struct Account {
    pub id: String,
    pub balance: f64,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
    pub instrument_id: InstrumentId,
    pub side: String,
    pub quantity: f64,
    pub price: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub id: String,
    pub instrument_id: InstrumentId,
    pub quantity: f64,
    pub avg_price: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_creation() {
        let config = CacheConfig::default();
        let cache = Cache::new(config);
        
        let stats = cache.get_stats();
        assert_eq!(stats.hit_ratio, 0.0);
        assert_eq!(stats.total_hits, 0);
        assert_eq!(stats.total_misses, 0);
    }
    
    #[test]
    fn test_currency_caching() {
        let cache = Cache::new(CacheConfig::default());
        
        let currency = Currency {
            code: "USD".to_string(),
            precision: 2,
            iso4217: 840,
            name: "US Dollar".to_string(),
        };
        
        // Add currency
        cache.add_currency(currency.clone()).unwrap();
        
        // Retrieve currency
        let retrieved = cache.get_currency("USD").unwrap();
        assert_eq!(retrieved.code, "USD");
        
        // Check stats
        let stats = cache.get_stats();
        assert_eq!(stats.total_hits, 1);
        assert_eq!(stats.currencies_count, 1);
    }
    
    #[test]
    fn test_cache_miss() {
        let cache = Cache::new(CacheConfig::default());
        
        // Try to get non-existent currency
        let result = cache.get_currency("EUR");
        assert!(result.is_none());
        
        // Check stats
        let stats = cache.get_stats();
        assert_eq!(stats.total_misses, 1);
        assert_eq!(stats.hit_ratio, 0.0);
    }
}
