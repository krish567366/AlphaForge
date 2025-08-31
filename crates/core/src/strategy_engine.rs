use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

use crate::data::{TradeTick, QuoteTick, Bar};
use crate::identifiers::{InstrumentId, StrategyId};
use crate::data_engine::DataEngine;
use crate::generic_cache::GenericCache;

/// Strategy state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyState {
    /// Strategy is initialized but not started
    Initialized,
    /// Strategy is actively running
    Running,
    /// Strategy is paused (can be resumed)
    Paused,
    /// Strategy is stopped (cannot be resumed)
    Stopped,
    /// Strategy encountered an error
    Error,
}

/// Base configuration for all strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    /// Unique identifier for the strategy
    pub strategy_id: StrategyId,
    /// Strategy name for logging and identification
    pub name: String,
    /// Instruments this strategy will trade
    pub instruments: Vec<InstrumentId>,
    /// Maximum position size per instrument
    pub max_position_size: f64,
    /// Risk management parameters
    pub max_daily_loss: f64,
    pub max_drawdown: f64,
    /// Enable/disable features
    pub enable_logging: bool,
    pub enable_metrics: bool,
    pub enable_backtesting: bool,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            strategy_id: StrategyId::new(1),
            name: "DefaultStrategy".to_string(),
            instruments: vec![],
            max_position_size: 1000.0,
            max_daily_loss: 10000.0,
            max_drawdown: 0.05, // 5%
            enable_logging: true,
            enable_metrics: true,
            enable_backtesting: false,
        }
    }
}

/// Strategy performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StrategyMetrics {
    /// Total number of trades executed
    pub total_trades: u64,
    /// Number of profitable trades
    pub winning_trades: u64,
    /// Number of losing trades
    pub losing_trades: u64,
    /// Total profit/loss
    pub total_pnl: f64,
    /// Gross profit from winning trades
    pub gross_profit: f64,
    /// Gross loss from losing trades
    pub gross_loss: f64,
    /// Maximum consecutive wins
    pub max_consecutive_wins: u64,
    /// Maximum consecutive losses
    pub max_consecutive_losses: u64,
    /// Maximum drawdown experienced
    pub max_drawdown: f64,
    /// Sharpe ratio (if applicable)
    pub sharpe_ratio: f64,
    /// Current open positions
    pub open_positions: HashMap<InstrumentId, f64>,
    /// Strategy uptime in seconds
    pub uptime_seconds: u64,
    /// Last update timestamp
    pub last_update_ts: u64,
}

/// Strategy execution context
pub struct StrategyContext {
    /// Strategy configuration
    pub config: StrategyConfig,
    /// Current strategy state
    pub state: StrategyState,
    /// Performance metrics
    pub metrics: StrategyMetrics,
    /// Reference to data engine
    pub data_engine: Arc<Mutex<DataEngine>>,
    /// Strategy-specific cache for indicators and state
    pub cache: Arc<Mutex<GenericCache<f64>>>,
    /// Strategy start time
    pub start_time: SystemTime,
    /// Last heartbeat time
    pub last_heartbeat: SystemTime,
}

impl StrategyContext {
    /// Create a new strategy context
    pub fn new(config: StrategyConfig, data_engine: Arc<Mutex<DataEngine>>) -> Self {
        let cache_config = crate::generic_cache::GenericCacheConfig {
            max_size: 10000,
            ttl_seconds: Some(300), // 5 minutes
            enable_statistics: true,
        };
        
        Self {
            config,
            state: StrategyState::Initialized,
            metrics: StrategyMetrics::default(),
            data_engine,
            cache: Arc::new(Mutex::new(GenericCache::new(cache_config))),
            start_time: SystemTime::now(),
            last_heartbeat: SystemTime::now(),
        }
    }

    /// Get current timestamp in nanoseconds
    pub fn current_time_ns(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos() as u64
    }

    /// Update strategy state
    pub fn set_state(&mut self, state: StrategyState) {
        self.state = state;
        self.last_heartbeat = SystemTime::now();
    }

    /// Check if strategy is active
    pub fn is_active(&self) -> bool {
        matches!(self.state, StrategyState::Running)
    }

    /// Update metrics with a new trade
    pub fn record_trade(&mut self, instrument_id: InstrumentId, pnl: f64, size: f64) {
        self.metrics.total_trades += 1;
        self.metrics.total_pnl += pnl;

        if pnl > 0.0 {
            self.metrics.winning_trades += 1;
            self.metrics.gross_profit += pnl;
        } else if pnl < 0.0 {
            self.metrics.losing_trades += 1;
            self.metrics.gross_loss += pnl.abs();
        }

        // Update position
        *self.metrics.open_positions.entry(instrument_id).or_insert(0.0) += size;

        self.metrics.last_update_ts = self.current_time_ns();
    }

