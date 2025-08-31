use pyo3::prelude::*;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use std::sync::Arc;
use alphaforge_core::execution_engine::{
    ExecutionEngine, Order, OrderSide, OrderType, OrderStatus, 
    TimeInForce, Fill, ExecutionStats
};
use alphaforge_core::identifiers::{StrategyId, InstrumentId, OrderId};
use alphaforge_core::message_bus::MessageBus;
use std::str::FromStr;

// ============================================================================
// PYTHON WRAPPERS FOR ORDER TYPES
// ============================================================================

/// Python wrapper for OrderSide
#[pyclass(name = "OrderSide")]
#[derive(Clone)]
pub struct PyOrderSide {
    pub inner: OrderSide,
}

#[pymethods]
impl PyOrderSide {
    #[classattr]
    const BUY: u8 = 0;
    
    #[classattr]  
    const SELL: u8 = 1;
    
    #[new]
    fn new(side: u8) -> PyResult<Self> {
        let inner = match side {
            0 => OrderSide::Buy,
            1 => OrderSide::Sell,
            _ => return Err(PyValueError::new_err("Invalid order side")),
        };
        Ok(Self { inner })
    }
    
    fn __str__(&self) -> String {
        format!("{:?}", self.inner)
    }
}

/// Python wrapper for OrderType
#[pyclass(name = "OrderType")]
#[derive(Clone)]
pub struct PyOrderType {
    pub inner: OrderType,
}

#[pymethods]
impl PyOrderType {
    #[classattr]
    const MARKET: u8 = 0;
    
    #[classattr]
    const LIMIT: u8 = 1;
    
    #[classattr]
    const STOP: u8 = 2;
    
    #[classattr]
    const STOP_LIMIT: u8 = 3;
    
    #[new]
    fn new(order_type: u8) -> PyResult<Self> {
        let inner = match order_type {
            0 => OrderType::Market,
            1 => OrderType::Limit,
            2 => OrderType::Stop,
            3 => OrderType::StopLimit,
            _ => return Err(PyValueError::new_err("Invalid order type")),
        };
        Ok(Self { inner })
    }
    
    fn __str__(&self) -> String {
        format!("{:?}", self.inner)
    }
}

/// Python wrapper for OrderStatus
#[pyclass(name = "OrderStatus")]
#[derive(Clone)]
pub struct PyOrderStatus {
    pub inner: OrderStatus,
}

#[pymethods]
impl PyOrderStatus {
    #[classattr]
    const INITIALIZED: u8 = 0;
    
    #[classattr]
    const SUBMITTED: u8 = 1;
    
    #[classattr]
    const ACCEPTED: u8 = 2;
    
    #[classattr]
    const PARTIALLY_FILLED: u8 = 3;
    
    #[classattr]
    const FILLED: u8 = 4;
    
    #[classattr]
    const CANCELLED: u8 = 5;
    
    #[classattr]
    const REJECTED: u8 = 6;
    
    #[classattr]
    const EXPIRED: u8 = 7;
    
    fn __str__(&self) -> String {
        format!("{:?}", self.inner)
    }
}

/// Python wrapper for TimeInForce
#[pyclass(name = "TimeInForce")]
#[derive(Clone)]
pub struct PyTimeInForce {
    pub inner: TimeInForce,
}

#[pymethods]
impl PyTimeInForce {
    #[classattr]
    const GTC: u8 = 0; // Good Till Cancelled
    
    #[classattr]
    const IOC: u8 = 1; // Immediate Or Cancel
    
    #[classattr]
    const FOK: u8 = 2; // Fill Or Kill
    
    #[classattr]
    const GTD: u8 = 3; // Good Till Date
    
    #[classattr]
    const DAY: u8 = 4; // Day order
    
    #[new]
    fn new(tif: u8) -> PyResult<Self> {
        let inner = match tif {
            0 => TimeInForce::GTC,
            1 => TimeInForce::IOC,
            2 => TimeInForce::FOK,
            3 => TimeInForce::GTD,
            4 => TimeInForce::DAY,
            _ => return Err(PyValueError::new_err("Invalid time in force")),
        };
        Ok(Self { inner })
    }
    
    fn __str__(&self) -> String {
        format!("{:?}", self.inner)
    }
}

// ============================================================================
// PYTHON WRAPPER FOR ORDER
// ============================================================================

/// Python wrapper for Order
#[pyclass(name = "Order")]
#[derive(Clone)]
pub struct PyOrder {
    pub inner: Order,
}

#[pymethods]
impl PyOrder {
    /// Create a new market order
    #[staticmethod]
    fn market(
        strategy_id: u64,
        instrument_id: String,
        side: PyOrderSide,
        quantity: f64,
    ) -> PyResult<Self> {
        let strategy_id = StrategyId::new(strategy_id);
        let instrument_id = InstrumentId::from_str(&instrument_id)
            .map_err(|e| PyValueError::new_err(format!("Invalid instrument ID: {}", e)))?;
            
        let order = Order::market(strategy_id, instrument_id, side.inner, quantity);
        Ok(Self { inner: order })
    }
    
