#!/usr/bin/env python3
"""Test the real Rust cache implementation"""

import sys
import time

# Ensure we're using the installed package, not local source
sys.path = [p for p in sys.path if 'AlphaForge' not in p or 'site-packages' in p]

try:
    import alphaforge_pyo3
    print("✅ Successfully imported alphaforge_pyo3")
    print(f"Available modules: {[x for x in dir(alphaforge_pyo3) if not x.startswith('_')]}")
    
    # Import cache components
    from alphaforge_pyo3.cache import Cache, CacheConfig, CacheStatistics
    print("✅ Successfully imported Cache classes from Rust extension")
    
    # Test basic cache operations
    config = CacheConfig(max_size=1000, enable_statistics=True)
    cache = Cache(config)
    
    print("\n🧪 Testing Rust Cache Performance...")
    
    # Performance test
    num_operations = 100_000
    start_time = time.perf_counter()
    
    # Fill cache
    for i in range(num_operations):
        key = f"key_{i}"
        value = f"value_{i}"
        cache.put(key, value)
    
    # Read from cache
    hits = 0
    for i in range(num_operations):
        key = f"key_{i}"
        result = cache.get(key)
        if result is not None:
            hits += 1
    
    end_time = time.perf_counter()
    elapsed = end_time - start_time
    ops_per_sec = (num_operations * 2) / elapsed  # *2 because we do put + get
    
    print(f"✅ Completed {num_operations} put + {num_operations} get operations")
    print(f"⚡ Performance: {ops_per_sec:,.0f} operations/second")
    print(f"🎯 Cache hits: {hits}/{num_operations}")
    print(f"📊 Cache size: {cache.size()}")
    
    # Check statistics
    stats = cache.statistics()
    if stats:
        print(f"📈 Statistics - Hits: {stats.hits}, Misses: {stats.misses}, Hit Rate: {stats.hit_rate:.1f}%")
        print(f"💾 Memory usage: {stats.memory_usage} bytes")
    
    # Test target performance (>1.5M ops/sec)
    target_ops_per_sec = 1_500_000
    if ops_per_sec >= target_ops_per_sec:
        print(f"🎉 SUCCESS: Achieved {ops_per_sec:,.0f} ops/sec (target: {target_ops_per_sec:,} ops/sec)")
        print("🚀 Real Rust implementation is working!")
    else:
        print(f"⚠️  Below target: {ops_per_sec:,.0f} ops/sec (target: {target_ops_per_sec:,} ops/sec)")
        print("🐍 Might be using Python fallback")

except ImportError as e:
    print(f"❌ Import failed: {e}")
    print("🐍 Using Python fallback implementation")
except Exception as e:
    print(f"❌ Test failed: {e}")
    import traceback
    traceback.print_exc()
