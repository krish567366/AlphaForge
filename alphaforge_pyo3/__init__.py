# AlphaForge PyO3 Extension Module
"""
PyO3 bindings for AlphaForge Rust components.
This module provides Python bindings for high-performance Rust implementations.
"""

# Import the actual compiled Rust extension
try:
    # Import from the compiled .pyd file submodules
    # Need to use a different name to avoid circular import
    import alphaforge_pyo3.alphaforge_pyo3 as _rust_ext
    Cache = _rust_ext.cache.Cache
    CacheConfig = _rust_ext.cache.CacheConfig
    CacheStatistics = _rust_ext.cache.CacheStatistics
    unix_nanos_now = _rust_ext.core.unix_nanos_now_py
    uuid4_new = _rust_ext.core.uuid4_new_py
    
    # Re-export main components for convenience
    __all__ = [
        'unix_nanos_now',
        'uuid4_new', 
        'Cache',
        'CacheConfig',
        'CacheStatistics',
    ]
    
except ImportError as e:
    # Fallback to Python stubs if Rust extension not available
    import warnings
    warnings.warn(f"Failed to import Rust extension, using Python fallback: {e}")
    
    import time
    from typing import Dict, List, Optional, Any, TypeVar, Generic

    T = TypeVar('T')

    def unix_nanos_now() -> int:
        """
        Get current Unix timestamp in nanoseconds.
        
        This is a fallback Python implementation.
        """
        return int(time.time_ns())

    def uuid4_new() -> str:
        """
        Generate a new UUID4 string.
        
        This is a fallback Python implementation.
        """
        import uuid
        return str(uuid.uuid4())

    class CacheStatistics:
        """
        Cache performance statistics.
        """
        
        def __init__(self):
            self.hits = 0
            self.misses = 0
            self.inserts = 0
            self.evictions = 0
            self.memory_usage = 0
        
        @property
        def hit_rate(self) -> float:
            """Calculate hit rate as percentage."""
            total = self.hits + self.misses
            if total == 0:
                return 0.0
            return (self.hits / total) * 100.0

    class CacheConfig:
        """
        Cache configuration.
        """
        
        def __init__(
            self,
            max_size: int = 10_000,
            ttl_seconds: Optional[int] = None,
            enable_statistics: bool = True,
        ):
            self.max_size = max_size
            self.ttl_seconds = ttl_seconds
            self.enable_statistics = enable_statistics

    class Cache(Generic[T]):
        """
        Fallback Python cache implementation.
        """
        
        def __init__(self, config: CacheConfig):
            """Initialize cache with configuration."""
            self.config = config
            self._data = {}
            self._stats = CacheStatistics()
        
        def get(self, key: str) -> Optional[T]:
            """Get value from cache."""
            if key in self._data:
                self._stats.hits += 1
                return self._data[key]
            else:
                self._stats.misses += 1
                return None
        
        def put(self, key: str, value: T) -> bool:
            """Put value into cache."""
            self._data[key] = value
            self._stats.inserts += 1
            return True
        
        def contains(self, key: str) -> bool:
            """Check if key exists in cache."""
            return key in self._data
        
        def remove(self, key: str) -> bool:
            """Remove key from cache."""
            if key in self._data:
                del self._data[key]
                return True
            return False
        
        def clear(self) -> None:
            """Clear all entries from cache."""
            self._data.clear()
        
        def size(self) -> int:
            """Get current cache size."""
            return len(self._data)
        
        def keys(self) -> List[str]:
            """Get all keys in cache."""
            return list(self._data.keys())
        
        def statistics(self) -> Optional[CacheStatistics]:
            """Get cache statistics."""
            return self._stats if self.config.enable_statistics else None
        
        def reset_statistics(self) -> None:
            """Reset cache statistics."""
            self._stats = CacheStatistics()

    __all__ = [
        'unix_nanos_now',
        'uuid4_new', 
        'Cache',
        'CacheConfig',
        'CacheStatistics',
    ]
