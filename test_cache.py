#!/usr/bin/env python3
"""Simple cache test without full AlphaForge imports."""

import sys
import os

# Add path to make cache imports work
sys.path.insert(0, os.path.join(os.path.dirname(__file__)))

# Import the cache directly
from alphaforge.core.cache import Cache, CacheConfig, CacheStatistics

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

if __name__ == "__main__":
    try:
        test_cache()
        print("\n🎉 Cache test completed successfully!")
    except Exception as e:
        print(f"\n❌ Cache test failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
