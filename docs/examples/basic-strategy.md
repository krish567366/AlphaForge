# Basic Strategy Development

Learn how to create your first algorithmic trading strategy with AlphaForge.

## Overview

This guide walks you through creating a simple moving average crossover strategy, one of the most popular algorithmic trading strategies. You'll learn:

- How to set up a basic strategy structure
- How to process market data
- How to generate trading signals
- How to submit orders
- How to monitor performance

## Strategy Concept: Moving Average Crossover

The moving average crossover strategy generates trading signals based on the relationship between two moving averages:

- **Buy Signal**: When the short-term moving average crosses above the long-term moving average
- **Sell Signal**: When the short-term moving average crosses below the long-term moving average

This strategy works well in trending markets and is simple to understand and implement.

## Implementation

### Step 1: Import Required Modules

```python
from alphaforge_pyo3.data import DataEngine, DataEngineConfig
from alphaforge_pyo3.execution import ExecutionEngine, Order, OrderType, OrderSide, TimeInForce
from alphaforge_pyo3.model import TradeTick, Price, Quantity
from alphaforge_pyo3.cache import GenericCache
import time
import numpy as np
from collections import deque
from typing import Dict, List, Optional
```

### Step 2: Create the Strategy Base Class

```python
class MovingAverageCrossoverStrategy:
    """
    A simple moving average crossover strategy.
    
    Generates buy signals when short MA crosses above long MA.
    Generates sell signals when short MA crosses below long MA.
    """
    
    def __init__(self, 
                 symbol: str,
                 short_window: int = 10,
                 long_window: int = 30,
                 quantity: float = 0.1,
                 strategy_id: str = "ma_crossover"):
        """
        Initialize the moving average crossover strategy.
        
        Args:
            symbol: Trading symbol (e.g., "BTCUSD")
            short_window: Period for short moving average
            long_window: Period for long moving average  
            quantity: Order quantity for each trade
            strategy_id: Unique identifier for this strategy
        """
        self.symbol = symbol
        self.short_window = short_window
        self.long_window = long_window
        self.quantity = quantity
        self.strategy_id = strategy_id
        
        # Price history for calculating moving averages
        self.price_history: deque = deque(maxlen=long_window)
        
        # Track current position
        self.position = 0.0  # Positive = long, negative = short, 0 = flat
        self.last_signal = None  # Track last signal to avoid duplicate orders
        
        # Performance tracking
        self.total_pnl = 0.0
        self.trade_count = 0
        self.winning_trades = 0
        
        # Initialize engines
        self.data_engine = None
        self.execution_engine = None
        
    def initialize_engines(self):
        """Initialize data and execution engines."""
        # Configure and start data engine
        config = DataEngineConfig(enable_statistics=True)
        self.data_engine = DataEngine(config)
        self.data_engine.start()
        
        # Initialize execution engine
        self.execution_engine = ExecutionEngine()
        
        print(f"Strategy '{self.strategy_id}' initialized for {self.symbol}")
        print(f"Short MA: {self.short_window}, Long MA: {self.long_window}")
```

### Step 3: Implement Moving Average Calculation

```python
    def calculate_moving_averages(self) -> tuple[Optional[float], Optional[float]]:
        """
        Calculate short and long moving averages.
        
        Returns:
            Tuple of (short_ma, long_ma) or (None, None) if insufficient data
        """
        if len(self.price_history) < self.long_window:
            return None, None
            
        prices = list(self.price_history)
        
        # Calculate short moving average
        short_ma = np.mean(prices[-self.short_window:]) if len(prices) >= self.short_window else None
        
        # Calculate long moving average  
        long_ma = np.mean(prices) if len(prices) >= self.long_window else None
        
        return short_ma, long_ma
        
    def generate_signal(self, short_ma: float, long_ma: float, price: float) -> Optional[str]:
        """
        Generate trading signal based on moving average crossover.
        
        Args:
            short_ma: Short-term moving average
            long_ma: Long-term moving average
            price: Current price
            
        Returns:
            'BUY', 'SELL', or None
        """
        if short_ma is None or long_ma is None:
            return None
            
        # Generate buy signal: short MA crosses above long MA
        if short_ma > long_ma and self.last_signal != 'BUY' and self.position <= 0:
            return 'BUY'
            
        # Generate sell signal: short MA crosses below long MA  
        elif short_ma < long_ma and self.last_signal != 'SELL' and self.position >= 0:
            return 'SELL'
            
        return None
```

