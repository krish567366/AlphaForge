//! AlphaForge Data Engine
//! 
//! Central orchestrator for market data processing with high-performance
//! tick aggregation, bar construction, and order book management.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::data::*;
use crate::identifiers::*;
use crate::time::UnixNanos;
use crate::generic_cache::GenericCache;

/// Configuration for the Data Engine
#[derive(Debug, Clone)]
pub struct DataEngineConfig {
    /// Maximum number of bars to cache per instrument
    pub max_bars_per_instrument: usize,
    /// Maximum number of ticks to buffer before processing
    pub max_tick_buffer_size: usize,
    /// Enable real-time bar aggregation
    pub enable_bar_aggregation: bool,
    /// Enable order book delta buffering
    pub enable_order_book_deltas: bool,
    /// Enable statistics collection
    pub enable_statistics: bool,
}

impl Default for DataEngineConfig {
    fn default() -> Self {
        Self {
            max_bars_per_instrument: 10_000,
            max_tick_buffer_size: 1_000,
            enable_bar_aggregation: true,
            enable_order_book_deltas: true,
            enable_statistics: true,
        }
    }
}

/// Statistics for the Data Engine performance
#[derive(Debug, Default, Clone)]
pub struct DataEngineStatistics {
    /// Total ticks processed
    pub ticks_processed: u64,
    /// Total bars generated
    pub bars_generated: u64,
    /// Total order book updates
    pub order_book_updates: u64,
    /// Processing rate (ticks per second)
    pub processing_rate: f64,
    /// Current memory usage (bytes)
    pub memory_usage: usize,
    /// Cache hit rate percentage
    pub cache_hit_rate: f64,
}

/// Bar aggregator for creating OHLCV bars from ticks
#[derive(Debug)]
pub struct BarAggregator {
    bar_type: BarType,
    current_bar: Option<PartialBar>,
    completed_bars: Vec<Bar>,
    last_close: Option<f64>,
}

/// Partial bar being constructed
#[derive(Debug, Clone)]
struct PartialBar {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    ts_start: UnixNanos,
    ts_last: UnixNanos,
    tick_count: u64,
}

impl BarAggregator {
    pub fn new(bar_type: BarType) -> Self {
        Self {
            bar_type,
            current_bar: None,
            completed_bars: Vec::new(),
            last_close: None,
        }
    }

    /// Process a trade tick and update the current bar
    pub fn update_with_trade(&mut self, tick: &TradeTick) -> Option<Bar> {
        let price = tick.price;
        let volume = tick.size;
        let ts = tick.ts_event;

        let should_close = match &mut self.current_bar {
            Some(partial) => {
                // Update existing partial bar
                partial.high = partial.high.max(price);
                partial.low = partial.low.min(price);
                partial.close = price;
                partial.volume += volume;
                partial.ts_last = ts;
                partial.tick_count += 1;

                // Check if bar should be closed based on bar specification
                Self::should_close_bar(&self.bar_type, partial, ts)
            }
            None => {
                // Start new partial bar
                self.current_bar = Some(PartialBar {
                    open: price,
                    high: price,
                    low: price,
                    close: price,
                    volume,
                    ts_start: ts,
                    ts_last: ts,
                    tick_count: 1,
                });
                false
            }
        };

        if should_close {
            self.close_current_bar(ts)
        } else {
            None
        }
    }

    /// Check if the current bar should be closed
    fn should_close_bar(bar_type: &BarType, partial: &PartialBar, current_ts: UnixNanos) -> bool {
        match &bar_type.bar_spec.aggregation {
            BarAggregation::Tick(count) => partial.tick_count >= *count,
            BarAggregation::Volume(volume) => partial.volume >= *volume as f64,
            BarAggregation::Dollar(dollar_amount) => partial.volume * partial.close >= *dollar_amount as f64,
            BarAggregation::Time(duration_nanos) => {
                (current_ts - partial.ts_start) >= *duration_nanos
            }
        }
    }

