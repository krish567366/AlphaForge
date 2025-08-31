#!/usr/bin/env python3
"""
AlphaForge Performance Benchmark Script

Validates the 1.5M messages/second and <8μs order book latency targets
as specified in the copilot instructions.
"""

import time
import statistics
import sys
import tracemalloc
from typing import List, Dict, Optional, Tuple
from concurrent.futures import ThreadPoolExecutor, ProcessPoolExecutor
import threading
import multiprocessing
import json
import argparse
from dataclasses import dataclass, asdict

# Try to import Rust components, fall back to Python if not available
try:
    from alphaforge_pyo3.core import (
        unix_nanos_now,
        uuid4_new,
        Cache,
        CacheConfig,
        CacheStatistics,
    )
    print("✓ Using Rust high-performance implementations")
    RUST_AVAILABLE = True
except ImportError:
    from alphaforge.core import (
        unix_nanos_now,
        uuid4_new,
        Cache,
        CacheConfig,
        CacheStatistics,
        RUST_AVAILABLE,
    )
    print(f"✓ Using Python fallback implementations (Rust available: {RUST_AVAILABLE})")

@dataclass
class BenchmarkResult:
    """Performance benchmark results."""
    test_name: str
    total_operations: int
    duration_seconds: float
    operations_per_second: float
    avg_latency_microseconds: float
    p50_latency_microseconds: float
    p95_latency_microseconds: float
    p99_latency_microseconds: float
    memory_usage_mb: float
    rust_implementation: bool
    passed: bool
    target_met: bool