    /// Create a new limit order
    #[staticmethod]
    fn limit(
        strategy_id: u64,
        instrument_id: String,
        side: PyOrderSide,
        quantity: f64,
        price: f64,
    ) -> PyResult<Self> {
        let strategy_id = StrategyId::new(strategy_id);
        let instrument_id = InstrumentId::from_str(&instrument_id)
            .map_err(|e| PyValueError::new_err(format!("Invalid instrument ID: {}", e)))?;
            
        let order = Order::limit(strategy_id, instrument_id, side.inner, quantity, price);
        Ok(Self { inner: order })
    }
    
    #[getter]
    fn order_id(&self) -> u64 {
        self.inner.order_id.id
    }
    
    #[getter]
    fn strategy_id(&self) -> u64 {
        self.inner.strategy_id.id
    }
    
    #[getter]
    fn instrument_id(&self) -> String {
        self.inner.instrument_id.to_string()
    }
    
    #[getter]
    fn side(&self) -> PyOrderSide {
        PyOrderSide { inner: self.inner.side }
    }
    
    #[getter]
    fn order_type(&self) -> PyOrderType {
        PyOrderType { inner: self.inner.order_type }
    }
    
    #[getter]
    fn quantity(&self) -> f64 {
        self.inner.quantity
    }
    
    #[getter]
    fn price(&self) -> Option<f64> {
        self.inner.price
    }
    
    #[getter]
    fn status(&self) -> PyOrderStatus {
        PyOrderStatus { inner: self.inner.status }
    }
    
    #[getter]
    fn filled_quantity(&self) -> f64 {
        self.inner.filled_quantity
    }
    
    #[getter]
    fn avg_fill_price(&self) -> Option<f64> {
        self.inner.avg_fill_price
    }
    
    /// Check if order is active
    fn is_active(&self) -> bool {
        self.inner.is_active()
    }
    
    /// Check if order is complete
    fn is_complete(&self) -> bool {
        self.inner.is_complete()
    }
    
    /// Get remaining quantity
    fn remaining_quantity(&self) -> f64 {
        self.inner.remaining_quantity()
    }
    
    /// Check if order is filled
    fn is_filled(&self) -> bool {
        self.inner.is_filled()
    }
    
    fn __str__(&self) -> String {
        format!("Order(id={}, instrument={}, side={:?}, quantity={}, status={:?})",
            self.inner.order_id.id,
            self.inner.instrument_id,
            self.inner.side,
            self.inner.quantity,
            self.inner.status
        )
    }
}

// ============================================================================
// PYTHON WRAPPER FOR FILL
// ============================================================================

/// Python wrapper for Fill
#[pyclass(name = "Fill")]
#[derive(Clone)]
pub struct PyFill {
    pub inner: Fill,
}

#[pymethods]
impl PyFill {
    #[new]
    fn new(
        order_id: u64,
        fill_id: String,
        price: f64,
        quantity: f64,
        commission: f64,
        commission_currency: String,
    ) -> Self {
        let fill = Fill {
            order_id: OrderId::from_u64(order_id),
            fill_id,
            price,
            quantity,
            timestamp: alphaforge_core::time::unix_nanos_now(),
            commission,
            commission_currency,
        };
        Self { inner: fill }
    }
    
    #[getter]
    fn order_id(&self) -> u64 {
        self.inner.order_id.id
    }
    
    #[getter]
    fn fill_id(&self) -> String {
        self.inner.fill_id.clone()
    }
    
    #[getter]
    fn price(&self) -> f64 {
        self.inner.price
    }
    
    #[getter]
    fn quantity(&self) -> f64 {
        self.inner.quantity
    }
    
    #[getter]
    fn commission(&self) -> f64 {
        self.inner.commission
    }
    
    #[getter]
    fn commission_currency(&self) -> String {
        self.inner.commission_currency.clone()
    }
    
    fn __str__(&self) -> String {
        format!("Fill(order_id={}, price={}, quantity={})",
            self.inner.order_id.id, self.inner.price, self.inner.quantity)
    }
}

// ============================================================================
// PYTHON WRAPPER FOR EXECUTION STATISTICS
// ============================================================================

/// Python wrapper for ExecutionStats
#[pyclass(name = "ExecutionStats")]
pub struct PyExecutionStats {
    pub inner: ExecutionStats,
}

#[pymethods]
impl PyExecutionStats {
    #[getter]
    fn orders_submitted(&self) -> u64 {
        self.inner.orders_submitted
    }
    
    #[getter]
    fn orders_filled(&self) -> u64 {
        self.inner.orders_filled
    }
    
    #[getter]
    fn orders_cancelled(&self) -> u64 {
        self.inner.orders_cancelled
    }
    