    /// Close the current bar and return it
    fn close_current_bar(&mut self, ts_close: UnixNanos) -> Option<Bar> {
        if let Some(partial) = self.current_bar.take() {
            let bar = Bar {
                bar_type: self.bar_type.clone(),
                open: partial.open,
                high: partial.high,
                low: partial.low,
                close: partial.close,
                volume: partial.volume,
                ts_event: partial.ts_last,
                ts_init: ts_close,
            };

            self.last_close = Some(partial.close);
            self.completed_bars.push(bar.clone());
            
            // Limit memory usage
            if self.completed_bars.len() > 1000 {
                self.completed_bars.remove(0);
            }

            Some(bar)
        } else {
            None
        }
    }

    /// Get the most recent completed bars
    pub fn get_recent_bars(&self, count: usize) -> Vec<Bar> {
        let start_idx = self.completed_bars.len().saturating_sub(count);
        self.completed_bars[start_idx..].to_vec()
    }
}

/// Order book delta buffer for efficient updates
#[derive(Debug)]
pub struct OrderBookDeltas {
    pub instrument_id: InstrumentId,
    pub deltas: Vec<OrderBookDelta>,
    pub sequence_number: u64,
    pub ts_last_update: UnixNanos,
}

/// Individual order book delta
#[derive(Debug, Clone)]
pub struct OrderBookDelta {
    pub side: BookSide,
    pub action: DeltaAction,
    pub price: f64,
    pub size: f64,
    pub order_id: Option<String>,
    pub ts: UnixNanos,
}

/// Order book side
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookSide {
    Bid,
    Ask,
}

/// Delta action type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeltaAction {
    Add,
    Update,
    Delete,
}

/// High-performance Data Engine for market data processing
#[derive(Debug)]
pub struct DataEngine {
    config: DataEngineConfig,
    
    // Cache for high-speed data storage
    tick_cache: Arc<GenericCache<TradeTick>>,
    quote_cache: Arc<GenericCache<QuoteTick>>,
    bar_cache: Arc<GenericCache<Bar>>,
    
    // Bar aggregation
    bar_aggregators: HashMap<BarType, BarAggregator>,
    
    // Order book delta management
    order_book_deltas: HashMap<InstrumentId, OrderBookDeltas>,
    
    // Statistics and metrics
    stats: Arc<RwLock<DataEngineStatistics>>,
    
    // Processing state
    is_running: bool,
    processed_count: u64,
}

impl DataEngine {
    /// Create a new Data Engine with specified configuration
    pub fn new(config: DataEngineConfig) -> Self {
        use crate::generic_cache::GenericCacheConfig;
        
        let cache_config = GenericCacheConfig {
            max_size: config.max_bars_per_instrument * 100, // Generous cache size
            ttl_seconds: Some(3600), // 1 hour TTL for market data
            enable_statistics: config.enable_statistics,
        };
        
        Self {
            config,
            tick_cache: Arc::new(GenericCache::new(cache_config.clone())),
            quote_cache: Arc::new(GenericCache::new(cache_config.clone())),
            bar_cache: Arc::new(GenericCache::new(cache_config)),
            bar_aggregators: HashMap::new(),
            order_book_deltas: HashMap::new(),
            stats: Arc::new(RwLock::new(DataEngineStatistics::default())),
            is_running: false,
            processed_count: 0,
        }
    }

    /// Start the Data Engine
    pub fn start(&mut self) -> Result<(), String> {
        if self.is_running {
            return Err("Data Engine is already running".to_string());
        }
        
        self.is_running = true;
        self.processed_count = 0;
        
        // Initialize statistics
        if let Ok(mut stats) = self.stats.write() {
            *stats = DataEngineStatistics::default();
        }
        
        Ok(())
    }

    /// Stop the Data Engine
    pub fn stop(&mut self) {
        self.is_running = false;
    }

