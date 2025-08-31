use pyo3::prelude::*;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use std::collections::HashMap;
use std::str::FromStr;

// ============================================================================
// STRATEGY ENGINE PYTHON WRAPPERS
// ============================================================================

/// Python wrapper for StrategyId
#[pyclass(name = "StrategyId")]
#[derive(Clone, Debug)]
pub struct PyStrategyId {
    inner: alphaforge_core::identifiers::StrategyId,
}

#[pymethods]
impl PyStrategyId {
    #[new]
    fn new(id: u64) -> Self {
        Self {
            inner: alphaforge_core::identifiers::StrategyId::new(id),
        }
    }

    #[getter]
    fn id(&self) -> u64 {
        self.inner.id
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!("StrategyId({})", self.inner.id)
    }
}

/// Python wrapper for StrategyState
#[pyclass(name = "StrategyState")]
#[derive(Clone, Debug)]
pub struct PyStrategyState {
    inner: alphaforge_core::strategy_engine::StrategyState,
}

#[pymethods]
impl PyStrategyState {
    #[getter]
    fn name(&self) -> String {
        match self.inner {
            alphaforge_core::strategy_engine::StrategyState::Initialized => "Initialized".to_string(),
            alphaforge_core::strategy_engine::StrategyState::Running => "Running".to_string(),
            alphaforge_core::strategy_engine::StrategyState::Paused => "Paused".to_string(),
            alphaforge_core::strategy_engine::StrategyState::Stopped => "Stopped".to_string(),
            alphaforge_core::strategy_engine::StrategyState::Error => "Error".to_string(),
        }
    }

    fn __str__(&self) -> String {
        self.name()
    }

    fn __repr__(&self) -> String {
        format!("StrategyState.{}", self.name())
    }
}

/// Python wrapper for StrategyConfig
#[pyclass(name = "StrategyConfig")]
#[derive(Clone, Debug)]
pub struct PyStrategyConfig {
    inner: alphaforge_core::strategy_engine::StrategyConfig,
}

#[pymethods]
impl PyStrategyConfig {
    #[new]
    #[pyo3(signature = (
        strategy_id,
        name,
        instruments = vec![],
        max_position_size = 1000.0,
        max_daily_loss = 10000.0,
        max_drawdown = 0.05,
        enable_logging = true,
        enable_metrics = true,
        enable_backtesting = false
    ))]
    fn new(
        strategy_id: PyStrategyId,
        name: String,
        instruments: Vec<String>,
        max_position_size: f64,
        max_daily_loss: f64,
        max_drawdown: f64,
        enable_logging: bool,
        enable_metrics: bool,
        enable_backtesting: bool,
    ) -> PyResult<Self> {
        use alphaforge_core::identifiers::InstrumentId;

        let instrument_ids: Result<Vec<InstrumentId>, String> = instruments
            .iter()
            .map(|id| InstrumentId::from_str(id))
            .collect();

        let instrument_ids = instrument_ids
            .map_err(|e| PyValueError::new_err(format!("Invalid instrument ID: {}", e)))?;

        Ok(Self {
            inner: alphaforge_core::strategy_engine::StrategyConfig {
                strategy_id: strategy_id.inner,
                name,
                instruments: instrument_ids,
                max_position_size,
                max_daily_loss,
                max_drawdown,
                enable_logging,
                enable_metrics,
                enable_backtesting,
            },
        })
    }

    #[getter]
    fn strategy_id(&self) -> PyStrategyId {
        PyStrategyId { inner: self.inner.strategy_id }
    }

    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    #[getter]
    fn instruments(&self) -> Vec<String> {
        self.inner.instruments.iter().map(|id| id.to_string()).collect()
    }

    #[getter]
    fn max_position_size(&self) -> f64 {
        self.inner.max_position_size
    }

    #[getter]
    fn max_daily_loss(&self) -> f64 {
        self.inner.max_daily_loss
    }

    #[getter]
    fn max_drawdown(&self) -> f64 {
        self.inner.max_drawdown
    }

    #[getter]
    fn enable_logging(&self) -> bool {
        self.inner.enable_logging
    }

    #[getter]
    fn enable_metrics(&self) -> bool {
        self.inner.enable_metrics
    }

    #[getter]
    fn enable_backtesting(&self) -> bool {
        self.inner.enable_backtesting
    }
}