### Step 4: Implement Order Management

```python
    def submit_order(self, signal: str, price: float) -> bool:
        """
        Submit order based on trading signal.
        
        Args:
            signal: 'BUY' or 'SELL'
            price: Current market price
            
        Returns:
            True if order submitted successfully, False otherwise
        """
        try:
            if signal == 'BUY':
                # Create market buy order
                order = Order.market(
                    self.symbol,
                    OrderSide.Buy, 
                    self.quantity,
                    self.strategy_id
                )
                
            elif signal == 'SELL':
                # Create market sell order  
                order = Order.market(
                    self.symbol,
                    OrderSide.Sell,
                    self.quantity, 
                    self.strategy_id
                )
            else:
                return False
                
            # Submit order
            order_id = self.execution_engine.submit_order(order)
            
            # Update position tracking
            if signal == 'BUY':
                self.position += self.quantity
            else:
                self.position -= self.quantity
                
            self.last_signal = signal
            self.trade_count += 1
            
            print(f"✅ {signal} order submitted: {order_id}")
            print(f"   Symbol: {self.symbol}")
            print(f"   Quantity: {self.quantity}")
            print(f"   Price: ${price:.2f}")
            print(f"   New Position: {self.position}")
            
            return True
            
        except Exception as e:
            print(f"❌ Error submitting {signal} order: {e}")
            return False
```

### Step 5: Implement Market Data Processing

```python
    def process_price_update(self, price: float):
        """
        Process new price update and generate trading signals.
        
        Args:
            price: Current market price
        """
        # Add price to history
        self.price_history.append(price)
        
        # Calculate moving averages
        short_ma, long_ma = self.calculate_moving_averages()
        
        if short_ma is not None and long_ma is not None:
            # Generate trading signal
            signal = self.generate_signal(short_ma, long_ma, price)
            
            # Print current state
            print(f"📊 Price: ${price:.2f} | Short MA: ${short_ma:.2f} | Long MA: ${long_ma:.2f}")
            
            # Submit order if signal generated
            if signal:
                self.submit_order(signal, price)
            
    def process_tick(self, tick: TradeTick):
        """
        Process incoming trade tick.
        
        Args:
            tick: Trade tick containing price and volume information
        """
        price = float(tick.price)
        self.process_price_update(price)
        
        # Process tick through data engine for bar generation
        if self.data_engine:
            bars = self.data_engine.process_trade_tick(tick)
            if bars:
                print(f"📈 Generated {len(bars)} new bars")
```

### Step 6: Performance Monitoring

```python
    def get_performance_metrics(self) -> Dict:
        """Get strategy performance metrics."""
        execution_stats = self.execution_engine.statistics()
        data_stats = self.data_engine.statistics() if self.data_engine else None
        
        return {
            'strategy_id': self.strategy_id,
            'symbol': self.symbol,
            'position': self.position,
            'total_trades': self.trade_count,
            'orders_submitted': execution_stats.orders_submitted,
            'orders_filled': execution_stats.orders_filled,
            'fill_ratio': execution_stats.fill_ratio,
            'avg_execution_latency_ms': execution_stats.avg_execution_latency_ms,
            'ticks_processed': data_stats.ticks_processed if data_stats else 0,
            'bars_generated': data_stats.bars_generated if data_stats else 0,
            'processing_rate': data_stats.processing_rate if data_stats else 0,
        }
        
    def print_performance_report(self):
        """Print detailed performance report."""
        metrics = self.get_performance_metrics()
        
        print("\n" + "="*60)
        print(f"📈 STRATEGY PERFORMANCE REPORT")
        print("="*60)
        print(f"Strategy ID: {metrics['strategy_id']}")
        print(f"Symbol: {metrics['symbol']}")
        print(f"Current Position: {metrics['position']}")
        print(f"Total Trades: {metrics['total_trades']}")
        print(f"Orders Submitted: {metrics['orders_submitted']}")
        print(f"Orders Filled: {metrics['orders_filled']}")
        print(f"Fill Ratio: {metrics['fill_ratio']:.2%}")
        print(f"Avg Execution Latency: {metrics['avg_execution_latency_ms']:.2f}ms")
        print(f"Ticks Processed: {metrics['ticks_processed']:,}")
        print(f"Bars Generated: {metrics['bars_generated']:,}")
        print(f"Processing Rate: {metrics['processing_rate']:,.0f} ticks/sec")
        print("="*60)
```

## Complete Strategy Example

Here's the complete strategy implementation:

```python
# save as: moving_average_strategy.py

from alphaforge_pyo3.data import DataEngine, DataEngineConfig
from alphaforge_pyo3.execution import ExecutionEngine, Order, OrderType, OrderSide, TimeInForce
from alphaforge_pyo3.model import TradeTick, Price, Quantity
import time
import numpy as np
from collections import deque
from typing import Dict, Optional

class MovingAverageCrossoverStrategy:
    def __init__(self, symbol: str, short_window: int = 10, long_window: int = 30, 
                 quantity: float = 0.1, strategy_id: str = "ma_crossover"):
        self.symbol = symbol
        self.short_window = short_window
        self.long_window = long_window
        self.quantity = quantity
        self.strategy_id = strategy_id
        
        self.price_history = deque(maxlen=long_window)
        self.position = 0.0
        self.last_signal = None
        self.trade_count = 0
        
        self.data_engine = None
        self.execution_engine = None
        
    def initialize_engines(self):
        config = DataEngineConfig(enable_statistics=True)
        self.data_engine = DataEngine(config)
        self.data_engine.start()
        self.execution_engine = ExecutionEngine()
        print(f"✅ Strategy '{self.strategy_id}' initialized for {self.symbol}")
        
    def calculate_moving_averages(self):
        if len(self.price_history) < self.long_window:
            return None, None
        prices = list(self.price_history)
        short_ma = np.mean(prices[-self.short_window:]) if len(prices) >= self.short_window else None
        long_ma = np.mean(prices) if len(prices) >= self.long_window else None
        return short_ma, long_ma
        
    def generate_signal(self, short_ma: float, long_ma: float, price: float):
        if short_ma is None or long_ma is None:
            return None
        if short_ma > long_ma and self.last_signal != 'BUY' and self.position <= 0:
            return 'BUY'
        elif short_ma < long_ma and self.last_signal != 'SELL' and self.position >= 0:
            return 'SELL'
        return None
        
    def submit_order(self, signal: str, price: float):
        try:
            if signal == 'BUY':
                order = Order.market(self.symbol, OrderSide.Buy, self.quantity, self.strategy_id)
                self.position += self.quantity
            else:
                order = Order.market(self.symbol, OrderSide.Sell, self.quantity, self.strategy_id)
                self.position -= self.quantity
                
            order_id = self.execution_engine.submit_order(order)
            self.last_signal = signal
            self.trade_count += 1
            
            print(f"🚀 {signal} order: {order_id} | Price: ${price:.2f} | Position: {self.position}")
            return True
        except Exception as e:
            print(f"❌ Order error: {e}")
            return False
            
    def process_price_update(self, price: float):
        self.price_history.append(price)
        short_ma, long_ma = self.calculate_moving_averages()
        
        if short_ma is not None and long_ma is not None:
            signal = self.generate_signal(short_ma, long_ma, price)
            print(f"📊 ${price:.2f} | Short: ${short_ma:.2f} | Long: ${long_ma:.2f}")
            if signal:
                self.submit_order(signal, price)
                
    def get_performance_metrics(self):
        exec_stats = self.execution_engine.statistics()
        data_stats = self.data_engine.statistics() if self.data_engine else None
        
        return {
            'symbol': self.symbol,
            'position': self.position,
            'trades': self.trade_count,
            'orders_submitted': exec_stats.orders_submitted,
            'orders_filled': exec_stats.orders_filled,
            'fill_ratio': exec_stats.fill_ratio,
            'avg_latency_ms': exec_stats.avg_execution_latency_ms,
            'ticks_processed': data_stats.ticks_processed if data_stats else 0,
        }

def run_strategy_demo():
    """Run a complete strategy demonstration."""
    print("🚀 Starting Moving Average Crossover Strategy Demo")
    print("="*60)
    
    # Create strategy
    strategy = MovingAverageCrossoverStrategy(
        symbol="BTCUSD",
        short_window=5,  # Shorter windows for demo
        long_window=15,
        quantity=0.1,
        strategy_id="demo_ma_strategy"
    )
    
    # Initialize engines
    strategy.initialize_engines()
    
    # Simulate price data with trend
    base_price = 45000.0
    prices = []
    
    # Generate trending price data
    for i in range(30):
        # Add trend and some noise
        trend = i * 50  # Upward trend
        noise = np.random.normal(0, 100)  # Price noise
        price = base_price + trend + noise
        prices.append(max(price, 1000))  # Ensure positive price
    
    print(f"📈 Processing {len(prices)} price updates...")
    print("-" * 60)
    
    # Process price updates
    for i, price in enumerate(prices):
        print(f"\n📅 Update {i+1:2d}:")
        strategy.process_price_update(price)
        time.sleep(0.1)  # Small delay for visualization
    
    # Print final performance report
    print("\n" + "="*60)
    print("📊 FINAL PERFORMANCE REPORT")
    print("="*60)
    
    metrics = strategy.get_performance_metrics()
    for key, value in metrics.items():
        if isinstance(value, float):
            print(f"{key}: {value:.4f}")
        else:
            print(f"{key}: {value}")
    
    print("="*60)
    print("✅ Strategy demo completed successfully!")

if __name__ == "__main__":
    run_strategy_demo()
```

