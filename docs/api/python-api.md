# Python API Reference

Complete reference for AlphaForge's Python API.

## Overview

AlphaForge exposes its high-performance Rust core through carefully designed Python bindings. All components maintain type safety and performance while providing a familiar Python interface.

## Module Structure

```
alphaforge_pyo3/
├── cache          # High-performance caching system
├── data           # Market data processing engine  
├── execution      # Order execution and management
├── model          # Core data types and structures
└── time           # Time management utilities
```

## Core Modules

### Cache Module

High-performance in-memory caching system.

#### `GenericCache`

Ultra-fast cache with LRU eviction and TTL support.

```python
from alphaforge_pyo3.cache import GenericCache

# Create cache instance
cache = GenericCache(max_size=10000, default_ttl=60.0)
```

**Constructor Parameters:**
- `max_size: int` - Maximum number of entries (default: 10000)
- `default_ttl: float` - Default time-to-live in seconds (default: 3600.0)

**Methods:**

##### `put(key: str, value: Any) -> None`
Store a value in the cache.

```python
cache.put("BTCUSD_price", 45000.0)
cache.put("user_config", {"theme": "dark", "timeout": 30})
```

##### `get(key: str) -> Optional[Any]`
Retrieve a value from the cache.

```python
price = cache.get("BTCUSD_price")  # Returns 45000.0 or None if expired/missing
config = cache.get("user_config")  # Returns dict or None
```

##### `delete(key: str) -> bool`
Remove a key from the cache.

```python
removed = cache.delete("old_data")  # Returns True if key existed
```

##### `clear() -> None`
Remove all entries from the cache.

```python
cache.clear()
```

##### `contains(key: str) -> bool`
Check if a key exists in the cache.

```python
exists = cache.contains("BTCUSD_price")  # Returns True/False
```

##### `size() -> int`
Get the current number of entries.

```python
entry_count = cache.size()
```

##### `statistics() -> GenericCacheStatistics`
Get cache performance statistics.

```python
stats = cache.statistics()
print(f"Hit ratio: {stats.hit_ratio:.2%}")
print(f"Total operations: {stats.total_operations}")
```

**Performance Characteristics:**
- **PUT operations**: 1.47M ops/sec
- **GET operations**: 3.25M ops/sec  
- **Average latency**: 0.3μs (GET), 0.7μs (PUT)
- **Thread-safe**: Full concurrent access support

---

### Data Module

Market data processing and aggregation engine.

#### `DataEngine`

High-performance market data processing engine.

```python
from alphaforge_pyo3.data import DataEngine, DataEngineConfig

# Configure data engine
config = DataEngineConfig(enable_statistics=True)
engine = DataEngine(config)
```

**Constructor:**
- `config: DataEngineConfig` - Engine configuration

##### `start() -> None`
Start the data engine.

```python
engine.start()
```

##### `stop() -> None`
Stop the data engine.

```python
engine.stop()  
```

##### `process_trade_tick(tick: TradeTick) -> List[Bar]`
Process a trade tick and return any generated bars.

```python
from alphaforge_pyo3.model import TradeTick, Price, Quantity
import time

tick = TradeTick(
    instrument_id="BTCUSD",
    price=Price(45000.0),
    quantity=Quantity(0.5),
    aggressor_side="BUY", 
    trade_id="12345",
    ts_event=int(time.time() * 1_000_000_000),
    ts_init=int(time.time() * 1_000_000_000)
)

bars = engine.process_trade_tick(tick)
print(f"Generated {len(bars)} bars")
```

##### `statistics() -> DataEngineStatistics`
Get processing statistics.

```python
stats = engine.statistics()
print(f"Ticks processed: {stats.ticks_processed:,}")
print(f"Processing rate: {stats.processing_rate:,.0f} ticks/sec")
print(f"Bars generated: {stats.bars_generated:,}")
```

#### `DataEngineConfig`

Configuration for the data engine.

```python
config = DataEngineConfig(
    enable_statistics=True,
    bar_types=["1min", "5min", "1hour"],
    max_bars_per_instrument=1000
)
```

**Parameters:**
- `enable_statistics: bool` - Enable performance statistics (default: True)
- `bar_types: List[str]` - Bar types to generate (default: ["1min"])
- `max_bars_per_instrument: int` - Maximum bars to keep per instrument (default: 1000)

**Performance Characteristics:**
- **Tick processing**: 146K+ ticks/sec
- **Bar generation**: Sub-7μs latency
- **Memory efficient**: Bounded memory usage
- **Multiple aggregation types**: Time, tick, volume, dollar bars

---

### Execution Module

Order execution and management system.

#### `ExecutionEngine`

Real-time order execution engine.

```python
from alphaforge_pyo3.execution import ExecutionEngine

engine = ExecutionEngine()
```

##### `submit_order(order: Order) -> str`
Submit an order for execution.

