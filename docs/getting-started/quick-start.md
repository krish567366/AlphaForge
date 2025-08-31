# Quick Start Guide

Get up and running with AlphaForge in minutes.

## System Requirements

### Minimum Requirements

- **Python**: 3.8 or higher
- **Operating System**: Windows 10, macOS 10.15, or Linux (Ubuntu 18.04+)
- **Memory**: 4GB RAM minimum, 8GB recommended
- **CPU**: Modern x64 processor with AVX2 support (recommended)

### Recommended Development Environment

- **Python**: 3.11 or 3.12 (latest stable)
- **IDE**: VS Code with Python extension
- **Memory**: 16GB RAM for optimal performance
- **Storage**: SSD for faster compilation and data access

## Installation

### Option 1: Install from PyPI (Recommended)

```bash
pip install alphaforge
```

!!! note "PyPI Release Status"
    The PyPI package will be available once we reach v1.0.0. Currently, please use the build from source option.

### Option 2: Build from Source

#### Prerequisites

1. **Install Rust** (required for building):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Install Python build tools**:
   ```bash
   pip install maturin
   ```

#### Build Steps

1. **Clone the repository**:
   ```bash
   git clone https://github.com/AlphaForge/alphaforge
   cd alphaforge
   ```

2. **Set up Python virtual environment** (recommended):
   ```bash
   python -m venv .venv
   # On Windows:
   .venv\Scripts\activate
   # On macOS/Linux:
   source .venv/bin/activate
   ```

3. **Build and install**:
   ```bash
   # Development build (faster compilation)
   maturin develop
   
   # Release build (optimized performance)
   maturin develop --release
   ```

4. **Verify installation**:
   ```python
   python -c "import alphaforge_pyo3; print('AlphaForge installed successfully!')"
   ```

## Your First AlphaForge Program

Let's create a simple program to demonstrate AlphaForge capabilities:

### 1. Cache System Demo

```python
from alphaforge_pyo3.cache import GenericCache

# Create a high-performance cache
cache = GenericCache(max_size=10000, default_ttl=60.0)

# Store and retrieve data
cache.put("BTCUSD_price", 45000.0)
cache.put("ETHUSD_price", 3000.0)

# Retrieve data
btc_price = cache.get("BTCUSD_price")
eth_price = cache.get("ETHUSD_price")

print(f"BTC Price: ${btc_price}")
print(f"ETH Price: ${eth_price}")

# Check cache statistics
stats = cache.statistics()
print(f"Cache hits: {stats.hits}")
print(f"Cache misses: {stats.misses}")
print(f"Hit ratio: {stats.hit_ratio:.2%}")
```

### 2. Data Processing Demo

```python
from alphaforge_pyo3.data import DataEngine, DataEngineConfig
from alphaforge_pyo3.model import TradeTick, Price, Quantity
import time

# Configure data engine
config = DataEngineConfig(enable_statistics=True)
engine = DataEngine(config)

# Start the engine
engine.start()

# Create sample tick data
tick = TradeTick(
    instrument_id="BTCUSD",
    price=Price(45000.0),
    quantity=Quantity(0.5),
    aggressor_side="BUY",
    trade_id="12345",
    ts_event=int(time.time() * 1_000_000_000),  # nanoseconds
    ts_init=int(time.time() * 1_000_000_000)
)

# Process the tick
bars = engine.process_trade_tick(tick)
print(f"Generated {len(bars)} bars from tick")

# Get processing statistics
stats = engine.statistics()
print(f"Ticks processed: {stats.ticks_processed}")
print(f"Bars generated: {stats.bars_generated}")
print(f"Processing rate: {stats.processing_rate:.0f} ticks/sec")

# Stop the engine
engine.stop()
```

### 3. Live Execution Demo