    /// Process a trade tick with high performance
    pub fn process_trade_tick(&mut self, tick: TradeTick) -> Result<Option<Bar>, String> {
        if !self.is_running {
            return Err("Data Engine is not running".to_string());
        }

        // Cache the tick for fast retrieval
        let cache_key = format!("trade_{}_{}", tick.instrument_id, tick.ts_event);
        self.tick_cache.put(cache_key, tick.clone());

        // Update statistics
        self.processed_count += 1;
        if let Ok(mut stats) = self.stats.write() {
            stats.ticks_processed += 1;
        }

        // Process bar aggregation if enabled
        let mut new_bar = None;
        if self.config.enable_bar_aggregation {
            // Find relevant bar aggregators for this instrument
            let mut completed_bars = Vec::new();
            
            for (bar_type, aggregator) in self.bar_aggregators.iter_mut() {
                if bar_type.instrument_id == tick.instrument_id {
                    if let Some(bar) = aggregator.update_with_trade(&tick) {
                        completed_bars.push(bar);
                    }
                }
            }
            
            // Cache completed bars
            for bar in completed_bars.iter() {
                let cache_key = format!("bar_{}_{}", bar.bar_type.instrument_id, bar.ts_event);
                self.bar_cache.put(cache_key, bar.clone());
                
                if let Ok(mut stats) = self.stats.write() {
                    stats.bars_generated += 1;
                }
            }
            
            new_bar = completed_bars.into_iter().next();
        }

        Ok(new_bar)
    }

    /// Process a quote tick
    pub fn process_quote_tick(&mut self, tick: QuoteTick) -> Result<(), String> {
        if !self.is_running {
            return Err("Data Engine is not running".to_string());
        }

        // Cache the quote
        let cache_key = format!("quote_{}_{}", tick.instrument_id, tick.ts_event);
        self.quote_cache.put(cache_key, tick);

        // Update statistics
        self.processed_count += 1;
        if let Ok(mut stats) = self.stats.write() {
            stats.ticks_processed += 1;
        }

        Ok(())
    }

    /// Add a bar aggregator for the specified bar type
    pub fn add_bar_aggregator(&mut self, bar_type: BarType) {
        let aggregator = BarAggregator::new(bar_type.clone());
        self.bar_aggregators.insert(bar_type, aggregator);
    }

    /// Remove a bar aggregator
    pub fn remove_bar_aggregator(&mut self, bar_type: &BarType) -> bool {
        self.bar_aggregators.remove(bar_type).is_some()
    }

    /// Get recent bars for an instrument
    pub fn get_recent_bars(&self, bar_type: &BarType, count: usize) -> Vec<Bar> {
        if let Some(aggregator) = self.bar_aggregators.get(bar_type) {
            aggregator.get_recent_bars(count)
        } else {
            Vec::new()
        }
    }

    /// Get cached trade tick
    pub fn get_trade_tick(&self, instrument_id: InstrumentId, ts: UnixNanos) -> Option<TradeTick> {
        let cache_key = format!("trade_{}_{}", instrument_id, ts);
        self.tick_cache.get(&cache_key)
    }

    /// Get cached quote tick
    pub fn get_quote_tick(&self, instrument_id: InstrumentId, ts: UnixNanos) -> Option<QuoteTick> {
        let cache_key = format!("quote_{}_{}", instrument_id, ts);
        self.quote_cache.get(&cache_key)
    }

    /// Get cached bar
    pub fn get_bar(&self, instrument_id: InstrumentId, ts: UnixNanos) -> Option<Bar> {
        let cache_key = format!("bar_{}_{}", instrument_id, ts);
        self.bar_cache.get(&cache_key)
    }

    /// Get current statistics
    pub fn statistics(&self) -> DataEngineStatistics {
        if let Ok(stats) = self.stats.read() {
            stats.clone()
        } else {
            DataEngineStatistics::default()
        }
    }

    /// Reset statistics
    pub fn reset_statistics(&mut self) {
        if let Ok(mut stats) = self.stats.write() {
            *stats = DataEngineStatistics::default();
        }
    }

    /// Check if the engine is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Get total processed count
    pub fn processed_count(&self) -> u64 {
        self.processed_count
    }

    /// Get cache statistics
    pub fn cache_statistics(&self) -> (Option<crate::generic_cache::GenericCacheStatistics>, 
                                      Option<crate::generic_cache::GenericCacheStatistics>,
                                      Option<crate::generic_cache::GenericCacheStatistics>) {
        (
            self.tick_cache.statistics(),
            self.quote_cache.statistics(),
            self.bar_cache.statistics()
        )
    }
}