    #[getter]
    fn orders_rejected(&self) -> u64 {
        self.inner.orders_rejected
    }
    
    #[getter]
    fn total_fill_volume(&self) -> f64 {
        self.inner.total_fill_volume
    }
    
    #[getter]
    fn total_commission(&self) -> f64 {
        self.inner.total_commission
    }
    
    #[getter]
    fn avg_execution_latency_ns(&self) -> u64 {
        self.inner.avg_execution_latency_ns
    }
    
    /// Get fill rate as percentage
    fn get_fill_rate(&self) -> f64 {
        if self.inner.orders_submitted > 0 {
            (self.inner.orders_filled as f64) / (self.inner.orders_submitted as f64)
        } else {
            0.0
        }
    }
    
    fn __str__(&self) -> String {
        format!("ExecutionStats(submitted={}, filled={}, cancelled={}, fill_rate={:.2}%)",
            self.inner.orders_submitted,
            self.inner.orders_filled, 
            self.inner.orders_cancelled,
            self.get_fill_rate() * 100.0
        )
    }
}

// ============================================================================
// PYTHON WRAPPER FOR EXECUTION ENGINE
// ============================================================================

/// Python wrapper for ExecutionEngine
#[pyclass(name = "ExecutionEngine")]
pub struct PyExecutionEngine {
    inner: Arc<ExecutionEngine>,
}

#[pymethods]
impl PyExecutionEngine {
    #[new]
    fn new() -> PyResult<Self> {
        let message_bus = Arc::new(MessageBus::new());
        let inner = Arc::new(ExecutionEngine::new(message_bus));
        
        Ok(Self { inner })
    }
    
    /// Submit order for execution
    fn submit_order(&self, order: PyOrder) -> PyResult<u64> {
        // Create a Tokio runtime for async execution
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;
            
        let inner = self.inner.clone();
        let order = order.inner;
        
        rt.block_on(async move {
            let result = inner.submit_order(order).await;
            match result {
                Ok(order_id) => Ok(order_id.id),
                Err(e) => Err(PyRuntimeError::new_err(format!("Execution error: {}", e))),
            }
        })
    }
    
    /// Cancel an order
    fn cancel_order(&self, order_id: u64) -> PyResult<()> {
        // Create a Tokio runtime for async execution
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;
            
        let inner = self.inner.clone();
        let order_id = OrderId::from_u64(order_id);
        
        rt.block_on(async move {
            let result = inner.cancel_order(order_id).await;
            match result {
                Ok(()) => Ok(()),
                Err(e) => Err(PyRuntimeError::new_err(format!("Execution error: {}", e))),
            }
        })
    }
    
    /// Handle order fill
    fn handle_fill(&self, fill: PyFill) -> PyResult<()> {
        self.inner.handle_fill(fill.inner)
            .map_err(|e| PyRuntimeError::new_err(format!("Fill error: {}", e)))
    }
    
    /// Get execution statistics
    fn get_statistics(&self) -> PyExecutionStats {
        PyExecutionStats {
            inner: self.inner.get_statistics()
        }
    }
    
    /// Get orders for a strategy
    fn get_strategy_orders(&self, strategy_id: u64) -> Vec<PyOrder> {
        let strategy_id = StrategyId::new(strategy_id);
        self.inner.get_strategy_orders(strategy_id)
            .into_iter()
            .map(|order| PyOrder { inner: order })
            .collect()
    }
    
    /// Get active orders count
    fn get_active_orders_count(&self) -> usize {
        self.inner.get_active_orders_count()
    }
    
    /// Configure instrument routing
    fn configure_routing(&self, instrument_id: String, exchange_name: String) -> PyResult<()> {
        let instrument_id = InstrumentId::from_str(&instrument_id)
            .map_err(|e| PyValueError::new_err(format!("Invalid instrument ID: {}", e)))?;
        self.inner.configure_routing(instrument_id, exchange_name);
        Ok(())
    }
    
    fn __str__(&self) -> String {
        let stats = self.inner.get_statistics();
        format!("ExecutionEngine(active_orders={}, total_submitted={})",
            self.inner.get_active_orders_count(),
            stats.orders_submitted
        )
    }
}

// ============================================================================
// MODULE REGISTRATION
// ============================================================================

/// Register execution engine types with Python module
pub fn register_execution_types(py: Python, parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let execution_module = PyModule::new_bound(py, "execution")?;
    
    // Order enums
    execution_module.add_class::<PyOrderSide>()?;
    execution_module.add_class::<PyOrderType>()?;
    execution_module.add_class::<PyOrderStatus>()?;
    execution_module.add_class::<PyTimeInForce>()?;
    
    // Core execution types
    execution_module.add_class::<PyOrder>()?;
    execution_module.add_class::<PyFill>()?;
    execution_module.add_class::<PyExecutionStats>()?;
    execution_module.add_class::<PyExecutionEngine>()?;
    
    parent_module.add_submodule(&execution_module)?;
    Ok(())
}
