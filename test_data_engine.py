#!/usr/bin/env python3
"""
Test script for the Data Engine Python bindings.

This test verifies:
- Data Engine configuration and initialization
- Trade tick and quote tick processing
- Bar aggregation functionality
- Statistics tracking
- Performance characteristics
"""

import sys
import os
import time

# Add the site-packages path directly
sys.path.insert(0, r'D:\AlphaForge\.venv\Lib\site-packages\alphaforge_pyo3')

# Add the project root to the path to import alphaforge modules
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

def test_data_engine_basic():
    """Test basic Data Engine functionality"""
    print("🚀 Testing Data Engine Basic Functionality")
    
    try:
        # Import the compiled Rust extension
        import alphaforge_pyo3
        DataEngine = alphaforge_pyo3.data.DataEngine
        DataEngineConfig = alphaforge_pyo3.data.DataEngineConfig
        TradeTick = alphaforge_pyo3.data.TradeTick
        QuoteTick = alphaforge_pyo3.data.QuoteTick
        BarType = alphaforge_pyo3.data.BarType
        
        # Create configuration
        config = DataEngineConfig(
            max_bars_per_instrument=1000,
            max_tick_buffer_size=500,
            enable_bar_aggregation=True,
            enable_order_book_deltas=True,
            enable_statistics=True
        )
        
        # Create Data Engine
        engine = DataEngine(config)
        
        # Start the engine
        engine.start()
        print("✅ Data Engine started successfully")
        
        # Check if running
        assert engine.is_running(), "Engine should be running"
        print("✅ Engine running status verified")
        
        # Add bar aggregator
        bar_type = BarType("BTC.USD", 100, "tick")  # 100-tick bars
        engine.add_bar_aggregator(bar_type)
        print("✅ Bar aggregator added")
        
        # Process some trade ticks
        for i in range(150):  # More than 100 to generate bar
            tick = TradeTick(
                instrument_id="BTC.USD", 
                price=50000.0 + i * 10.0,
                size=1.0,
                aggressor_side=0,  # Buyer
                trade_id=f"trade_{i}",
                ts_event=int(time.time() * 1_000_000_000) + i * 1_000_000,  # nanoseconds
                ts_init=int(time.time() * 1_000_000_000) + i * 1_000_000
            )
            
            # Process tick - should return bar when we hit 100 ticks
            result = engine.process_trade_tick(tick)
            if result is not None:
                print(f"✅ Bar generated: Open={result.open}, High={result.high}, Low={result.low}, Close={result.close}, Volume={result.volume}")
        
        # Process some quote ticks
        for i in range(50):
            quote = QuoteTick(
                instrument_id="BTC.USD",
                bid_price=49990.0 + i * 5.0,
                ask_price=50010.0 + i * 5.0,
                bid_size=10.0,
                ask_size=10.0,
                ts_event=int(time.time() * 1_000_000_000) + i * 2_000_000,
                ts_init=int(time.time() * 1_000_000_000) + i * 2_000_000
            )
            
            engine.process_quote_tick(quote)
        
        print("✅ Processed 50 quote ticks")
        
        # Check statistics
        stats = engine.statistics()
        print(f"✅ Statistics: Ticks={stats.ticks_processed}, Bars={stats.bars_generated}, Rate={stats.processing_rate:.2f} ticks/sec")
        
        # Get recent bars
        recent_bars = engine.get_recent_bars(bar_type, 5)
        print(f"✅ Retrieved {len(recent_bars)} recent bars")
        
        for i, bar in enumerate(recent_bars):
            print(f"   Bar {i}: O={bar.open}, H={bar.high}, L={bar.low}, C={bar.close}, V={bar.volume}")
        
        # Check processed count
        processed = engine.processed_count()
        print(f"✅ Total processed count: {processed}")
        
        # Stop the engine
        engine.stop()
        assert not engine.is_running(), "Engine should be stopped"
        print("✅ Data Engine stopped successfully")
        
        return True
        
    except ImportError as e:
        print(f"❌ Failed to import Data Engine: {e}")
        return False
    except Exception as e:
        print(f"❌ Data Engine test failed: {e}")
        import traceback
        traceback.print_exc()
        return False

