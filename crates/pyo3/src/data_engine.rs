use pyo3::prelude::*;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use std::str::FromStr;

// ============================================================================
// DATA ENGINE PYTHON WRAPPERS
// ============================================================================

/// Python wrapper for DataEngineConfig
#[pyclass(name = "DataEngineConfig")]
#[derive(Clone, Debug)]
pub struct PyDataEngineConfig {
    inner: alphaforge_core::data_engine::DataEngineConfig,
}

#[pymethods]
impl PyDataEngineConfig {
    #[new]
    #[pyo3(signature = (max_bars_per_instrument = 10000, max_tick_buffer_size = 1000, enable_bar_aggregation = true, enable_order_book_deltas = true, enable_statistics = true))]
    fn new(
        max_bars_per_instrument: usize,
        max_tick_buffer_size: usize,
        enable_bar_aggregation: bool,
        enable_order_book_deltas: bool,
        enable_statistics: bool,
    ) -> Self {
        Self {
            inner: alphaforge_core::data_engine::DataEngineConfig {
                max_bars_per_instrument,
                max_tick_buffer_size,
                enable_bar_aggregation,
                enable_order_book_deltas,
                enable_statistics,
            },
        }
    }

    #[getter]
    fn max_bars_per_instrument(&self) -> usize {
        self.inner.max_bars_per_instrument
    }

    #[getter]
    fn max_tick_buffer_size(&self) -> usize {
        self.inner.max_tick_buffer_size
    }

    #[getter]
    fn enable_bar_aggregation(&self) -> bool {
        self.inner.enable_bar_aggregation
    }

    #[getter]
    fn enable_order_book_deltas(&self) -> bool {
        self.inner.enable_order_book_deltas
    }

    #[getter]
    fn enable_statistics(&self) -> bool {
        self.inner.enable_statistics
    }
}

/// Python wrapper for DataEngineStatistics
#[pyclass(name = "DataEngineStatistics")]
#[derive(Clone, Debug)]
pub struct PyDataEngineStatistics {
    inner: alphaforge_core::data_engine::DataEngineStatistics,
}

#[pymethods]
impl PyDataEngineStatistics {
    #[getter]
    fn ticks_processed(&self) -> u64 {
        self.inner.ticks_processed
    }

    #[getter]
    fn bars_generated(&self) -> u64 {
        self.inner.bars_generated
    }

    #[getter]
    fn order_book_updates(&self) -> u64 {
        self.inner.order_book_updates
    }

    #[getter]
    fn processing_rate(&self) -> f64 {
        self.inner.processing_rate
    }

    #[getter]
    fn memory_usage(&self) -> usize {
        self.inner.memory_usage
    }

    #[getter]
    fn cache_hit_rate(&self) -> f64 {
        self.inner.cache_hit_rate
    }
}

/// Python wrapper for TradeTick
#[pyclass(name = "TradeTick")]
#[derive(Clone, Debug)]
pub struct PyTradeTick {
    inner: alphaforge_core::data::TradeTick,
}

#[pymethods]
impl PyTradeTick {
    #[new]
    fn new(
        instrument_id: String,
        price: f64,
        size: f64,
        aggressor_side: u8, // 0=Buyer, 1=Seller, 2=NoAggressor
        trade_id: String,
        ts_event: u64,
        ts_init: u64,
    ) -> PyResult<Self> {
        use alphaforge_core::data::AggressorSide;
        use alphaforge_core::identifiers::InstrumentId;

        let aggressor = match aggressor_side {
            0 => AggressorSide::Buyer,
            1 => AggressorSide::Seller,
            2 => AggressorSide::NoAggressor,
            _ => return Err(PyValueError::new_err("Invalid aggressor_side")),
        };

        Ok(Self {
            inner: alphaforge_core::data::TradeTick {
                instrument_id: InstrumentId::from_str(&instrument_id)
                    .map_err(|e| PyValueError::new_err(format!("Invalid instrument_id: {}", e)))?,
                price,
                size,
                aggressor_side: aggressor,
                trade_id,
                ts_event,
                ts_init,
            },
        })
    }

    #[getter]
    fn instrument_id(&self) -> String {
        self.inner.instrument_id.to_string()
    }

    #[getter]
    fn price(&self) -> f64 {
        self.inner.price
    }

    #[getter]
    fn size(&self) -> f64 {
        self.inner.size
    }

    #[getter]
    fn trade_id(&self) -> String {
        self.inner.trade_id.clone()
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.ts_event
    }

    #[getter]
    fn ts_init(&self) -> u64 {
        self.inner.ts_init
    }
}

/// Python wrapper for QuoteTick
#[pyclass(name = "QuoteTick")]
#[derive(Clone, Debug)]
pub struct PyQuoteTick {
    inner: alphaforge_core::data::QuoteTick,
}

#[pymethods]
impl PyQuoteTick {
    #[new]
    fn new(
        instrument_id: String,
        bid_price: f64,
        ask_price: f64,
        bid_size: f64,
        ask_size: f64,
        ts_event: u64,
        ts_init: u64,
    ) -> PyResult<Self> {
        use alphaforge_core::identifiers::InstrumentId;

        Ok(Self {
            inner: alphaforge_core::data::QuoteTick {
                instrument_id: InstrumentId::from_str(&instrument_id)
                    .map_err(|e| PyValueError::new_err(format!("Invalid instrument_id: {}", e)))?,
                bid_price,
                ask_price,
                bid_size,
                ask_size,
                ts_event,
                ts_init,
            },
        })
    }

    #[getter]
    fn instrument_id(&self) -> String {
        self.inner.instrument_id.to_string()
    }

