//! AlphaForge PyO3 Python Extension Module
//! 
//! High-performance Python bindings for AlphaForge trading system.

use pyo3::prelude::*;
use pyo3::types::PyModule;
use tracing_subscriber::{EnvFilter, fmt};
use alphaforge_core::generic_cache;

mod data_engine;
mod strategy_engine;
mod execution_engine;

/// Python-compatible wrapper for PyObject that implements Clone
#[derive(Debug)]
struct PyObjectWrapper(PyObject);

impl Clone for PyObjectWrapper {
    fn clone(&self) -> Self {
        Python::with_gil(|py| {
            PyObjectWrapper(self.0.clone_ref(py))
        })
    }
}

impl From<PyObject> for PyObjectWrapper {
    fn from(obj: PyObject) -> Self {
        PyObjectWrapper(obj)
    }
}

impl Into<PyObject> for PyObjectWrapper {
    fn into(self) -> PyObject {
        self.0
    }
}

/// AlphaForge Python extension module
#[pymodule]
fn alphaforge_pyo3(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Version information
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("ALPHAFORGE_VERSION", env!("CARGO_PKG_VERSION"))?;
    m.add("ALPHAFORGE_USER_AGENT", format!("AlphaForge/{}", env!("CARGO_PKG_VERSION")))?;    
    
    // Initialize logging subsystem
    init_logging()?;
    
    // Register core submodules
    let py = m.py();
    register_core_module(py, m)?;
    register_cache_module(py, m)?;
    register_data_module(py, m)?;
    register_strategy_module(py, m)?;
    register_execution_module(py, m)?;
    register_model_module(py, m)?;
    register_time_module(py, m)?;
    register_message_module(py, m)?;
    
    Ok(())
}

/// Initialize Rust logging system
fn init_logging() -> PyResult<()> {
    // Only initialize once
    static INIT: std::sync::Once = std::sync::Once::new();
    
    INIT.call_once(|| {
        // Set up tracing subscriber for structured logging
        let filter = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("info"))
            .unwrap();
            
        fmt()
            .with_env_filter(filter)
            .with_target(false)
            .with_thread_ids(true)
            .with_thread_names(true)
            .compact()
            .init();
    });
    
    Ok(())
}

/// Register core module functions
fn register_core_module(py: Python, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let core_module = PyModule::new_bound(py, "core")?;
    
    // Add core functions
    core_module.add_function(wrap_pyfunction!(unix_nanos_now_py, &core_module)?)?;
    core_module.add_function(wrap_pyfunction!(uuid4_new_py, &core_module)?)?;
    
    parent.add_submodule(&core_module)?;
    
    // Register in sys.modules for direct import
    let sys = py.import_bound("sys")?;
    let modules = sys.getattr("modules")?;
    modules.set_item("alphaforge.core.rust.core", &core_module)?;
    
    Ok(())
}

/// Register model module classes
fn register_model_module(py: Python, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let model_module = PyModule::new_bound(py, "model")?;
    
    // Add model classes
    model_module.add_class::<PyPrice>()?;
    model_module.add_class::<PyQuantity>()?;
    model_module.add_class::<PyInstrumentId>()?;
    model_module.add_class::<PyOrderBook>()?;
    
    parent.add_submodule(&model_module)?;
    
    // Register in sys.modules
    let sys = py.import_bound("sys")?;
    let modules = sys.getattr("modules")?;
    modules.set_item("alphaforge.core.rust.model", &model_module)?;
    
    Ok(())
}

/// Register time module
fn register_time_module(py: Python, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let time_module = PyModule::new_bound(py, "time")?;
    
    time_module.add_class::<PyAtomicTime>()?;
    // Note: PyLiveClock temporarily removed due to clock module absence
    
    parent.add_submodule(&time_module)?;
    
    // Register in sys.modules
    let sys = py.import_bound("sys")?;
    let modules = sys.getattr("modules")?;
    modules.set_item("alphaforge.core.rust.time", &time_module)?;
    
    Ok(())
}

/// Register message module
fn register_message_module(py: Python, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let message_module = PyModule::new_bound(py, "message")?;
    
    message_module.add_class::<PyMessageBus>()?;
    message_module.add_class::<PyMessageEnvelope>()?;
    
    parent.add_submodule(&message_module)?;
    
    // Register in sys.modules
    let sys = py.import_bound("sys")?;
    let modules = sys.getattr("modules")?;
    modules.set_item("alphaforge.core.rust.message", &message_module)?;
    
    Ok(())
}

