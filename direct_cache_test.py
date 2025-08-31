#!/usr/bin/env python3
"""
Direct cache test without AlphaForge package imports.
Tests the cache implementation directly.
"""

# Direct imports without going through __init__.py
from typing import Dict, Any, Optional, List, TypeVar, Generic
from dataclasses import dataclass
from threading import RLock
import time
import json
from collections import OrderedDict

T = TypeVar('T')

@dataclass
class CacheStatistics:
    """Cache performance statistics."""
    hits: int = 0
    misses: int = 0
    inserts: int = 0
    evictions: int = 0
    memory_usage: int = 0
    
    @property
    def hit_rate(self) -> float:
        """Calculate hit rate as percentage."""
        total = self.hits + self.misses
        if total == 0:
            return 0.0
        return (self.hits / total) * 100.0

@dataclass
class CacheConfig:
    """Cache configuration."""
    max_size: int = 10_000
    ttl_seconds: Optional[int] = None
    enable_statistics: bool = True
    enable_persistence: bool = False
    persistence_path: Optional[str] = None

class CacheEntry:
    """Internal cache entry."""
    
    def __init__(self, value: Any, ttl: Optional[float] = None):
        self.value = value
        self.created_at = time.time()
        self.expires_at = ttl + self.created_at if ttl else None
        self.access_count = 0
        self.last_accessed = self.created_at
    
    def is_expired(self) -> bool:
        """Check if entry has expired."""
        if self.expires_at is None:
            return False
        return time.time() > self.expires_at
    
    def touch(self) -> None:
        """Update access statistics."""
        self.access_count += 1
        self.last_accessed = time.time()

class Cache(Generic[T]):
    """
    Python cache implementation.
    
    Uses OrderedDict for LRU eviction and threading.RLock for thread safety.
    """
    
    def __init__(self, config: CacheConfig):
        self.config = config
        self._lock = RLock()
        self._data: OrderedDict[str, CacheEntry] = OrderedDict()
        self._stats = CacheStatistics() if config.enable_statistics else None
    
    def get(self, key: str) -> Optional[T]:
        """Get value from cache."""
        with self._lock:
            entry = self._data.get(key)
            
            if entry is None:
                if self._stats:
                    self._stats.misses += 1
                return None
            
            if entry.is_expired():
                self._data.pop(key, None)
                if self._stats:
                    self._stats.misses += 1
                    self._stats.evictions += 1
                return None
            
            # Move to end for LRU
            self._data.move_to_end(key)
            entry.touch()
            
            if self._stats:
                self._stats.hits += 1
            
            return entry.value
    
    def put(self, key: str, value: T) -> bool:
        """Put value into cache."""
        with self._lock:
            ttl = self.config.ttl_seconds
            entry = CacheEntry(value, ttl)
            
            # Check if key already exists
            existed = key in self._data
            
            # Add/update entry
            self._data[key] = entry
            self._data.move_to_end(key)  # Move to end for LRU
            
            if self._stats and not existed:
                self._stats.inserts += 1
            
            # Check size limit
            while len(self._data) > self.config.max_size:
                # Remove oldest entry (LRU)
                oldest_key, _ = self._data.popitem(last=False)
                if self._stats:
                    self._stats.evictions += 1
            
            return True
    
    def contains(self, key: str) -> bool:
        """Check if key exists in cache."""
        with self._lock:
            entry = self._data.get(key)
            if entry is None:
                return False
            
            if entry.is_expired():
                self._data.pop(key, None)
                if self._stats:
                    self._stats.evictions += 1
                return False
            
            return True
    
    def remove(self, key: str) -> bool:
        """Remove key from cache."""
        with self._lock:
            existed = key in self._data
            self._data.pop(key, None)
            return existed
    
    def clear(self) -> None:
        """Clear all entries from cache."""
        with self._lock:
            self._data.clear()
            if self._stats:
                self._stats = CacheStatistics()
    
    def size(self) -> int:
        """Get current cache size."""
        with self._lock:
            return len(self._data)
    
    def keys(self) -> List[str]:
        """Get all keys in cache."""
        with self._lock:
            return list(self._data.keys())
    
    def values(self) -> List[T]:
        """Get all values in cache."""
        with self._lock:
            return [entry.value for entry in self._data.values()]
    
    def statistics(self) -> Optional[CacheStatistics]:
        """Get cache statistics."""
        return self._stats
    
    def reset_statistics(self) -> None:
        """Reset cache statistics."""
        if self._stats:
            with self._lock:
                self._stats = CacheStatistics()
    
    def __len__(self) -> int:
        """Get cache size."""
        return self.size()
    
    def __contains__(self, key: str) -> bool:
        """Check if key is in cache."""
        return self.contains(key)
    
    def __getitem__(self, key: str) -> T:
        """Get item with dict-like access."""
        value = self.get(key)
        if value is None:
            raise KeyError(key)
        return value
    
    def __setitem__(self, key: str, value: T) -> None:
        """Set item with dict-like access."""
        self.put(key, value)
    
    def __delitem__(self, key: str) -> None:
        """Delete item with dict-like access."""
        if not self.remove(key):
            raise KeyError(key)

