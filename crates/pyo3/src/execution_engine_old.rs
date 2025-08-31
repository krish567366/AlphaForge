use pyo3::prelude::*;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use std::sync::Arc;
use std::collections::HashMap;

// Import the Rust execution engine
use alphaforge_core::execution_engine::{
    ExecutionEngine, Order, OrderSide, OrderType, OrderStatus, TimeInForce, Fill, ExecutionStats,
    ExecutionError, OrderEvent,
};
use alphaforge_core::identifiers::{OrderId, InstrumentId, StrategyId, VenueOrderId};
use alphaforge_core::message_bus::MessageBus;

// ============================================================================
// PYTHON WRAPPER FOR ORDER SIDE
// ============================================================================

#[pyclass(name = "OrderSide")]
#[derive(Clone, Debug)]
pub struct PyOrderSide {
    inner: OrderSide,
}

#[pymethods]
impl PyOrderSide {
    #[classattr]
    const Buy: PyOrderSide = PyOrderSide { inner: OrderSide::Buy };
    
    #[classattr]
    const Sell: PyOrderSide = PyOrderSide { inner: OrderSide::Sell };
    
    fn __str__(&self) -> String {
        format!("{:?}", self.inner)
    }
    
    fn __repr__(&self) -> String {
        format!("OrderSide.{:?}", self.inner)
    }
}

// ============================================================================
// PYTHON WRAPPER FOR ORDER TYPE
// ============================================================================

#[pyclass(name = "OrderType")]
#[derive(Clone, Debug)]
pub struct PyOrderType {
    inner: OrderType,
}

#[pymethods]
impl PyOrderType {
    #[classattr]
    const Market: PyOrderType = PyOrderType { inner: OrderType::Market };
    
    #[classattr]
    const Limit: PyOrderType = PyOrderType { inner: OrderType::Limit };
    
    #[classattr]
    const Stop: PyOrderType = PyOrderType { inner: OrderType::Stop };
    
    #[classattr]
    const StopLimit: PyOrderType = PyOrderType { inner: OrderType::StopLimit };
    
    fn __str__(&self) -> String {
        format!("{:?}", self.inner)
    }
    
    fn __repr__(&self) -> String {
        format!("OrderType.{:?}", self.inner)
    }
}

// ============================================================================
// PYTHON WRAPPER FOR ORDER STATUS
// ============================================================================

#[pyclass(name = "OrderStatus")]
#[derive(Clone, Debug)]
pub struct PyOrderStatus {
    inner: OrderStatus,
}

#[pymethods]
impl PyOrderStatus {
    #[classattr]
    const Initialized: PyOrderStatus = PyOrderStatus { inner: OrderStatus::Initialized };
    
    #[classattr]
    const Submitted: PyOrderStatus = PyOrderStatus { inner: OrderStatus::Submitted };
    
    #[classattr]
    const Accepted: PyOrderStatus = PyOrderStatus { inner: OrderStatus::Accepted };
    
    #[classattr]
    const PartiallyFilled: PyOrderStatus = PyOrderStatus { inner: OrderStatus::PartiallyFilled };
    
    #[classattr]
    const Filled: PyOrderStatus = PyOrderStatus { inner: OrderStatus::Filled };
    
    #[classattr]
    const Cancelled: PyOrderStatus = PyOrderStatus { inner: OrderStatus::Cancelled };
    
    #[classattr]
    const Rejected: PyOrderStatus = PyOrderStatus { inner: OrderStatus::Rejected };
    
    #[classattr]
    const Expired: PyOrderStatus = PyOrderStatus { inner: OrderStatus::Expired };
    
    fn __str__(&self) -> String {
        format!("{:?}", self.inner)
    }
    
    fn __repr__(&self) -> String {
        format!("OrderStatus.{:?}", self.inner)
    }
}

// ============================================================================
// PYTHON WRAPPER FOR TIME IN FORCE
// ============================================================================

#[pyclass(name = "TimeInForce")]
#[derive(Clone, Debug)]
pub struct PyTimeInForce {
    inner: TimeInForce,
}

#[pymethods]
impl PyTimeInForce {
    #[classattr]
    const GTC: PyTimeInForce = PyTimeInForce { inner: TimeInForce::GTC };
    
    #[classattr]
    const IOC: PyTimeInForce = PyTimeInForce { inner: TimeInForce::IOC };
    
    #[classattr]
    const FOK: PyTimeInForce = PyTimeInForce { inner: TimeInForce::FOK };
    
    #[classattr]
    const GTD: PyTimeInForce = PyTimeInForce { inner: TimeInForce::GTD };
    
    #[classattr]
    const DAY: PyTimeInForce = PyTimeInForce { inner: TimeInForce::DAY };
    
    fn __str__(&self) -> String {
        format!("{:?}", self.inner)
    }
    
    fn __repr__(&self) -> String {
        format!("TimeInForce.{:?}", self.inner)
    }
}

// ============================================================================
// PYTHON WRAPPER FOR ORDER
// ============================================================================