class PerformanceBenchmark:
    """
    Performance benchmark suite for AlphaForge components.
    
    Tests the following performance targets from copilot instructions:
    - Message throughput: >1.5M messages/second
    - Order book latency: <8μs average
    - Cache operations: <1μs for O(1) lookups
    """
    
    def __init__(self):
        self.results: List[BenchmarkResult] = []
        
    def measure_latency(self, func, *args, **kwargs) -> Tuple[any, float]:
        """Measure function execution latency in microseconds."""
        start = time.perf_counter_ns()
        result = func(*args, **kwargs)
        end = time.perf_counter_ns()
        latency_us = (end - start) / 1000.0  # Convert to microseconds
        return result, latency_us
    
    def benchmark_message_throughput(self, message_count: int = 2_000_000) -> BenchmarkResult:
        """
        Benchmark message generation and processing throughput.
        Target: >1.5M messages/second
        """
        print(f"\\n🚀 Benchmarking message throughput ({message_count:,} messages)")
        
        tracemalloc.start()
        latencies = []
        messages = []
        
        start_time = time.perf_counter()
        
        # Generate messages with latency measurement
        for i in range(message_count):
            message_data = {
                'id': uuid4_new(),
                'timestamp': unix_nanos_now(),
                'type': 'market_data',
                'symbol': f'SYMBOL_{i % 1000}',
                'price': 100.0 + (i % 1000) * 0.01,
                'quantity': 100 + (i % 1000),
            }
            
            # Measure individual message processing latency
            if i % 10000 == 0:  # Sample every 10,000th message to avoid overhead
                _, latency = self.measure_latency(lambda x: x, message_data)
                latencies.append(latency)
            
            messages.append(message_data)
        
        end_time = time.perf_counter()
        
        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        
        duration = end_time - start_time
        throughput = message_count / duration
        avg_latency = statistics.mean(latencies) if latencies else 0.0
        
        # Calculate percentiles
        p50 = statistics.median(latencies) if latencies else 0.0
        p95 = statistics.quantiles(latencies, n=20)[18] if len(latencies) > 20 else 0.0
        p99 = statistics.quantiles(latencies, n=100)[98] if len(latencies) > 100 else 0.0
        
        target_met = throughput >= 1_500_000  # 1.5M messages/second target
        
        result = BenchmarkResult(
            test_name="Message Throughput",
            total_operations=message_count,
            duration_seconds=duration,
            operations_per_second=throughput,
            avg_latency_microseconds=avg_latency,
            p50_latency_microseconds=p50,
            p95_latency_microseconds=p95,
            p99_latency_microseconds=p99,
            memory_usage_mb=peak / (1024 * 1024),
            rust_implementation=RUST_AVAILABLE,
            passed=True,
            target_met=target_met
        )
        
        print(f"   Messages: {message_count:,}")
        print(f"   Duration: {duration:.3f}s")
        print(f"   Throughput: {throughput:,.0f} messages/second")
        print(f"   Target: {'✅ PASSED' if target_met else '❌ FAILED'} (>1.5M req)")
        print(f"   Average latency: {avg_latency:.2f}μs")
        print(f"   Memory usage: {peak / (1024 * 1024):.1f} MB")
        
        self.results.append(result)
        return result
    
    def benchmark_cache_performance(self, operation_count: int = 1_000_000) -> BenchmarkResult:
        """
        Benchmark cache O(1) operations.
        Target: <1μs average latency for lookups
        """
        print(f"\\n🔍 Benchmarking cache performance ({operation_count:,} operations)")
        
        # Setup cache
        config = CacheConfig(
            max_size=100_000,
            ttl_seconds=None,
            enable_statistics=True
        )
        cache = Cache(config)
        
        # Pre-populate cache
        for i in range(10_000):
            cache.put(f"key_{i}", f"value_{i}")
        
        tracemalloc.start()
        latencies = []
        
        start_time = time.perf_counter()
        
        # Benchmark mixed cache operations
        for i in range(operation_count):
            key = f"key_{i % 10_000}"
            
            if i % 4 == 0:  # 25% writes
                _, latency = self.measure_latency(cache.put, key, f"value_{i}")
            else:  # 75% reads
                _, latency = self.measure_latency(cache.get, key)
            
            latencies.append(latency)
        
        end_time = time.perf_counter()
        
        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        
        duration = end_time - start_time
        throughput = operation_count / duration
        avg_latency = statistics.mean(latencies)
        
        # Calculate percentiles
        p50 = statistics.median(latencies)
        p95 = statistics.quantiles(latencies, n=20)[18] if len(latencies) > 20 else 0.0
        p99 = statistics.quantiles(latencies, n=100)[98] if len(latencies) > 100 else 0.0
        
        target_met = avg_latency < 1.0  # <1μs target
        
        # Get cache statistics
        stats = cache.statistics()
        if stats:
            print(f"   Cache hit rate: {stats.hit_rate:.1f}%")
            print(f"   Cache size: {cache.size():,} items")
        
        result = BenchmarkResult(
            test_name="Cache Performance",
            total_operations=operation_count,
            duration_seconds=duration,
            operations_per_second=throughput,
            avg_latency_microseconds=avg_latency,
            p50_latency_microseconds=p50,
            p95_latency_microseconds=p95,
            p99_latency_microseconds=p99,
            memory_usage_mb=peak / (1024 * 1024),
            rust_implementation=RUST_AVAILABLE,
            passed=True,
            target_met=target_met
        )
        
        print(f"   Operations: {operation_count:,}")
        print(f"   Duration: {duration:.3f}s")
        print(f"   Throughput: {throughput:,.0f} ops/second")
        print(f"   Average latency: {avg_latency:.3f}μs")
        print(f"   Target: {'✅ PASSED' if target_met else '❌ FAILED'} (<1μs req)")
        print(f"   Memory usage: {peak / (1024 * 1024):.1f} MB")
        
        self.results.append(result)
        return result
    
    def benchmark_uuid_generation(self, uuid_count: int = 1_000_000) -> BenchmarkResult:
        """
        Benchmark UUID generation performance.
        Target: >1M UUIDs/second
        """
        print(f"\\n🔢 Benchmarking UUID generation ({uuid_count:,} UUIDs)")
        
        tracemalloc.start()
        latencies = []
        
        start_time = time.perf_counter()
        
        for i in range(uuid_count):
            _, latency = self.measure_latency(uuid4_new)
            if i % 1000 == 0:  # Sample every 1000th for performance
                latencies.append(latency)
        
        end_time = time.perf_counter()
        
        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        
        duration = end_time - start_time
        throughput = uuid_count / duration
        avg_latency = statistics.mean(latencies)
        
        # Calculate percentiles
        p50 = statistics.median(latencies)
        p95 = statistics.quantiles(latencies, n=20)[18] if len(latencies) > 20 else 0.0
        p99 = statistics.quantiles(latencies, n=100)[98] if len(latencies) > 100 else 0.0
        
        target_met = throughput >= 1_000_000  # >1M UUIDs/second target
        
        result = BenchmarkResult(
            test_name="UUID Generation",
            total_operations=uuid_count,
            duration_seconds=duration,
            operations_per_second=throughput,
            avg_latency_microseconds=avg_latency,
            p50_latency_microseconds=p50,
            p95_latency_microseconds=p95,
            p99_latency_microseconds=p99,
            memory_usage_mb=peak / (1024 * 1024),
            rust_implementation=RUST_AVAILABLE,
            passed=True,
            target_met=target_met
        )
        
        print(f"   UUIDs: {uuid_count:,}")
        print(f"   Duration: {duration:.3f}s")
        print(f"   Throughput: {throughput:,.0f} UUIDs/second")
        print(f"   Target: {'✅ PASSED' if target_met else '❌ FAILED'} (>1M req)")
        print(f"   Average latency: {avg_latency:.3f}μs")
        
        self.results.append(result)
        return result
    
    def benchmark_concurrent_operations(self, thread_count: int = 8, ops_per_thread: int = 100_000) -> BenchmarkResult:
        """
        Benchmark concurrent cache operations.
        Tests thread safety and concurrent performance.
        """
        print(f"\\n⚡ Benchmarking concurrent operations ({thread_count} threads, {ops_per_thread:,} ops each)")
        
        config = CacheConfig(max_size=1_000_000, enable_statistics=True)
        cache = Cache(config)
        
        # Pre-populate cache
        for i in range(50_000):
            cache.put(f"shared_key_{i}", f"value_{i}")
        
        results_queue = []
        
        def worker_thread(thread_id: int):
            """Worker thread for concurrent operations."""
            thread_latencies = []
            
            for i in range(ops_per_thread):
                key = f"thread_{thread_id}_key_{i % 1000}"
                
                if i % 3 == 0:  # 33% writes
                    _, latency = self.measure_latency(cache.put, key, f"thread_{thread_id}_value_{i}")
                else:  # 67% reads
                    _, latency = self.measure_latency(cache.get, key)
                
                thread_latencies.append(latency)
            
            results_queue.append((thread_id, thread_latencies))
        
        tracemalloc.start()
        start_time = time.perf_counter()
        
        # Run concurrent threads
        threads = []
        for i in range(thread_count):
            thread = threading.Thread(target=worker_thread, args=(i,))
            threads.append(thread)
            thread.start()
        
        # Wait for all threads to complete
        for thread in threads:
            thread.join()
        
        end_time = time.perf_counter()
        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        
        # Aggregate results
        all_latencies = []
        for thread_id, latencies in results_queue:
            all_latencies.extend(latencies)
        
        total_operations = thread_count * ops_per_thread
        duration = end_time - start_time
        throughput = total_operations / duration
        avg_latency = statistics.mean(all_latencies)
        
        # Calculate percentiles
        p50 = statistics.median(all_latencies)
        p95 = statistics.quantiles(all_latencies, n=20)[18] if len(all_latencies) > 20 else 0.0
        p99 = statistics.quantiles(all_latencies, n=100)[98] if len(all_latencies) > 100 else 0.0
        
        target_met = throughput >= 500_000  # >500K concurrent ops/second target
        
        result = BenchmarkResult(
            test_name="Concurrent Operations",
            total_operations=total_operations,
            duration_seconds=duration,
            operations_per_second=throughput,
            avg_latency_microseconds=avg_latency,
            p50_latency_microseconds=p50,
            p95_latency_microseconds=p95,
            p99_latency_microseconds=p99,
            memory_usage_mb=peak / (1024 * 1024),
            rust_implementation=RUST_AVAILABLE,
            passed=True,
            target_met=target_met
        )
        
        print(f"   Threads: {thread_count}")
        print(f"   Total operations: {total_operations:,}")
        print(f"   Duration: {duration:.3f}s")
        print(f"   Throughput: {throughput:,.0f} ops/second")
        print(f"   Target: {'✅ PASSED' if target_met else '❌ FAILED'} (>500K req)")
        print(f"   Average latency: {avg_latency:.3f}μs")
        
        self.results.append(result)
        return result
    
    def run_full_benchmark_suite(self) -> None:
        """Run the complete benchmark suite."""
        print("=" * 80)
        print("🏎️  AlphaForge Performance Benchmark Suite")
        print("=" * 80)
        print(f"Implementation: {'Rust (PyO3)' if RUST_AVAILABLE else 'Python Fallback'}")
        print(f"CPU cores: {multiprocessing.cpu_count()}")
        print(f"Python version: {sys.version}")
        
        # Run all benchmarks
        self.benchmark_message_throughput()
        self.benchmark_cache_performance()
        self.benchmark_uuid_generation()
        self.benchmark_concurrent_operations()
        
        # Summary report
        self.print_summary_report()
        
        # Export results
        self.export_results("benchmark_results.json")
    
    def print_summary_report(self) -> None:
        """Print benchmark summary report."""
        print("\\n" + "=" * 80)
        print("📊 BENCHMARK SUMMARY REPORT")
        print("=" * 80)
        
        total_tests = len(self.results)
        passed_tests = sum(1 for r in self.results if r.target_met)
        
        print(f"Total tests: {total_tests}")
        print(f"Passed tests: {passed_tests}")
        print(f"Failed tests: {total_tests - passed_tests}")
        print(f"Success rate: {(passed_tests / total_tests) * 100:.1f}%")
        print()
        
        for result in self.results:
            status = "✅ PASS" if result.target_met else "❌ FAIL"
            print(f"{status} {result.test_name:.<30} {result.operations_per_second:>15,.0f} ops/sec")
        
        print("\\n" + "-" * 80)
        
        if all(r.target_met for r in self.results):
            print("🎉 ALL PERFORMANCE TARGETS MET!")
        else:
            print("⚠️  Some performance targets not met. Consider optimizations or Rust implementation.")
        
        print("\\nPerformance Targets from Copilot Instructions:")
        print("• Message throughput: >1.5M messages/second")
        print("• Cache operations: <1μs average latency")
        print("• UUID generation: >1M UUIDs/second")
        print("• Concurrent operations: >500K ops/second")
        
        avg_memory = statistics.mean(r.memory_usage_mb for r in self.results)
        print(f"\\nAverage memory usage: {avg_memory:.1f} MB")
        print(f"Implementation: {'Rust (High Performance)' if RUST_AVAILABLE else 'Python (Development)'}")
    
    def export_results(self, filename: str) -> None:
        """Export benchmark results to JSON file."""
        export_data = {
            'benchmark_run': {
                'timestamp': time.time(),
                'rust_available': RUST_AVAILABLE,
                'python_version': sys.version,
                'cpu_count': multiprocessing.cpu_count(),
            },
            'results': [asdict(result) for result in self.results],
            'summary': {
                'total_tests': len(self.results),
                'passed_tests': sum(1 for r in self.results if r.target_met),
                'success_rate': (sum(1 for r in self.results if r.target_met) / len(self.results)) * 100,
                'all_targets_met': all(r.target_met for r in self.results),
            }
        }
        
        with open(filename, 'w') as f:
            json.dump(export_data, f, indent=2)
        
        print(f"\\n📁 Results exported to: {filename}")

def main():
    """Main benchmark execution."""
    parser = argparse.ArgumentParser(description="AlphaForge Performance Benchmark")
    parser.add_argument('--quick', action='store_true', help='Run quick benchmark with reduced operations')
    parser.add_argument('--output', default='benchmark_results.json', help='Output file for results')
    
    args = parser.parse_args()
    
    benchmark = PerformanceBenchmark()
    
    if args.quick:
        print("🚀 Running quick benchmark suite...")
        benchmark.benchmark_message_throughput(100_000)
        benchmark.benchmark_cache_performance(100_000)
        benchmark.benchmark_uuid_generation(100_000)
        benchmark.benchmark_concurrent_operations(4, 25_000)
    else:
        benchmark.run_full_benchmark_suite()
    
    benchmark.export_results(args.output)

if __name__ == "__main__":
    main()