    /// Calculate current win rate
    pub fn win_rate(&self) -> f64 {
        if self.metrics.total_trades == 0 {
            0.0
        } else {
            self.metrics.winning_trades as f64 / self.metrics.total_trades as f64
        }
    }

    /// Calculate current profit factor
    pub fn profit_factor(&self) -> f64 {
        if self.metrics.gross_loss == 0.0 {
            f64::INFINITY
        } else {
            self.metrics.gross_profit / self.metrics.gross_loss
        }
    }
}

/// Base trait for all trading strategies
pub trait Strategy: Send + Sync {
    /// Initialize the strategy
    fn on_start(&mut self, context: &mut StrategyContext) -> Result<(), String>;

    /// Handle incoming trade tick data
    fn on_trade_tick(&mut self, context: &mut StrategyContext, tick: &TradeTick) -> Result<(), String>;

    /// Handle incoming quote tick data
    fn on_quote_tick(&mut self, context: &mut StrategyContext, tick: &QuoteTick) -> Result<(), String>;

    /// Handle incoming bar data
    fn on_bar(&mut self, context: &mut StrategyContext, bar: &Bar) -> Result<(), String>;

    /// Handle strategy timer events
    fn on_timer(&mut self, context: &mut StrategyContext) -> Result<(), String>;

    /// Stop the strategy
    fn on_stop(&mut self, context: &mut StrategyContext) -> Result<(), String>;

    /// Get strategy name
    fn name(&self) -> &str;

    /// Get strategy version
    fn version(&self) -> &str {
        "1.0.0"
    }
}

/// Strategy engine that manages multiple strategies
pub struct StrategyEngine {
    /// Registered strategies
    strategies: HashMap<StrategyId, (Box<dyn Strategy>, StrategyContext)>,
    /// Reference to data engine
    data_engine: Arc<Mutex<DataEngine>>,
    /// Engine state
    is_running: bool,
    /// Engine statistics
    total_strategies: usize,
    active_strategies: usize,
}

impl StrategyEngine {
    /// Create a new strategy engine
    pub fn new(data_engine: Arc<Mutex<DataEngine>>) -> Self {
        Self {
            strategies: HashMap::new(),
            data_engine,
            is_running: false,
            total_strategies: 0,
            active_strategies: 0,
        }
    }

    /// Register a new strategy
    pub fn add_strategy(&mut self, strategy: Box<dyn Strategy>, config: StrategyConfig) -> Result<(), String> {
        let strategy_id = config.strategy_id;
        
        if self.strategies.contains_key(&strategy_id) {
            return Err(format!("Strategy with ID {:?} already exists", strategy_id));
        }

        let context = StrategyContext::new(config, Arc::clone(&self.data_engine));
        self.strategies.insert(strategy_id, (strategy, context));
        self.total_strategies += 1;

        Ok(())
    }

    /// Start the strategy engine
    pub fn start(&mut self) -> Result<(), String> {
        if self.is_running {
            return Err("Strategy engine is already running".to_string());
        }

        // Start all strategies
        for (_, (strategy, context)) in &mut self.strategies {
            context.set_state(StrategyState::Running);
            strategy.on_start(context)?;
        }

        self.is_running = true;
        self.active_strategies = self.strategies.len();
        Ok(())
    }

    /// Stop the strategy engine
    pub fn stop(&mut self) -> Result<(), String> {
        if !self.is_running {
            return Ok(());
        }

        // Stop all strategies
        for (_, (strategy, context)) in &mut self.strategies {
            context.set_state(StrategyState::Stopped);
            strategy.on_stop(context)?;
        }

        self.is_running = false;
        self.active_strategies = 0;
        Ok(())
    }

    /// Process a trade tick for all relevant strategies
    pub fn process_trade_tick(&mut self, tick: &TradeTick) -> Result<(), String> {
        if !self.is_running {
            return Ok(());
        }

        for (_, (strategy, context)) in &mut self.strategies {
            if context.is_active() && context.config.instruments.contains(&tick.instrument_id) {
                strategy.on_trade_tick(context, tick)?;
            }
        }

        Ok(())
    }

    /// Process a quote tick for all relevant strategies
    pub fn process_quote_tick(&mut self, tick: &QuoteTick) -> Result<(), String> {
        if !self.is_running {
            return Ok(());
        }

        for (_, (strategy, context)) in &mut self.strategies {
            if context.is_active() && context.config.instruments.contains(&tick.instrument_id) {
                strategy.on_quote_tick(context, tick)?;
            }
        }

        Ok(())
    }