#[pyclass(name = "Order")]
#[derive(Clone, Debug)]
pub struct PyOrder {
    inner: Order,
}

#[pymethods]
impl PyOrder {
    #[new]
    #[pyo3(signature = (
        strategy_id,
        instrument_id,
        side,
        order_type,
        quantity,
        price = None,
        stop_price = None,
        time_in_force = None
    ))]
    fn new(
        strategy_id: crate::strategy_engine::PyStrategyId,
        instrument_id: String,
        side: PyOrderSide,
        order_type: PyOrderType,
        quantity: f64,
        price: Option<f64>,
        stop_price: Option<f64>,
        time_in_force: Option<PyTimeInForce>,
    ) -> PyResult<Self> {
        use std::str::FromStr;
        
        let instrument_id = InstrumentId::from_str(&instrument_id)
            .map_err(|e| PyValueError::new_err(format!("Invalid instrument ID: {}", e)))?;
        
        let tif = time_in_force.map(|t| t.inner).unwrap_or(TimeInForce::GTC);
        
        let now = alphaforge_core::time::unix_nanos_now();
        let order_id = OrderId::new();
        
        let order = Order {
            order_id,
            strategy_id: strategy_id.inner,
            instrument_id,
            side: side.inner,
            order_type: order_type.inner,
            quantity,
            price,
            stop_price,
            time_in_force: tif,
            status: OrderStatus::Initialized,
            venue_order_id: None,
            filled_quantity: 0.0,
            avg_fill_price: None,
            created_time: now,
            updated_time: now,
            commission: 0.0,
            tags: HashMap::new(),
        };
        
        Ok(Self { inner: order })
    }
    
    #[staticmethod]
    fn market(
        strategy_id: crate::strategy_engine::PyStrategyId,
        instrument_id: String,
        side: PyOrderSide,
        quantity: f64,
    ) -> PyResult<Self> {
        use std::str::FromStr;
        
        let instrument_id = InstrumentId::from_str(&instrument_id)
            .map_err(|e| PyValueError::new_err(format!("Invalid instrument ID: {}", e)))?;
        
        let order = Order::market(strategy_id.inner, instrument_id, side.inner, quantity);
        Ok(Self { inner: order })
    }
    
    #[staticmethod]
    fn limit(
        strategy_id: crate::strategy_engine::PyStrategyId,
        instrument_id: String,
        side: PyOrderSide,
        quantity: f64,
        price: f64,
    ) -> PyResult<Self> {
        use std::str::FromStr;
        
        let instrument_id = InstrumentId::from_str(&instrument_id)
            .map_err(|e| PyValueError::new_err(format!("Invalid instrument ID: {}", e)))?;
        
        let order = Order::limit(strategy_id.inner, instrument_id, side.inner, quantity, price);
        Ok(Self { inner: order })
    }
    
    #[getter]
    fn order_id(&self) -> String {
        self.inner.order_id.to_string()
    }
    
    #[getter]
    fn strategy_id(&self) -> crate::strategy_engine::PyStrategyId {
        crate::strategy_engine::PyStrategyId { inner: self.inner.strategy_id }
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
    
    #[getter]
    fn commission(&self) -> f64 {
        self.inner.commission
    }
    
    fn is_active(&self) -> bool {
        self.inner.is_active()
    }
    
    fn is_complete(&self) -> bool {
        self.inner.is_complete()
    }
    
    fn is_filled(&self) -> bool {
        self.inner.is_filled()
    }
    
    fn remaining_quantity(&self) -> f64 {
        self.inner.remaining_quantity()
    }
    
    fn __str__(&self) -> String {
        format!("Order(id={}, instrument={}, side={:?}, quantity={}, status={:?})", 
                self.inner.order_id, 
                self.inner.instrument_id,
                self.inner.side,
                self.inner.quantity,
                self.inner.status)
    }
    
    fn __repr__(&self) -> String {
        self.__str__()
    }
}

// ============================================================================
// PYTHON WRAPPER FOR FILL
// ============================================================================

#[pyclass(name = "Fill")]
#[derive(Clone, Debug)]
pub struct PyFill {
    inner: Fill,
}

#[pymethods]
impl PyFill {
    #[new]
    fn new(
        order_id: String,
        fill_id: String,
        price: f64,
        quantity: f64,
        commission: f64,
        commission_currency: String,
    ) -> PyResult<Self> {
        let order_id = OrderId::from_u64(
            order_id.parse::<u64>()
                .map_err(|_| PyValueError::new_err("Invalid order ID format"))?
        );
        
        let fill = Fill {
            order_id,
            fill_id,
            price,
            quantity,
            timestamp: alphaforge_core::time::unix_nanos_now(),
            commission,
            commission_currency,
        };
        
        Ok(Self { inner: fill })
    }
    
    #[getter]
    fn order_id(&self) -> String {
        self.inner.order_id.to_string()
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
        format!("Fill(order_id={}, price={}, quantity={}, commission={})",
                self.inner.order_id,
                self.inner.price,
                self.inner.quantity,
                self.inner.commission)
    }
    