```python
from alphaforge_pyo3.execution import ExecutionEngine, Order, OrderType, OrderSide, TimeInForce

# Create execution engine
engine = ExecutionEngine()

# Create different order types
market_order = Order.market("BTCUSD", OrderSide.Buy, 0.1, "strategy_1")
limit_order = Order.limit("ETHUSD", OrderSide.Sell, 0.5, 3000.0, TimeInForce.GoodTillCancel, "strategy_1")
stop_order = Order.stop("BTCUSD", OrderSide.Sell, 0.2, 44000.0, TimeInForce.GoodTillCancel, "strategy_1")

# Submit orders
market_id = engine.submit_order(market_order)
limit_id = engine.submit_order(limit_order)  
stop_id = engine.submit_order(stop_order)

print(f"Market order ID: {market_id}")
print(f"Limit order ID: {limit_id}")
print(f"Stop order ID: {stop_id}")

# Get execution statistics
stats = engine.statistics()
print(f"Orders submitted: {stats.orders_submitted}")
print(f"Orders filled: {stats.orders_filled}")
print(f"Fill ratio: {stats.fill_ratio:.2%}")
print(f"Average execution latency: {stats.avg_execution_latency_ms:.2f}ms")
```

## Performance Validation

Run this benchmark to validate your installation performance:

```python
from alphaforge_pyo3.cache import GenericCache
import time
import random

def benchmark_cache_performance():
    """Benchmark cache performance"""
    cache = GenericCache(max_size=100000, default_ttl=3600.0)
    
    # Warm up
    for i in range(1000):
        cache.put(f"key_{i}", random.random())
    
    # Benchmark PUT operations
    start_time = time.perf_counter()
    num_operations = 100000
    
    for i in range(num_operations):
        cache.put(f"benchmark_key_{i}", random.random())
    
    put_duration = time.perf_counter() - start_time
    put_ops_per_sec = num_operations / put_duration
    
    # Benchmark GET operations
    start_time = time.perf_counter()
    
    for i in range(num_operations):
        cache.get(f"benchmark_key_{i}")
    
    get_duration = time.perf_counter() - start_time
    get_ops_per_sec = num_operations / get_duration
    
    # Results
    print(f"PUT Operations: {put_ops_per_sec:,.0f} ops/sec")
    print(f"GET Operations: {get_ops_per_sec:,.0f} ops/sec")
    print(f"Combined: {(put_ops_per_sec + get_ops_per_sec):,.0f} ops/sec")
    
    # Validate performance targets
    if put_ops_per_sec > 1_000_000:
        print("✅ PUT performance: EXCELLENT")
    elif put_ops_per_sec > 500_000:
        print("✅ PUT performance: GOOD")
    else:
        print("⚠️ PUT performance: BELOW EXPECTED")
        
    if get_ops_per_sec > 2_000_000:
        print("✅ GET performance: EXCELLENT") 
    elif get_ops_per_sec > 1_000_000:
        print("✅ GET performance: GOOD")
    else:
        print("⚠️ GET performance: BELOW EXPECTED")

# Run benchmark
benchmark_cache_performance()
```

Expected output on modern hardware:
```
PUT Operations: 1,470,000 ops/sec
GET Operations: 3,250,000 ops/sec
Combined: 4,720,000 ops/sec
✅ PUT performance: EXCELLENT
✅ GET performance: EXCELLENT
```

## Next Steps

Now that you have AlphaForge installed and verified, explore these topics:

1. **[Configuration](configuration.md)** - Learn about system configuration options
2. **[Architecture Overview](../architecture/overview.md)** - Understand the system design
3. **[Basic Strategy](../examples/basic-strategy.md)** - Build your first trading strategy
4. **[API Reference](../api/python-api.md)** - Explore the complete API

## Troubleshooting

### Common Installation Issues

**Issue**: `maturin` command not found
```bash
# Solution: Install maturin
pip install maturin
```

**Issue**: Rust compiler errors
```bash
# Solution: Update Rust toolchain
rustup update stable
```

**Issue**: Python version compatibility
```bash
# Solution: Use Python 3.8+
python --version
# If needed, install newer Python version
```

**Issue**: Performance below expectations
```bash
# Solution: Ensure release build
maturin develop --release
# Enable CPU optimizations
export RUSTFLAGS="-C target-cpu=native"
maturin develop --release
```

### Getting Help

- **Documentation**: Browse the full [documentation](../index.md)
- **GitHub Issues**: Report bugs or request features on [GitHub](https://github.com/AlphaForge/alphaforge/issues)
- **Discussions**: Join conversations in [GitHub Discussions](https://github.com/AlphaForge/alphaforge/discussions)

---

**Ready to start building? Let's create your first trading strategy!**