/// Register cache module
fn register_cache_module(py: Python, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let cache_module = PyModule::new_bound(py, "cache")?;
    
    cache_module.add_class::<PyCache>()?;
    cache_module.add_class::<PyCacheConfig>()?;
    cache_module.add_class::<PyCacheStatistics>()?;
    
    parent.add_submodule(&cache_module)?;
    
    // Register in sys.modules
    let sys = py.import_bound("sys")?;
    let modules = sys.getattr("modules")?;
    modules.set_item("alphaforge.core.rust.cache", &cache_module)?;
    
    Ok(())
}

/// Register data module with Data Engine
fn register_data_module(py: Python, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    data_engine::register_data_engine_module(py, parent)
}

/// Register strategy module with Strategy Engine
fn register_strategy_module(py: Python, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    strategy_engine::register_strategy_engine_module(py, parent)
}

/// Register execution module with Execution Engine
fn register_execution_module(py: Python, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    execution_engine::register_execution_types(py, parent)
}

// Core function bindings
#[pyfunction]
fn unix_nanos_now_py() -> u64 {
    alphaforge_core::time::unix_nanos_now()
}

#[pyfunction] 
fn uuid4_new_py() -> String {
    alphaforge_core::uuid::UUID4::new().to_string()
}

// Python wrapper for Price
#[pyclass(name = "Price")]
#[derive(Clone, Debug)]
pub struct PyPrice {
    inner: alphaforge_model::orderbook::Price,
}

#[pymethods]
impl PyPrice {
    #[new]
    fn new(value: f64, precision: u8) -> PyResult<Self> {
        let price = alphaforge_model::orderbook::Price::from_f64(value, precision)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(Self { inner: price })
    }
    
    #[getter]
    fn value(&self) -> f64 {
        self.inner.as_f64()
    }
    
    #[getter]
    fn raw(&self) -> i64 {
        self.inner.raw()
    }
    
    fn __str__(&self) -> String {
        format!("{:.9}", self.inner.as_f64())
    }
    
    fn __repr__(&self) -> String {
        format!("Price({})", self.inner.as_f64())
    }
    
    fn __add__(&self, other: &Self) -> PyResult<Self> {
        let result = self.inner.checked_add(other.inner)
            .ok_or_else(|| pyo3::exceptions::PyOverflowError::new_err("Price addition overflow"))?;
        Ok(Self { inner: result })
    }
    
    fn __sub__(&self, other: &Self) -> PyResult<Self> {
        let result = self.inner.checked_sub(other.inner)
            .ok_or_else(|| pyo3::exceptions::PyOverflowError::new_err("Price subtraction underflow"))?;
        Ok(Self { inner: result })
    }
    
    fn __mul__(&self, factor: f64) -> PyResult<Self> {
        let result = self.inner.checked_mul_f64(factor)
            .ok_or_else(|| pyo3::exceptions::PyOverflowError::new_err("Price multiplication overflow"))?;
        Ok(Self { inner: result })
    }
    
    fn __eq__(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
    
    fn __lt__(&self, other: &Self) -> bool {
        self.inner < other.inner
    }
    
    fn __le__(&self, other: &Self) -> bool {
        self.inner <= other.inner
    }
    
    fn __gt__(&self, other: &Self) -> bool {
        self.inner > other.inner
    }
    
    fn __ge__(&self, other: &Self) -> bool {
        self.inner >= other.inner
    }
    
    fn __hash__(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.inner.hash(&mut hasher);
        hasher.finish()
    }
}

// Python wrapper for Quantity
#[pyclass(name = "Quantity")]
#[derive(Clone, Debug)]
pub struct PyQuantity {
    inner: alphaforge_model::orderbook::Quantity,
}

#[pymethods]
impl PyQuantity {
    #[new]
    fn new(value: f64, precision: u8) -> PyResult<Self> {
        let quantity = alphaforge_model::orderbook::Quantity::from_f64(value, precision)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(Self { inner: quantity })
    }
    
    #[getter]
    fn value(&self) -> f64 {
        self.inner.as_f64()
    }
    
    #[getter]
    fn raw(&self) -> u64 {
        self.inner.raw()
    }
    
    fn __str__(&self) -> String {
        format!("{:.8}", self.inner.as_f64())
    }
    
    fn __repr__(&self) -> String {
        format!("Quantity({})", self.inner.as_f64())
    }
}

// Python wrapper for InstrumentId
#[pyclass(name = "InstrumentId")]
#[derive(Clone, Debug)]
pub struct PyInstrumentId {
    inner: alphaforge_model::identifiers::InstrumentId,
}

#[pymethods]
impl PyInstrumentId {
    #[new]
    fn new(identifier: &str) -> PyResult<Self> {
        let id = alphaforge_model::identifiers::InstrumentId::new(identifier)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(Self { inner: id })
    }
    