    fn __repr__(&self) -> String {
        self.__str__()
    }
}

// ============================================================================
// PYTHON WRAPPER FOR EXECUTION STATISTICS
// ============================================================================

#[pyclass(name = "ExecutionStats")]
#[derive(Clone, Debug)]
pub struct PyExecutionStats {
    inner: ExecutionStats,
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
    
    fn get_fill_rate(&self) -> f64 {
        if self.inner.orders_submitted == 0 {
            0.0
        } else {
            self.inner.orders_filled as f64 / self.inner.orders_submitted as f64
        }
    }
    
    fn __str__(&self) -> String {
        format!("ExecutionStats(submitted={}, filled={}, cancelled={}, fill_rate={:.2%})",
                self.inner.orders_submitted,
                self.inner.orders_filled,
                self.inner.orders_cancelled,
                self.get_fill_rate())
    }
    
    fn __repr__(&self) -> String {
        self.__str__()
    }
}

// ============================================================================
// PYTHON WRAPPER FOR EXECUTION ENGINE
// ============================================================================

#[pyclass(name = "ExecutionEngine")]
pub struct PyExecutionEngine {
    inner: Arc<ExecutionEngine>,
}

#[pymethods]
impl PyExecutionEngine {
    #[new]
    fn new() -> Self {
        let message_bus = Arc::new(MessageBus::new());
        let engine = ExecutionEngine::new(message_bus);
        
        Self {
            inner: Arc::new(engine),
        }
    }
    
    #[pyo3(signature = (order))]
    fn submit_order(&self, order: PyOrder, py: Python) -> PyResult<Py<PyAny>> {
        let engine = self.inner.clone();
        let rust_order = order.inner.clone();
        
        pyo3_asyncio::tokio::future_into_py(py, async move {
            match engine.submit_order(rust_order).await {
                Ok(order_id) => Ok(order_id.to_string()),
                Err(e) => Err(PyRuntimeError::new_err(format!("Failed to submit order: {}", e))),
            }
        })
    }
    
    #[pyo3(signature = (order_id))]
    fn cancel_order(&self, order_id: String, py: Python) -> PyResult<Py<PyAny>> {
        let engine = self.inner.clone();
        let order_id = OrderId::from_u64(
            order_id.parse::<u64>()
                .map_err(|_| PyValueError::new_err("Invalid order ID format"))?
        );
        
        pyo3_asyncio::tokio::future_into_py(py, async move {
            match engine.cancel_order(order_id).await {
                Ok(_) => Ok(Python::with_gil(|py| py.None())),
                Err(e) => Err(PyRuntimeError::new_err(format!("Failed to cancel order: {}", e))),
            }
        })
    }
    
    fn handle_fill(&self, fill: PyFill) -> PyResult<()> {
        self.inner.handle_fill(fill.inner)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to handle fill: {}", e)))
    }
    
    fn get_statistics(&self) -> PyExecutionStats {
        PyExecutionStats {
            inner: self.inner.get_statistics(),
        }
    }
    
    fn get_strategy_orders(&self, strategy_id: crate::strategy_engine::PyStrategyId) -> Vec<PyOrder> {
        self.inner.get_strategy_orders(strategy_id.inner)
            .into_iter()
            .map(|order| PyOrder { inner: order })
            .collect()
    }
    
    fn get_active_orders_count(&self) -> usize {
        self.inner.get_active_orders_count()
    }
    
    fn configure_routing(&self, instrument_id: String, exchange_name: String) -> PyResult<()> {
        use std::str::FromStr;
        
        let instrument_id = InstrumentId::from_str(&instrument_id)
            .map_err(|e| PyValueError::new_err(format!("Invalid instrument ID: {}", e)))?;
        
        self.inner.configure_routing(instrument_id, exchange_name);
        Ok(())
    }
    
    fn __str__(&self) -> String {
        format!("ExecutionEngine(active_orders={})", self.get_active_orders_count())
    }
    
    fn __repr__(&self) -> String {
        self.__str__()
    }
}

/// Register the execution engine module with Python
pub fn register_execution_module(py: Python, parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let execution_module = PyModule::new_bound(py, "execution")?;
    
    // Add enums
    execution_module.add_class::<PyOrderSide>()?;
    execution_module.add_class::<PyOrderType>()?;
    execution_module.add_class::<PyOrderStatus>()?;
    execution_module.add_class::<PyTimeInForce>()?;
    
    // Add data structures
    execution_module.add_class::<PyOrder>()?;
    execution_module.add_class::<PyFill>()?;
    execution_module.add_class::<PyExecutionStats>()?;
    
    // Add engine
    execution_module.add_class::<PyExecutionEngine>()?;
    
    // Add to parent module
    parent_module.add_submodule(&execution_module)?;
    
    // Register in sys.modules
    let sys = py.import_bound("sys")?;
    let modules = sys.getattr("modules")?;
    modules.set_item("alphaforge_pyo3.execution", &execution_module)?;
    
    Ok(())
}