```python
from alphaforge_pyo3.execution import Order, OrderSide, OrderType, TimeInForce

# Market order
market_order = Order.market("BTCUSD", OrderSide.Buy, 0.1, "strategy_1")
order_id = engine.submit_order(market_order)

# Limit order  
limit_order = Order.limit("ETHUSD", OrderSide.Sell, 0.5, 3000.0, 
                         TimeInForce.GoodTillCancel, "strategy_1")
order_id = engine.submit_order(limit_order)
```

##### `cancel_order(order_id: str) -> bool`
Cancel an existing order.

```python
success = engine.cancel_order("order_123")
```

##### `get_order(order_id: str) -> Optional[Order]`
Retrieve order details.

```python
order = engine.get_order("order_123")
if order:
    print(f"Order status: {order.status}")
```

##### `get_active_orders(strategy_id: str) -> List[Order]`
Get all active orders for a strategy.

```python
active_orders = engine.get_active_orders("strategy_1")
print(f"Active orders: {len(active_orders)}")
```

##### `statistics() -> ExecutionStatistics`
Get execution statistics.

```python
stats = engine.statistics()
print(f"Orders submitted: {stats.orders_submitted}")
print(f"Fill ratio: {stats.fill_ratio:.2%}")
print(f"Avg execution latency: {stats.avg_execution_latency_ms:.2f}ms")
```

#### `Order`

Order representation with factory methods.

**Factory Methods:**

##### `Order.market(symbol: str, side: OrderSide, quantity: float, strategy_id: str) -> Order`
Create a market order.

```python
order = Order.market("BTCUSD", OrderSide.Buy, 0.1, "my_strategy")
```

##### `Order.limit(symbol: str, side: OrderSide, quantity: float, price: float, time_in_force: TimeInForce, strategy_id: str) -> Order`
Create a limit order.

```python
order = Order.limit("BTCUSD", OrderSide.Sell, 0.1, 45000.0, 
                   TimeInForce.GoodTillCancel, "my_strategy")
```

##### `Order.stop(symbol: str, side: OrderSide, quantity: float, stop_price: float, time_in_force: TimeInForce, strategy_id: str) -> Order`
Create a stop order.

```python
order = Order.stop("BTCUSD", OrderSide.Sell, 0.1, 44000.0,
                  TimeInForce.GoodTillCancel, "my_strategy")
```

##### `Order.stop_limit(symbol: str, side: OrderSide, quantity: float, stop_price: float, limit_price: float, time_in_force: TimeInForce, strategy_id: str) -> Order`
Create a stop-limit order.

```python
order = Order.stop_limit("BTCUSD", OrderSide.Sell, 0.1, 44000.0, 43900.0,
                        TimeInForce.GoodTillCancel, "my_strategy")
```

**Properties:**
- `order_id: str` - Unique order identifier
- `symbol: str` - Trading symbol
- `side: OrderSide` - Buy or Sell
- `quantity: float` - Order quantity
- `price: Optional[float]` - Limit price (if applicable)
- `stop_price: Optional[float]` - Stop price (if applicable)
- `order_type: OrderType` - Market, Limit, Stop, StopLimit
- `time_in_force: TimeInForce` - Order time in force
- `strategy_id: str` - Strategy identifier
- `status: OrderStatus` - Current order status
- `created_at: int` - Creation timestamp (nanoseconds)

#### Enumerations

##### `OrderSide`
```python
from alphaforge_pyo3.execution import OrderSide

OrderSide.Buy    # Buy side
OrderSide.Sell   # Sell side
```

##### `OrderType`  
```python
from alphaforge_pyo3.execution import OrderType

OrderType.Market      # Market order
OrderType.Limit       # Limit order
OrderType.Stop        # Stop order
OrderType.StopLimit   # Stop-limit order
```

##### `OrderStatus`
```python
from alphaforge_pyo3.execution import OrderStatus

OrderStatus.Initialized  # Order created
OrderStatus.Submitted    # Order submitted to exchange
OrderStatus.PartiallyFilled  # Partially executed
OrderStatus.Filled       # Fully executed  
OrderStatus.Cancelled    # Order cancelled
OrderStatus.Rejected     # Order rejected
```

##### `TimeInForce`
```python
from alphaforge_pyo3.execution import TimeInForce

TimeInForce.Day                # Good for trading day
TimeInForce.GoodTillCancel     # Good until cancelled
TimeInForce.ImmediateOrCancel  # Execute immediately or cancel
TimeInForce.FillOrKill         # Fill completely or cancel
```

**Performance Characteristics:**
- **Order submission**: Sub-millisecond latency
- **Throughput**: 15K+ orders/sec
- **Memory efficient**: Pooled order objects
- **Thread-safe**: Concurrent order management

---

### Model Module

Core data types and structures.

#### `TradeTick`

Represents a single trade execution.

```python
from alphaforge_pyo3.model import TradeTick, Price, Quantity

tick = TradeTick(
    instrument_id="BTCUSD",
    price=Price(45000.0),
    quantity=Quantity(0.5),
    aggressor_side="BUY",
    trade_id="trade_12345", 
    ts_event=1640995200000000000,  # Event timestamp (nanoseconds)
    ts_init=1640995200000000000    # Init timestamp (nanoseconds)
)
```