    #[getter]
    fn bid_price(&self) -> f64 {
        self.inner.bid_price
    }

    #[getter]
    fn ask_price(&self) -> f64 {
        self.inner.ask_price
    }

    #[getter]
    fn bid_size(&self) -> f64 {
        self.inner.bid_size
    }

    #[getter]
    fn ask_size(&self) -> f64 {
        self.inner.ask_size
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.ts_event
    }

    #[getter]
    fn ts_init(&self) -> u64 {
        self.inner.ts_init
    }
}

/// Python wrapper for Bar
#[pyclass(name = "Bar")]
#[derive(Clone, Debug)]
pub struct PyBar {
    inner: alphaforge_core::data::Bar,
}

#[pymethods]
impl PyBar {
    #[getter]
    fn open(&self) -> f64 {
        self.inner.open
    }

    #[getter]
    fn high(&self) -> f64 {
        self.inner.high
    }

    #[getter]
    fn low(&self) -> f64 {
        self.inner.low
    }

    #[getter]
    fn close(&self) -> f64 {
        self.inner.close
    }

    #[getter]
    fn volume(&self) -> f64 {
        self.inner.volume
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.ts_event
    }

    #[getter]
    fn ts_init(&self) -> u64 {
        self.inner.ts_init
    }
}

/// Python wrapper for BarType
#[pyclass(name = "BarType")]
#[derive(Clone, Debug)]
pub struct PyBarType {
    inner: alphaforge_core::data::BarType,
}

#[pymethods]
impl PyBarType {
    #[new]
    fn new(instrument_id: String, step: u64, aggregation: String) -> PyResult<Self> {
        use alphaforge_core::data::{BarSpecification, BarAggregation};
        use alphaforge_core::identifiers::InstrumentId;

        let aggregation_type = match aggregation.as_str() {
            "tick" => BarAggregation::Tick(step),
            "volume" => BarAggregation::Volume(step),
            "dollar" => BarAggregation::Dollar(step),
            "time" => BarAggregation::Time(step),
            _ => return Err(PyValueError::new_err("Invalid aggregation type")),
        };

        Ok(Self {
            inner: alphaforge_core::data::BarType {
                instrument_id: InstrumentId::from_str(&instrument_id)
                    .map_err(|e| PyValueError::new_err(format!("Invalid instrument_id: {}", e)))?,
                bar_spec: BarSpecification {
                    step,
                    aggregation: aggregation_type,
                },
            },
        })
    }

    #[getter]
    fn instrument_id(&self) -> String {
        self.inner.instrument_id.to_string()
    }

    #[getter]
    fn step(&self) -> u64 {
        self.inner.bar_spec.step
    }
}

/// Python wrapper for DataEngine
#[pyclass(name = "DataEngine")]
pub struct PyDataEngine {
    inner: alphaforge_core::data_engine::DataEngine,
}

#[pymethods]
impl PyDataEngine {
    #[new]
    fn new(config: PyDataEngineConfig) -> Self {
        Self {
            inner: alphaforge_core::data_engine::DataEngine::new(config.inner),
        }
    }

    /// Start the Data Engine
    fn start(&mut self) -> PyResult<()> {
        self.inner.start()
            .map_err(|e| PyRuntimeError::new_err(e))
    }

    /// Stop the Data Engine
    fn stop(&mut self) {
        self.inner.stop();
    }

    /// Process a trade tick
    fn process_trade_tick(&mut self, tick: PyTradeTick) -> PyResult<Option<PyBar>> {
        match self.inner.process_trade_tick(tick.inner) {
            Ok(Some(bar)) => Ok(Some(PyBar { inner: bar })),
            Ok(None) => Ok(None),
            Err(e) => Err(PyRuntimeError::new_err(e)),
        }
    }

    /// Process a quote tick
    fn process_quote_tick(&mut self, tick: PyQuoteTick) -> PyResult<()> {
        self.inner.process_quote_tick(tick.inner)
            .map_err(|e| PyRuntimeError::new_err(e))
    }

    /// Add bar aggregator
    fn add_bar_aggregator(&mut self, bar_type: PyBarType) {
        self.inner.add_bar_aggregator(bar_type.inner);
    }

    /// Get recent bars
    fn get_recent_bars(&self, bar_type: PyBarType, count: usize) -> Vec<PyBar> {
        self.inner.get_recent_bars(&bar_type.inner, count)
            .into_iter()
            .map(|bar| PyBar { inner: bar })
            .collect()
    }

    /// Check if engine is running
    fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    /// Get processed count
    fn processed_count(&self) -> u64 {
        self.inner.processed_count()
    }

    /// Get statistics
    fn statistics(&self) -> PyDataEngineStatistics {
        PyDataEngineStatistics {
            inner: self.inner.statistics(),
        }
    }

    /// Reset statistics
    fn reset_statistics(&mut self) {
        self.inner.reset_statistics();
    }
}

/// Register data engine module
pub fn register_data_engine_module(py: Python, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let data_module = PyModule::new_bound(py, "data")?;
    
    // Add data engine classes
    data_module.add_class::<PyDataEngine>()?;
    data_module.add_class::<PyDataEngineConfig>()?;
    data_module.add_class::<PyDataEngineStatistics>()?;
    data_module.add_class::<PyTradeTick>()?;
    data_module.add_class::<PyQuoteTick>()?;
    data_module.add_class::<PyBar>()?;
    data_module.add_class::<PyBarType>()?;
    
    parent.add_submodule(&data_module)?;
    
    // Register in sys.modules
    let sys = py.import_bound("sys")?;
    let modules = sys.getattr("modules")?;
    modules.set_item("alphaforge.core.rust.data", &data_module)?;
    
    Ok(())
}
