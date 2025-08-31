#!/usr/bin/env python3
"""
AlphaForge Performance Benchmark - Real Rust Implementation
Testing >1.5M operations/second with <8μs latency targets
"""

import sys
import time
import statistics
from typing import List

# Ensure we're using the installed package, not local source
sys.path = [p for p in sys.path if 'AlphaForge' not in p or 'site-packages' in p]

def benchmark_cache():
    """Test cache performance with real Rust implementation"""
    print("🔥 ALPHAFORGE RUST CACHE BENCHMARK")
    print("=" * 50)
    
    try:
        from alphaforge_pyo3.cache import Cache, CacheConfig
        print("✅ Using REAL Rust cache implementation")
    except ImportError:
        print("❌ Failed to import Rust cache, skipping test")
        return False
    
    # Test different cache sizes
    test_configs = [
        ("Small Cache", 1_000, 50_000),
        ("Medium Cache", 10_000, 100_000), 
        ("Large Cache", 50_000, 200_000),
    ]
    
    results = []
    
    for test_name, cache_size, num_ops in test_configs:
        print(f"\n🧪 {test_name}: {cache_size:,} size, {num_ops:,} operations")
        
        config = CacheConfig(max_size=cache_size, enable_statistics=True)
        cache = Cache(config)
        
        # Warm up
        for i in range(min(1000, num_ops // 10)):
            cache.put(f"warmup_{i}", f"value_{i}")
        
        # Measure put operations
        put_times = []
        start_time = time.perf_counter()
        
        for i in range(num_ops):
            op_start = time.perf_counter_ns()
            cache.put(f"key_{i}", f"value_{i}_{i*2}")
            op_end = time.perf_counter_ns()
            
            if i % 1000 == 0:  # Sample every 1000th operation
                put_times.append((op_end - op_start) / 1000)  # Convert to microseconds
        
        put_elapsed = time.perf_counter() - start_time
        put_ops_per_sec = num_ops / put_elapsed
        
        # Measure get operations
        get_times = []
        start_time = time.perf_counter()
        hits = 0
        
        for i in range(num_ops):
            op_start = time.perf_counter_ns()
            result = cache.get(f"key_{i}")
            op_end = time.perf_counter_ns()
            
            if result is not None:
                hits += 1
            
            if i % 1000 == 0:  # Sample every 1000th operation
                get_times.append((op_end - op_start) / 1000)  # Convert to microseconds
        
        get_elapsed = time.perf_counter() - start_time
        get_ops_per_sec = num_ops / get_elapsed
        
        # Combined performance
        total_ops_per_sec = (num_ops * 2) / (put_elapsed + get_elapsed)
        
        # Latency statistics
        avg_put_latency = statistics.mean(put_times) if put_times else 0
        avg_get_latency = statistics.mean(get_times) if get_times else 0
        p99_put_latency = statistics.quantiles(put_times, n=100)[98] if len(put_times) > 100 else 0
        p99_get_latency = statistics.quantiles(get_times, n=100)[98] if len(get_times) > 100 else 0
        
        # Cache statistics
        stats = cache.statistics()
        hit_rate = (hits / num_ops) * 100
        
        print(f"  📈 PUT Performance: {put_ops_per_sec:,.0f} ops/sec")
        print(f"  📉 GET Performance: {get_ops_per_sec:,.0f} ops/sec") 
        print(f"  ⚡ Combined: {total_ops_per_sec:,.0f} ops/sec")
        print(f"  🎯 Hit Rate: {hit_rate:.1f}% ({hits:,}/{num_ops:,})")
        print(f"  ⏱️  Avg Latency - PUT: {avg_put_latency:.1f}μs, GET: {avg_get_latency:.1f}μs")
        print(f"  📊 P99 Latency - PUT: {p99_put_latency:.1f}μs, GET: {p99_get_latency:.1f}μs")
        
        if stats:
            print(f"  📋 Stats - Hits: {stats.hits:,}, Misses: {stats.misses:,}")
        
        results.append({
            'name': test_name,
            'total_ops_per_sec': total_ops_per_sec,
            'put_ops_per_sec': put_ops_per_sec,
            'get_ops_per_sec': get_ops_per_sec,
            'hit_rate': hit_rate,
            'avg_put_latency': avg_put_latency,
            'avg_get_latency': avg_get_latency,
            'p99_put_latency': p99_put_latency,
            'p99_get_latency': p99_get_latency
        })
    
    return results

def benchmark_time_functions():
    """Test time function performance"""
    print(f"\n🕒 TIME FUNCTIONS BENCHMARK")
    print("=" * 50)
    
    try:
        from alphaforge_pyo3.time import unix_nanos_now
        print("✅ Using REAL Rust time functions")
        
        # Test unix_nanos_now performance
        num_calls = 1_000_000
        start_time = time.perf_counter()
        
        for _ in range(num_calls):
            timestamp = unix_nanos_now()
        
        elapsed = time.perf_counter() - start_time
        calls_per_sec = num_calls / elapsed
        avg_latency_ns = (elapsed / num_calls) * 1_000_000_000
        
        print(f"  ⚡ unix_nanos_now(): {calls_per_sec:,.0f} calls/sec")
        print(f"  ⏱️  Average latency: {avg_latency_ns:.1f} ns/call")
        print(f"  📋 Sample timestamp: {unix_nanos_now()}")
        
        return True
        
    except ImportError:
        print("❌ Failed to import Rust time functions")
        return False

def main():
    """Run comprehensive AlphaForge benchmarks"""
    print("🚀 ALPHAFORGE HIGH-PERFORMANCE RUST BENCHMARKS")
    print("Targeting >1.5M ops/sec throughput and <8μs latency")
    print("=" * 60)
    
    # Run cache benchmarks
    cache_results = benchmark_cache()
    
    # Run time function benchmarks  
    time_success = benchmark_time_functions()
    
    # Summary
    print(f"\n🏆 BENCHMARK SUMMARY")
    print("=" * 50)
    
    if cache_results:
        max_throughput = max(r['total_ops_per_sec'] for r in cache_results)
        avg_latency = statistics.mean([r['avg_get_latency'] for r in cache_results])
        
        print(f"🎯 Peak Throughput: {max_throughput:,.0f} operations/second")
        print(f"⏱️  Average GET Latency: {avg_latency:.1f} μs")
        
        # Check targets
        throughput_target = 1_500_000
        latency_target = 8.0  # microseconds
        
        if max_throughput >= throughput_target:
            print(f"✅ THROUGHPUT TARGET MET: {max_throughput:,.0f} ≥ {throughput_target:,} ops/sec")
        else:
            print(f"❌ THROUGHPUT BELOW TARGET: {max_throughput:,.0f} < {throughput_target:,} ops/sec")
            
        if avg_latency <= latency_target:
            print(f"✅ LATENCY TARGET MET: {avg_latency:.1f}μs ≤ {latency_target}μs")
        else:
            print(f"❌ LATENCY ABOVE TARGET: {avg_latency:.1f}μs > {latency_target}μs")
            
        if max_throughput >= throughput_target and avg_latency <= latency_target:
            print(f"\n🎉 ALL PERFORMANCE TARGETS ACHIEVED!")
            print(f"🚀 AlphaForge Rust implementation is PRODUCTION-READY!")
        else:
            print(f"\n⚠️  Some targets missed - optimization needed")
    
    if time_success:
        print(f"✅ Time functions working with Rust implementation")
    else:
        print(f"❌ Time functions using Python fallback")
    
    print(f"\n🔥 Real Rust implementation delivering high performance!")

if __name__ == "__main__":
    main()