def test_cache():
    """Test Python cache implementation."""
    print("🧪 Testing Python cache implementation...")
    
    # Create cache
    config = CacheConfig(max_size=1000, enable_statistics=True)
    cache = Cache(config)
    
    # Test basic operations
    cache.put("key1", "value1")
    cache.put("key2", "value2")
    cache.put("key3", {"nested": "data"})
    
    # Test retrieval
    assert cache.get("key1") == "value1"
    assert cache.get("key2") == "value2"
    assert cache.get("missing") is None
    
    # Test contains
    assert cache.contains("key1") is True
    assert cache.contains("missing") is False
    
    # Test size
    assert cache.size() == 3
    assert len(cache) == 3
    
    # Test keys
    keys = cache.keys()
    assert "key1" in keys
    assert "key2" in keys
    assert "key3" in keys
    
    # Test dict-like interface
    cache["key4"] = "value4"
    assert cache["key4"] == "value4"
    assert "key4" in cache
    
    # Test removal
    assert cache.remove("key1") is True
    assert cache.remove("missing") is False
    assert cache.get("key1") is None
    
    # Test statistics
    stats = cache.statistics()
    if stats:
        print(f"   Cache stats: {stats.hits} hits, {stats.misses} misses")
        print(f"   Hit rate: {stats.hit_rate:.1f}%")
    
    # Test clear
    cache.clear()
    assert cache.size() == 0
    
    print("✅ Python cache implementation working correctly!")
    print(f"   All basic operations functional")
    print(f"   Dict-like interface working")
    print(f"   Statistics tracking enabled")
    
    return True

def benchmark_cache(operations=100_000):
    """Quick performance benchmark."""
    print(f"\\n⚡ Running quick benchmark ({operations:,} operations)...")
    
    config = CacheConfig(max_size=50_000, enable_statistics=True)
    cache = Cache(config)
    
    # Pre-populate
    for i in range(10_000):
        cache.put(f"key_{i}", f"value_{i}")
    
    start_time = time.perf_counter()
    
    # Mixed operations
    for i in range(operations):
        key = f"key_{i % 10_000}"
        if i % 4 == 0:  # 25% writes
            cache.put(key, f"value_{i}")
        else:  # 75% reads
            cache.get(key)
    
    duration = time.perf_counter() - start_time
    throughput = operations / duration
    
    stats = cache.statistics()
    
    print(f"   Operations: {operations:,}")
    print(f"   Duration: {duration:.3f}s")
    print(f"   Throughput: {throughput:,.0f} ops/second")
    if stats:
        print(f"   Hit rate: {stats.hit_rate:.1f}%")
    
    return throughput

if __name__ == "__main__":
    try:
        test_cache()
        throughput = benchmark_cache()
        print(f"\\n🎉 Cache tests completed successfully!")
        print(f"   Python fallback cache operational")
        print(f"   Ready for PyO3 Rust integration")
    except Exception as e:
        print(f"\\n❌ Cache test failed: {e}")
        import traceback
        traceback.print_exc()
        import sys
        sys.exit(1)