/// Python wrapper for StrategyMetrics
#[pyclass(name = "StrategyMetrics")]
#[derive(Clone, Debug)]
pub struct PyStrategyMetrics {
    inner: alphaforge_core::strategy_engine::StrategyMetrics,
}

#[pymethods]
impl PyStrategyMetrics {
    #[getter]
    fn total_trades(&self) -> u64 {
        self.inner.total_trades
    }

    #[getter]
    fn winning_trades(&self) -> u64 {
        self.inner.winning_trades
    }

    #[getter]
    fn losing_trades(&self) -> u64 {
        self.inner.losing_trades
    }

    #[getter]
    fn total_pnl(&self) -> f64 {
        self.inner.total_pnl
    }

    #[getter]
    fn gross_profit(&self) -> f64 {
        self.inner.gross_profit
    }

    #[getter]
    fn gross_loss(&self) -> f64 {
        self.inner.gross_loss
    }

    #[getter]
    fn max_consecutive_wins(&self) -> u64 {
        self.inner.max_consecutive_wins
    }

    #[getter]
    fn max_consecutive_losses(&self) -> u64 {
        self.inner.max_consecutive_losses
    }

    #[getter]
    fn max_drawdown(&self) -> f64 {
        self.inner.max_drawdown
    }

    #[getter]
    fn sharpe_ratio(&self) -> f64 {
        self.inner.sharpe_ratio
    }

    #[getter]
    fn open_positions(&self) -> HashMap<String, f64> {
        self.inner
            .open_positions
            .iter()
            .map(|(id, size)| (id.to_string(), *size))
            .collect()
    }

    #[getter]
    fn uptime_seconds(&self) -> u64 {
        self.inner.uptime_seconds
    }

    #[getter]
    fn last_update_ts(&self) -> u64 {
        self.inner.last_update_ts
    }

    /// Calculate win rate
    fn win_rate(&self) -> f64 {
        if self.inner.total_trades == 0 {
            0.0
        } else {
            self.inner.winning_trades as f64 / self.inner.total_trades as f64
        }
    }

    /// Calculate profit factor
    fn profit_factor(&self) -> f64 {
        if self.inner.gross_loss == 0.0 {
            f64::INFINITY
        } else {
            self.inner.gross_profit / self.inner.gross_loss
        }
    }

    fn __str__(&self) -> String {
        format!(
            "StrategyMetrics(trades={}, pnl={:.2}, win_rate={:.2}%)", 
            self.inner.total_trades,
            self.inner.total_pnl,
            self.win_rate() * 100.0
        )
    }
}

/// Base Python Strategy class that users can inherit from
#[pyclass(name = "Strategy", subclass)]
pub struct PyStrategy {
    name: String,
    version: String,
}

#[pymethods]
impl PyStrategy {
    #[new]
    #[pyo3(signature = (name, version = None))]
    fn new(name: String, version: Option<String>) -> Self {
        Self {
            name,
            version: version.unwrap_or_else(|| "1.0.0".to_string()),
        }
    }

    #[getter]
    fn name(&self) -> String {
        self.name.clone()
    }

    #[getter]
    fn version(&self) -> String {
        self.version.clone()
    }

    /// Override this method in your strategy
    fn on_start(&mut self, _py: Python) -> PyResult<()> {
        // Default implementation - override in Python
        Ok(())
    }

    /// Override this method in your strategy
    fn on_trade_tick(&mut self, _py: Python, _tick: &crate::data_engine::PyTradeTick) -> PyResult<()> {
        // Default implementation - override in Python
        Ok(())
    }

    /// Override this method in your strategy
    fn on_quote_tick(&mut self, _py: Python, _tick: &crate::data_engine::PyQuoteTick) -> PyResult<()> {
        // Default implementation - override in Python
        Ok(())
    }