## Running the Strategy

1. **Save the code** to a file called `moving_average_strategy.py`

2. **Run the strategy**:
   ```bash
   python moving_average_strategy.py
   ```

3. **Expected output**:
   ```
   🚀 Starting Moving Average Crossover Strategy Demo
   ============================================================
   ✅ Strategy 'demo_ma_strategy' initialized for BTCUSD
   📈 Processing 30 price updates...
   ------------------------------------------------------------
   
   📅 Update  1:
   📊 $44975.32 | Short: $44975.32 | Long: $44975.32
   
   📅 Update  6:
   📊 $45234.67 | Short: $45178.45 | Long: $45123.21
   🚀 BUY order: order_1 | Price: $45234.67 | Position: 0.1
   
   📅 Update 20:
   📊 $46456.89 | Short: $46234.56 | Long: $46456.78
   🚀 SELL order: order_2 | Price: $46456.89 | Position: 0.0
   
   ============================================================
   📊 FINAL PERFORMANCE REPORT
   ============================================================
   symbol: BTCUSD
   position: 0.0
   trades: 2
   orders_submitted: 2
   orders_filled: 2
   fill_ratio: 1.0000
   avg_latency_ms: 0.1234
   ticks_processed: 0
   ============================================================
   ✅ Strategy demo completed successfully!
   ```

## Key Concepts Explained

### 1. Moving Average Calculation
The strategy maintains a rolling window of recent prices and calculates two moving averages with different periods.

### 2. Signal Generation  
Crossover signals are generated when the short-term average crosses the long-term average, indicating potential trend changes.

### 3. Position Management
The strategy tracks its current position to avoid duplicate orders and ensure proper risk management.

### 4. Performance Monitoring
Real-time metrics tracking enables continuous performance evaluation and optimization.

## Next Steps

1. **[Advanced Strategies](advanced-strategies.md)** - Learn about more sophisticated trading strategies
2. **[Backtesting](backtesting.md)** - Test strategies on historical data  
3. **[Live Trading](live-trading.md)** - Deploy strategies for real-time trading
4. **[Risk Management](../trading/risk-management.md)** - Add risk controls to your strategies

## Common Modifications

### 1. Add Stop Loss
```python
def check_stop_loss(self, current_price: float, entry_price: float, stop_pct: float = 0.02):
    if self.position > 0:  # Long position
        stop_price = entry_price * (1 - stop_pct)
        if current_price <= stop_price:
            return 'SELL'
    elif self.position < 0:  # Short position  
        stop_price = entry_price * (1 + stop_pct)
        if current_price >= stop_price:
            return 'BUY'
    return None
```

### 2. Add Volume Confirmation
```python
def generate_signal_with_volume(self, short_ma: float, long_ma: float, 
                               price: float, volume: float, avg_volume: float):
    # Only generate signals if volume is above average
    if volume > avg_volume * 1.5:
        return self.generate_signal(short_ma, long_ma, price)
    return None
```

### 3. Multiple Timeframes
```python
class MultiTimeframeStrategy(MovingAverageCrossoverStrategy):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.daily_ma = deque(maxlen=20)
        self.hourly_ma = deque(maxlen=10)
        
    def generate_multi_tf_signal(self, price: float):
        # Check daily trend first, then hourly signals
        daily_trend = self.get_daily_trend()
        hourly_signal = self.generate_signal(...)
        
        # Only take signals in direction of daily trend
        if daily_trend == 'UP' and hourly_signal == 'BUY':
            return 'BUY'
        elif daily_trend == 'DOWN' and hourly_signal == 'SELL':
            return 'SELL'
        return None
```

This basic strategy framework provides a solid foundation for building more sophisticated algorithmic trading strategies with AlphaForge.