**Properties:**
- `instrument_id: str` - Trading instrument identifier
- `price: Price` - Trade price
- `quantity: Quantity` - Trade quantity
- `aggressor_side: str` - "BUY" or "SELL"
- `trade_id: str` - Unique trade identifier
- `ts_event: int` - Event timestamp (nanoseconds since Unix epoch)
- `ts_init: int` - Initialization timestamp (nanoseconds since Unix epoch)

#### `Price`

High-precision price representation.

```python
from alphaforge_pyo3.model import Price

price = Price(45000.0)
print(f"Price: ${price}")  # Price: $45000.00
print(f"Raw value: {float(price)}")  # Raw value: 45000.0
```

#### `Quantity`

High-precision quantity representation.

```python
from alphaforge_pyo3.model import Quantity

qty = Quantity(0.5)
print(f"Quantity: {qty}")  # Quantity: 0.5
print(f"Raw value: {float(qty)}")  # Raw value: 0.5
```

---

### Time Module

Time management and utilities.

#### Functions

##### `unix_nanos_now() -> int`
Get current Unix timestamp in nanoseconds.

```python
from alphaforge_pyo3.time import unix_nanos_now

timestamp = unix_nanos_now()
print(f"Current time: {timestamp}")  # Current time: 1640995200123456789
```

##### `nanos_to_millis(nanos: int) -> int`
Convert nanoseconds to milliseconds.

```python
from alphaforge_pyo3.time import nanos_to_millis

millis = nanos_to_millis(1640995200123456789)
print(f"Milliseconds: {millis}")  # Milliseconds: 1640995200123
```

##### `nanos_to_secs(nanos: int) -> int`
Convert nanoseconds to seconds.

```python
from alphaforge_pyo3.time import nanos_to_secs

secs = nanos_to_secs(1640995200123456789)
print(f"Seconds: {secs}")  # Seconds: 1640995200
```

---

## Statistics and Monitoring

All major components provide comprehensive statistics for monitoring and optimization.

### Cache Statistics

```python
stats = cache.statistics()

# Access statistics
print(f"Total operations: {stats.total_operations}")
print(f"Cache hits: {stats.hits}")  
print(f"Cache misses: {stats.misses}")
print(f"Hit ratio: {stats.hit_ratio:.2%}")
print(f"Evictions: {stats.evictions}")
print(f"Memory usage: {stats.estimated_memory_bytes:,} bytes")
```

### Data Engine Statistics

```python
stats = data_engine.statistics()

print(f"Ticks processed: {stats.ticks_processed:,}")
print(f"Bars generated: {stats.bars_generated:,}")
print(f"Processing rate: {stats.processing_rate:,.0f} ticks/sec")
print(f"Average processing latency: {stats.avg_processing_latency_ns} ns")
print(f"Memory usage: {stats.memory_usage_bytes:,} bytes")
```

### Execution Engine Statistics

```python
stats = execution_engine.statistics()

print(f"Orders submitted: {stats.orders_submitted}")
print(f"Orders filled: {stats.orders_filled}")
print(f"Orders cancelled: {stats.orders_cancelled}")
print(f"Fill ratio: {stats.fill_ratio:.2%}")
print(f"Average execution latency: {stats.avg_execution_latency_ms:.2f} ms")
print(f"Total volume: {stats.total_volume:,.2f}")
print(f"Total commission: {stats.total_commission:,.2f}")
```

## Error Handling

All AlphaForge operations use proper Python exception handling:

```python
try:
    order_id = engine.submit_order(order)
    print(f"Order submitted: {order_id}")
except ValueError as e:
    print(f"Invalid order parameters: {e}")
except RuntimeError as e:
    print(f"Execution engine error: {e}")
except Exception as e:
    print(f"Unexpected error: {e}")
```

## Performance Best Practices

### 1. Reuse Objects

```python
# Good: Reuse engine instances
engine = ExecutionEngine()
for order in orders:
    engine.submit_order(order)

# Avoid: Creating new engines repeatedly
for order in orders:
    engine = ExecutionEngine()  # Expensive!
    engine.submit_order(order)
```

### 2. Batch Operations  

```python
# Good: Batch cache operations
cache = GenericCache()
for key, value in data.items():
    cache.put(key, value)

# Better: Use cache warming if available
cache.warm_cache(data)
```

### 3. Monitor Statistics

```python
# Monitor performance regularly
def check_performance():
    stats = engine.statistics()
    if stats.avg_execution_latency_ms > 10.0:
        print("⚠️  High execution latency detected!")
    
    cache_stats = cache.statistics()
    if cache_stats.hit_ratio < 0.8:
        print("⚠️  Low cache hit ratio!")
```

### 4. Proper Resource Management

```python
# Good: Explicit cleanup
engine = DataEngine(config)
try:
    engine.start()
    # Use engine...
finally:
    engine.stop()  # Ensure cleanup

# Better: Context manager (if available)
with DataEngine(config) as engine:
    # Use engine...
    pass  # Automatic cleanup
```

---

This API reference provides the complete interface for building high-performance algorithmic trading systems with AlphaForge. All components are designed for maximum performance while maintaining Python's ease of use.