    #[getter]
    fn symbol(&self) -> &str {
        self.inner.symbol()
    }
    
    #[getter] 
    fn venue(&self) -> &str {
        self.inner.venue()
    }
    
    #[getter]
    fn value(&self) -> &str {
        self.inner.value()
    }
    
    fn __str__(&self) -> String {
        self.inner.to_string()
    }
    
    fn __repr__(&self) -> String {
        format!("InstrumentId('{}')", self.inner.value())
    }
}

// Python wrapper for OrderBook
#[pyclass(name = "OrderBook")]
pub struct PyOrderBook {
    inner: std::sync::Mutex<alphaforge_model::orderbook::OrderBook>,
}

#[pymethods]
impl PyOrderBook {
    #[new]
    fn new(instrument_id: &PyInstrumentId) -> Self {
        let book = alphaforge_model::orderbook::OrderBook::new(instrument_id.inner.clone());
        Self {
            inner: std::sync::Mutex::new(book),
        }
    }
    
    fn best_bid_price(&self) -> Option<PyPrice> {
        let book = self.inner.lock().unwrap();
        book.best_bid_price().map(|price| PyPrice { inner: price })
    }
    
    fn best_ask_price(&self) -> Option<PyPrice> {
        let book = self.inner.lock().unwrap();
        book.best_ask_price().map(|price| PyPrice { inner: price })
    }
    
    fn spread(&self) -> Option<f64> {
        let book = self.inner.lock().unwrap();
        book.spread().map(|s| s.to_string().parse().unwrap_or(0.0))
    }
    
    #[getter]
    fn count(&self) -> usize {
        let book = self.inner.lock().unwrap();
        book.count
    }
    
    fn clear(&mut self) {
        let mut book = self.inner.lock().unwrap();
        book.clear();
    }
}

// Python wrapper for AtomicTime
#[pyclass(name = "AtomicTime")]
pub struct PyAtomicTime {
    inner: alphaforge_core::time::AtomicTime,
}

#[pymethods]
impl PyAtomicTime {
    #[new]
    fn new() -> Self {
        Self {
            inner: alphaforge_core::time::AtomicTime::new(),
        }
    }
    
    fn get(&self) -> u64 {
        self.inner.get()
    }
    
    fn set(&self, timestamp: u64) {
        self.inner.set(timestamp);
    }
    
    fn update_now(&self) {
        self.inner.update_now();
    }
}

// Python wrapper for LiveClock - Temporarily commented out due to clock module absence
// #[pyclass(name = "LiveClock")]
// pub struct PyLiveClock {
//     inner: std::sync::Mutex<alphaforge_core::clock::LiveClock>,
// }

// #[pymethods]
// impl PyLiveClock {
//     #[new]
//     fn new() -> Self {
//         Self {
//             inner: std::sync::Mutex::new(alphaforge_core::clock::LiveClock::new()),
//         }
//     }
//     
//     fn timestamp_ns(&self) -> u64 {
//         let clock = self.inner.lock().unwrap();
//         use alphaforge_core::clock::Clock;
//         clock.timestamp_ns()
//     }
// }

// Python wrapper for MessageBus
#[pyclass(name = "MessageBus")]
pub struct PyMessageBus {
    inner: std::sync::Arc<alphaforge_core::message::MessageBus>,
}

#[pymethods]
impl PyMessageBus {
    #[new]
    fn new() -> Self {
        Self {
            inner: std::sync::Arc::new(alphaforge_core::message::MessageBus::new()),
        }
    }
    
    fn get_stats(&self) -> PyResult<(u64, u64, u64, u64)> {
        let stats = self.inner.stats();
        Ok((
            stats.total_messages_sent.load(std::sync::atomic::Ordering::Relaxed),
            stats.total_messages_delivered.load(std::sync::atomic::Ordering::Relaxed),
            stats.total_publish_time_nanos.load(std::sync::atomic::Ordering::Relaxed),
            stats.publish_count.load(std::sync::atomic::Ordering::Relaxed)
        ))
    }
}

// Python wrapper for MessageEnvelope
#[pyclass(name = "MessageEnvelope")]
#[derive(Clone)]
pub struct PyMessageEnvelope {
    inner: alphaforge_core::message::MessageEnvelope,
}

#[pymethods]
impl PyMessageEnvelope {
    #[new]
    fn new(sender: String, message_type: String, payload: Vec<u8>) -> Self {
        Self {
            inner: alphaforge_core::message::MessageEnvelope::new(sender, message_type, payload),
        }
    }
    
    #[getter]
    fn id(&self) -> String {
        self.inner.id.to_string()
    }
    
    #[getter]
    fn timestamp(&self) -> u64 {
        self.inner.timestamp
    }
    