def test_data_engine_performance():
    """Test Data Engine performance characteristics"""
    print("\n🏁 Testing Data Engine Performance")
    
    try:
        import alphaforge_pyo3
        DataEngine = alphaforge_pyo3.data.DataEngine
        DataEngineConfig = alphaforge_pyo3.data.DataEngineConfig
        TradeTick = alphaforge_pyo3.data.TradeTick
        BarType = alphaforge_pyo3.data.BarType
        
        # Create optimized configuration
        config = DataEngineConfig(
            max_bars_per_instrument=10000,
            max_tick_buffer_size=1000,
            enable_bar_aggregation=True,
            enable_order_book_deltas=False,  # Disable for performance
            enable_statistics=True
        )
        
        engine = DataEngine(config)
        engine.start()
        
        # Add bar aggregator
        bar_type = BarType("BTC.USD", 1000, "tick")  # 1000-tick bars
        engine.add_bar_aggregator(bar_type)
        
        # Performance test - process many ticks
        num_ticks = 10000
        start_time = time.time()
        
        for i in range(num_ticks):
            tick = TradeTick(
                instrument_id="BTC.USD",
                price=50000.0 + (i % 1000) * 1.0,  # Price variation
                size=1.0 + (i % 10) * 0.1,
                aggressor_side=i % 2,  # Alternate buyer/seller
                trade_id=f"perf_trade_{i}",
                ts_event=int(time.time() * 1_000_000_000) + i * 100_000,
                ts_init=int(time.time() * 1_000_000_000) + i * 100_000
            )
            
            engine.process_trade_tick(tick)
        
        end_time = time.time()
        duration = end_time - start_time
        throughput = num_ticks / duration
        
        print(f"✅ Processed {num_ticks} ticks in {duration:.3f}s")
        print(f"✅ Throughput: {throughput:,.0f} ticks/second")
        print(f"✅ Average latency: {(duration * 1_000_000 / num_ticks):.2f} μs per tick")
        
        # Check final statistics
        stats = engine.statistics()
        print(f"✅ Final stats: Ticks={stats.ticks_processed}, Bars={stats.bars_generated}")
        print(f"✅ Processing rate: {stats.processing_rate:,.0f} ticks/sec")
        print(f"✅ Memory usage: {stats.memory_usage:,} bytes")
        
        engine.stop()
        return True
        
    except Exception as e:
        print(f"❌ Performance test failed: {e}")
        import traceback
        traceback.print_exc()
        return False

def test_data_engine_error_handling():
    """Test Data Engine error handling"""
    print("\n🛡️ Testing Data Engine Error Handling")
    
    try:
        import alphaforge_pyo3
        DataEngine = alphaforge_pyo3.data.DataEngine
        DataEngineConfig = alphaforge_pyo3.data.DataEngineConfig
        TradeTick = alphaforge_pyo3.data.TradeTick
        
        config = DataEngineConfig()
        engine = DataEngine(config)
        
        # Test invalid instrument ID
        try:
            tick = TradeTick(
                instrument_id="invalid_id_that_should_fail",
                price=50000.0,
                size=1.0,
                aggressor_side=5,  # Invalid aggressor side
                trade_id="error_test",
                ts_event=int(time.time() * 1_000_000_000),
                ts_init=int(time.time() * 1_000_000_000)
            )
            print("❌ Should have failed with invalid aggressor side")
            return False
        except ValueError as e:
            print(f"✅ Correctly caught invalid aggressor side: {e}")
        
        # Test valid tick
        tick = TradeTick(
            instrument_id="123456",  # Numeric string ID
            price=50000.0,
            size=1.0,
            aggressor_side=0,
            trade_id="valid_test",
            ts_event=int(time.time() * 1_000_000_000),
            ts_init=int(time.time() * 1_000_000_000)
        )
        print("✅ Valid tick created successfully")
        
        return True
        
    except Exception as e:
        print(f"❌ Error handling test failed: {e}")
        import traceback
        traceback.print_exc()
        return False

def main():
    """Run all Data Engine tests"""
    print("=" * 60)
    print("🧪 ALPHAFORGE DATA ENGINE PYTHON BINDINGS TEST")
    print("=" * 60)
    
    tests = [
        test_data_engine_basic,
        test_data_engine_performance, 
        test_data_engine_error_handling,
    ]
    
    passed = 0
    total = len(tests)
    
    for test_func in tests:
        if test_func():
            passed += 1
            print("✅ PASS")
        else:
            print("❌ FAIL")
        print("-" * 40)
    
    print(f"\n📊 Results: {passed}/{total} tests passed")
    
    if passed == total:
        print("🎉 All Data Engine tests passed!")
        return 0
    else:
        print("💥 Some Data Engine tests failed!")
        return 1

if __name__ == "__main__":
    sys.exit(main())