    /// Override this method in your strategy
    fn on_bar(&mut self, _py: Python, _bar: &crate::data_engine::PyBar) -> PyResult<()> {
        // Default implementation - override in Python
        Ok(())
    }

    /// Override this method in your strategy
    fn on_timer(&mut self, _py: Python) -> PyResult<()> {
        // Default implementation - override in Python
        Ok(())
    }

    /// Override this method in your strategy
    fn on_stop(&mut self, _py: Python) -> PyResult<()> {
        // Default implementation - override in Python
        Ok(())
    }
}

/// Python wrapper for StrategyContext
#[pyclass(name = "StrategyContext")]
pub struct PyStrategyContext {
    // We'll store minimal data here and access the real context through the engine
    strategy_id: PyStrategyId,
    state: PyStrategyState,
}

#[pymethods]
impl PyStrategyContext {
    #[getter]
    fn strategy_id(&self) -> PyStrategyId {
        self.strategy_id.clone()
    }

    #[getter]
    fn state(&self) -> PyStrategyState {
        self.state.clone()
    }

    /// Check if strategy is active
    fn is_active(&self) -> bool {
        matches!(
            self.state.inner,
            alphaforge_core::strategy_engine::StrategyState::Running
        )
    }

    /// Get current timestamp in nanoseconds
    fn current_time_ns(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos() as u64
    }
}

/// Simple strategy engine wrapper for Python
#[pyclass(name = "StrategyEngine")]
pub struct PyStrategyEngine {
    // We'll keep a simplified version for now - just track strategy metadata
    strategy_configs: HashMap<u64, PyStrategyConfig>,
    is_running: bool,
}

#[pymethods]
impl PyStrategyEngine {
    #[new]
    fn new() -> Self {
        Self {
            strategy_configs: HashMap::new(),
            is_running: false,
        }
    }

    /// Add a strategy to the engine
    fn add_strategy(&mut self, strategy_id: u64, config: PyStrategyConfig) -> PyResult<()> {
        if self.strategy_configs.contains_key(&strategy_id) {
            return Err(PyRuntimeError::new_err(format!(
                "Strategy with ID {} already exists", 
                strategy_id
            )));
        }

        self.strategy_configs.insert(strategy_id, config);
        Ok(())
    }

    /// Start the strategy engine
    fn start(&mut self) -> PyResult<()> {
        if self.is_running {
            return Err(PyRuntimeError::new_err("Strategy engine is already running"));
        }

        self.is_running = true;
        Ok(())
    }

    /// Stop the strategy engine
    fn stop(&mut self) {
        self.is_running = false;
    }

    /// Check if engine is running
    fn is_running(&self) -> bool {
        self.is_running
    }

    /// Get total number of strategies
    fn total_strategies(&self) -> usize {
        self.strategy_configs.len()
    }

    /// Get strategy config by ID
    fn get_strategy_config(&self, strategy_id: u64) -> Option<PyStrategyConfig> {
        self.strategy_configs.get(&strategy_id).cloned()
    }
}

/// Register strategy engine module
pub fn register_strategy_engine_module(py: Python, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let strategy_module = PyModule::new_bound(py, "strategy")?;
    
    // Add strategy engine classes
    strategy_module.add_class::<PyStrategyId>()?;
    strategy_module.add_class::<PyStrategyState>()?;
    strategy_module.add_class::<PyStrategyConfig>()?;
    strategy_module.add_class::<PyStrategyMetrics>()?;
    strategy_module.add_class::<PyStrategy>()?;
    strategy_module.add_class::<PyStrategyContext>()?;
    strategy_module.add_class::<PyStrategyEngine>()?;
    
    parent.add_submodule(&strategy_module)?;
    
    // Register in sys.modules
    let sys = py.import_bound("sys")?;
    let modules = sys.getattr("modules")?;
    modules.set_item("alphaforge_pyo3.strategy", &strategy_module)?;
    
    Ok(())
}
