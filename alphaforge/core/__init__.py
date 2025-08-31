# AlphaForge Core Module
"""
Core utilities and foundational components for AlphaForge.
"""

from alphaforge.core.component import Component, ComponentState
from alphaforge.core.config import AlphaForgeConfig
from alphaforge.core.exceptions import AlphaForgeError, ValidationError
from alphaforge.core.logging import get_logger
from alphaforge.core.uuid import uuid4_new, uuid4_bytes, is_valid_uuid4
from alphaforge.core.time import UnixNanos, AtomicTime

# Import Rust components when available
try:
    from alphaforge_pyo3.core import (
        unix_nanos_now,
        uuid4_new as rust_uuid4_new,
        Cache,
        CacheConfig,
        CacheStatistics,
    )
    RUST_AVAILABLE = True
    # Override with Rust implementations if available
    uuid4_new = rust_uuid4_new
except ImportError:
    RUST_AVAILABLE = False
    # Using fallback implementations
    import time
    import uuid
    from alphaforge.core.cache import Cache, CacheConfig, CacheStatistics
    
    def unix_nanos_now() -> int:
        return int(time.time_ns())
    
    def uuid4_new() -> str:
        return str(uuid.uuid4())

__all__ = [
    "Component",
    "ComponentState", 
    "AlphaForgeConfig",
    "AlphaForgeError",
    "ValidationError",
    "get_logger",
    "unix_nanos_now",
    "uuid4_new",
    "Cache",
    "CacheConfig",
    "CacheStatistics",
    "RUST_AVAILABLE",
]