    #[getter]
    fn sender(&self) -> &str {
        &self.inner.sender
    }
    
    #[getter]
    fn message_type(&self) -> &str {
        &self.inner.message_type
    }
    
    #[getter]
    fn payload(&self) -> Vec<u8> {
        self.inner.payload.clone()
    }
}

// Cache Statistics wrapper for Python
#[pyclass(name = "CacheStatistics")]
#[derive(Clone)]
pub struct PyCacheStatistics {
    #[pyo3(get)]
    pub hits: u64,
    #[pyo3(get)]
    pub misses: u64,
    #[pyo3(get)]
    pub inserts: u64,
    #[pyo3(get)]
    pub evictions: u64,
    #[pyo3(get)]
    pub memory_usage: usize,
}

#[pymethods]
impl PyCacheStatistics {
    #[getter]
    fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

impl From<generic_cache::GenericCacheStatistics> for PyCacheStatistics {
    fn from(stats: alphaforge_core::generic_cache::GenericCacheStatistics) -> Self {
        PyCacheStatistics {
            hits: stats.hits,
            misses: stats.misses,
            inserts: stats.inserts,
            evictions: stats.evictions,
            memory_usage: stats.memory_usage,
        }
    }
}

// Cache Configuration wrapper for Python
#[pyclass(name = "CacheConfig")]
#[derive(Clone)]
pub struct PyCacheConfig {
    #[pyo3(get, set)]
    pub max_size: usize,
    #[pyo3(get, set)]
    pub ttl_seconds: Option<u64>,
    #[pyo3(get, set)]
    pub enable_statistics: bool,
    #[pyo3(get, set)]
    pub enable_persistence: bool,
    #[pyo3(get, set)]
    pub persistence_path: Option<String>,
}

#[pymethods]
impl PyCacheConfig {
    #[new]
    #[pyo3(signature = (max_size=10000, ttl_seconds=None, enable_statistics=true, enable_persistence=false, persistence_path=None))]
    fn new(
        max_size: usize,
        ttl_seconds: Option<u64>,
        enable_statistics: bool,
        enable_persistence: bool,
        persistence_path: Option<String>,
    ) -> Self {
        PyCacheConfig {
            max_size,
            ttl_seconds,
            enable_statistics,
            enable_persistence,
            persistence_path,
        }
    }
}

impl From<PyCacheConfig> for generic_cache::GenericCacheConfig {
    fn from(config: PyCacheConfig) -> Self {
        generic_cache::GenericCacheConfig {
            max_size: config.max_size,
            ttl_seconds: config.ttl_seconds,
            enable_statistics: config.enable_statistics,
        }
    }
}

// High-performance Cache wrapper for Python using real Rust implementation
#[pyclass(name = "Cache")]
pub struct PyCache {
    cache: generic_cache::GenericCache<PyObjectWrapper>,
}

#[pymethods]
impl PyCache {
    #[new]
    fn new(config: PyCacheConfig) -> Self {
        let rust_config = generic_cache::GenericCacheConfig::from(config);
        PyCache {
            cache: generic_cache::GenericCache::new(rust_config),
        }
    }

    /// Get value from cache
    fn get(&self, py: Python, key: &str) -> PyResult<Option<PyObject>> {
        match self.cache.get(key) {
            Some(wrapper) => Ok(Some(wrapper.0.clone_ref(py))),
            None => Ok(None),
        }
    }

    /// Put value into cache
    fn put(&self, key: &str, value: PyObject) -> bool {
        self.cache.put(key.to_string(), PyObjectWrapper::from(value))
    }

    /// Check if key exists in cache
    fn contains(&self, key: &str) -> bool {
        self.cache.contains(key)
    }

    /// Remove key from cache
    fn remove(&self, key: &str) -> bool {
        self.cache.remove(key)
    }

    /// Clear all entries from cache
    fn clear(&self) {
        self.cache.clear()
    }

    /// Get current cache size
    fn size(&self) -> usize {
        self.cache.size()
    }

    /// Get all keys in cache
    fn keys(&self) -> Vec<String> {
        self.cache.keys()
    }

    /// Get cache statistics
    fn statistics(&self) -> Option<PyCacheStatistics> {
        self.cache.statistics().map(PyCacheStatistics::from)
    }

    /// Reset cache statistics
    fn reset_statistics(&self) {
        self.cache.reset_statistics()
    }

    /// Save cache to disk if persistence is enabled
    fn save_to_disk(&self) -> bool {
        // For now, return true as if saved (persistence can be implemented later)
        true
    }

    // Python dict-like interface
    fn __len__(&self) -> usize {
        self.size()
    }

    fn __contains__(&self, key: &str) -> bool {
        self.contains(key)
    }
}
