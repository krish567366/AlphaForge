#!/usr/bin/env python3
"""
AlphaForge Strategy Framework Demo

This example demonstrates the Strategy Framework capabilities including:
- Strategy creation and configuration
- Strategy engine management
- Python bindings for high-performance Rust backend
"""

import sys
import os

# Add the site-packages path directly  
sys.path.insert(0, r'D:\AlphaForge\.venv\Lib\site-packages\alphaforge_pyo3')

try:
    import alphaforge_pyo3 as alphaforge
    print("✅ AlphaForge module loaded successfully")
    
    # ============================================================================
    # Strategy Framework Demo
    # ============================================================================
    
    print("\n🧠 Strategy Framework Demo")
    print("=" * 50)
    
    # Create strategy engine
    print("1. Creating Strategy Engine...")
    engine = alphaforge.strategy.StrategyEngine()
    print(f"   Engine running: {engine.is_running()}")
    print(f"   Total strategies: {engine.total_strategies()}")
    
    # Create strategy configuration
    print("\n2. Creating Strategy Configuration...")
    strategy_config = alphaforge.strategy.StrategyConfig(
        strategy_id=1,
        name="MovingAverageCrossover",
        instruments=["EURUSD"],
        max_position_size=10000.0,
        max_daily_loss=1000.0,
        risk_per_trade=100.0
    )
    
    print(f"   Strategy ID: {strategy_config.strategy_id()}")
    print(f"   Strategy Name: {strategy_config.name()}")
    print(f"   Instruments: {strategy_config.instruments()}")
    print(f"   Max Position Size: {strategy_config.max_position_size()}")
    print(f"   Max Daily Loss: {strategy_config.max_daily_loss()}")
    print(f"   Risk Per Trade: {strategy_config.risk_per_trade()}")
    
    # Add strategy to engine
    print("\n3. Adding Strategy to Engine...")
    engine.add_strategy(1, strategy_config)
    print(f"   Total strategies after add: {engine.total_strategies()}")
    
    # Retrieve strategy config
    print("\n4. Retrieving Strategy Configuration...")
    retrieved_config = engine.get_strategy_config(1)
    if retrieved_config:
        print(f"   Retrieved strategy: {retrieved_config.name()}")
    else:
        print("   Strategy not found")
    
    # Start the engine
    print("\n5. Starting Strategy Engine...")
    engine.start()
    print(f"   Engine running: {engine.is_running()}")
    
    # Stop the engine
    print("\n6. Stopping Strategy Engine...")
    engine.stop()
    print(f"   Engine running: {engine.is_running()}")
    
    # Create strategy metrics
    print("\n7. Creating Strategy Metrics...")
    metrics = alphaforge.strategy.StrategyMetrics()
    print(f"   Total trades: {metrics.total_trades()}")
    print(f"   Winning trades: {metrics.winning_trades()}")
    print(f"   Losing trades: {metrics.losing_trades()}")
    print(f"   Win rate: {metrics.win_rate():.2%}")
    print(f"   Total PnL: ${metrics.total_pnl():.2f}")
    print(f"   Average trade PnL: ${metrics.avg_trade_pnl():.2f}")
    print(f"   Max drawdown: ${metrics.max_drawdown():.2f}")
    print(f"   Sharpe ratio: {metrics.sharpe_ratio():.2f}")
    
    # Create a simple strategy instance
    print("\n8. Creating Strategy Instance...")
    strategy = alphaforge.strategy.Strategy("TestStrategy", "1.0.0")
    print(f"   Strategy Name: {strategy.name()}")
    print(f"   Strategy Version: {strategy.version()}")
    
    print("\n✅ Strategy Framework demo completed successfully!")
    print("\n📊 Strategy Framework Features Demonstrated:")
    print("   • High-performance Rust backend with Python bindings")
    print("   • Strategy configuration and management")
    print("   • Strategy engine lifecycle (start/stop)")
    print("   • Performance metrics tracking")
    print("   • Multiple strategy support")
    print("   • Type-safe strategy identifiers")
    
except ImportError as e:
    print(f"❌ Failed to import alphaforge module: {e}")
    print("Make sure to build the project first with: cargo build")
except Exception as e:
    print(f"❌ Error during strategy framework demo: {e}")
    import traceback
    traceback.print_exc()