    /// Process a bar for all relevant strategies
    pub fn process_bar(&mut self, bar: &Bar) -> Result<(), String> {
        if !self.is_running {
            return Ok(());
        }

        for (_, (strategy, context)) in &mut self.strategies {
            if context.is_active() {
                strategy.on_bar(context, bar)?;
            }
        }

        Ok(())
    }

    /// Run timer events for all strategies
    pub fn process_timer(&mut self) -> Result<(), String> {
        if !self.is_running {
            return Ok(());
        }

        for (_, (strategy, context)) in &mut self.strategies {
            if context.is_active() {
                strategy.on_timer(context)?;
            }
        }

        Ok(())
    }

    /// Get strategy metrics
    pub fn get_strategy_metrics(&self, strategy_id: &StrategyId) -> Option<&StrategyMetrics> {
        self.strategies.get(strategy_id).map(|(_, context)| &context.metrics)
    }

    /// Get all strategy metrics
    pub fn get_all_metrics(&self) -> HashMap<StrategyId, &StrategyMetrics> {
        self.strategies
            .iter()
            .map(|(id, (_, context))| (*id, &context.metrics))
            .collect()
    }

    /// Check if engine is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Get total number of strategies
    pub fn total_strategies(&self) -> usize {
        self.total_strategies
    }

    /// Get number of active strategies
    pub fn active_strategies(&self) -> usize {
        self.active_strategies
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock strategy for testing
    struct TestStrategy {
        name: String,
        trade_count: u64,
    }

    impl TestStrategy {
        fn new(name: String) -> Self {
            Self { name, trade_count: 0 }
        }
    }

    impl Strategy for TestStrategy {
        fn on_start(&mut self, _context: &mut StrategyContext) -> Result<(), String> {
            println!("Strategy {} started", self.name);
            Ok(())
        }

        fn on_trade_tick(&mut self, context: &mut StrategyContext, tick: &TradeTick) -> Result<(), String> {
            self.trade_count += 1;
            
            // Simulate a trade with random P&L
            let pnl = if self.trade_count % 2 == 0 { 100.0 } else { -50.0 };
            context.record_trade(tick.instrument_id, pnl, tick.size);
            
            Ok(())
        }

        fn on_quote_tick(&mut self, _context: &mut StrategyContext, _tick: &QuoteTick) -> Result<(), String> {
            Ok(())
        }

        fn on_bar(&mut self, _context: &mut StrategyContext, _bar: &Bar) -> Result<(), String> {
            Ok(())
        }

        fn on_timer(&mut self, _context: &mut StrategyContext) -> Result<(), String> {
            Ok(())
        }

        fn on_stop(&mut self, _context: &mut StrategyContext) -> Result<(), String> {
            println!("Strategy {} stopped", self.name);
            Ok(())
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_strategy_context() {
        let config = StrategyConfig::default();
        let data_engine = Arc::new(Mutex::new(crate::data_engine::DataEngine::new(
            crate::data_engine::DataEngineConfig::default()
        )));
        
        let mut context = StrategyContext::new(config, data_engine);
        
        assert_eq!(context.state, StrategyState::Initialized);
        assert!(!context.is_active());
        
        context.set_state(StrategyState::Running);
        assert!(context.is_active());
        
        // Test trade recording
        let instrument_id = InstrumentId::new(123);
        context.record_trade(instrument_id, 100.0, 1.0);
        
        assert_eq!(context.metrics.total_trades, 1);
        assert_eq!(context.metrics.winning_trades, 1);
        assert_eq!(context.metrics.total_pnl, 100.0);
        assert_eq!(context.win_rate(), 1.0);
    }

    #[test]
    fn test_strategy_engine() {
        let data_engine = Arc::new(Mutex::new(crate::data_engine::DataEngine::new(
            crate::data_engine::DataEngineConfig::default()
        )));
        
        let mut engine = StrategyEngine::new(Arc::clone(&data_engine));
        
        // Add a test strategy
        let strategy = Box::new(TestStrategy::new("TestStrategy1".to_string()));
        let mut config = StrategyConfig::default();
        config.strategy_id = StrategyId::new(1);
        config.instruments = vec![InstrumentId::new(123)];
        
        engine.add_strategy(strategy, config).unwrap();
        
        assert_eq!(engine.total_strategies(), 1);
        assert!(!engine.is_running());
        
        // Start engine
        engine.start().unwrap();
        assert!(engine.is_running());
        assert_eq!(engine.active_strategies(), 1);
        
        // Stop engine
        engine.stop().unwrap();
        assert!(!engine.is_running());
    }
}
