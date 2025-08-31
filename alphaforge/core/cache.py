# AlphaForge Cache Module - Python Fallback Implementation
"""
Python fallback implementation for cache system when Rust components are not available.
This provides the same API but with reduced performance compared to the Rust implementation.
"""

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
    Python fallback cache implementation.
    
    Provides the same API as the Rust cache but with reduced performance.
    Uses OrderedDict for LRU eviction and threading.RLock for thread safety.
    """
    
    def __init__(self, config: CacheConfig):
        self.config = config
        self._lock = RLock()
        self._data: OrderedDict[str, CacheEntry] = OrderedDict()
        self._stats = CacheStatistics() if config.enable_statistics else None
        
        # Load from persistence if enabled
        if config.enable_persistence and config.persistence_path:
            self._load_from_disk()
    
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
            # Clean expired entries first
            current_time = time.time()
            expired_keys = []
            
            for key, entry in self._data.items():
                if entry.is_expired():
                    expired_keys.append(key)
            
            for key in expired_keys:
                self._data.pop(key, None)
                if self._stats:
                    self._stats.evictions += 1
            
            return list(self._data.keys())
    
    def values(self) -> List[T]:
        """Get all values in cache."""
        with self._lock:
            # Clean expired entries first
            current_time = time.time()
            expired_keys = []
            
            for key, entry in self._data.items():
                if entry.is_expired():
                    expired_keys.append(key)
            
            for key in expired_keys:
                self._data.pop(key, None)
                if self._stats:
                    self._stats.evictions += 1
            
            return [entry.value for entry in self._data.values()]
    
    def statistics(self) -> Optional[CacheStatistics]:
        """Get cache statistics."""
        if not self._stats:
            return None
        
        with self._lock:
            # Update memory usage estimate
            if self._stats:
                self._stats.memory_usage = len(self._data) * 64  # Rough estimate
            
            return self._stats
    
    def reset_statistics(self) -> None:
        """Reset cache statistics."""
        if self._stats:
            with self._lock:
                self._stats = CacheStatistics()
    
    def _load_from_disk(self) -> None:
        """Load cache from disk if persistence is enabled."""
        if not self.config.persistence_path:
            return
        
        try:
            with open(self.config.persistence_path, 'r') as f:
                data = json.load(f)
                
            for key, value in data.items():
                self.put(key, value)
                
        except (FileNotFoundError, json.JSONDecodeError):
            # File doesn't exist or is corrupted, start fresh
            pass
    
    def save_to_disk(self) -> bool:
        """Save cache to disk if persistence is enabled."""
        if not self.config.persistence_path:
            return False
        
        try:
            with self._lock:
                # Only save non-expired entries
                data = {}
                current_time = time.time()
                
                for key, entry in self._data.items():
                    if not entry.is_expired():
                        data[key] = entry.value
            
            with open(self.config.persistence_path, 'w') as f:
                json.dump(data, f, indent=2)
                
            return True
            
        except Exception:
            return False
    
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
