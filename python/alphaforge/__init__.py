"""
AlphaForge - High-Performance Algorithmic Trading System

A modern, high-performance algorithmic trading system built with Rust for speed
and Python for flexibility. Provides sub-microsecond latency data processing,
advanced caching, and comprehensive trading infrastructure.
"""

__version__ = "1.0.0"
__author__ = "AlphaForge Team"
__email__ = "team@alphaforge.dev"
__license__ = "MIT"

# Import core functionality
from .core import *

# Version info
VERSION = (1, 0, 0)

def get_version():
    """Get the current version as a string."""
    return __version__

def get_version_info():
    """Get detailed version information."""
    return {
        "version": __version__,
        "author": __author__,
        "license": __license__,
        "python_version": VERSION,
    }

# Import submodules for convenience
try:
    from .core.rust import *
    _RUST_AVAILABLE = True
except ImportError:
    _RUST_AVAILABLE = False

def is_rust_available():
    """Check if Rust extensions are available."""
    return _RUST_AVAILABLE

# Performance info
def get_build_info():
    """Get build information including Rust extension status."""
    info = {
        "rust_extensions": is_rust_available(),
        "version": __version__,
    }
    
    if is_rust_available():
        try:
            from .core.rust import ALPHAFORGE_USER_AGENT
            info["user_agent"] = ALPHAFORGE_USER_AGENT
        except ImportError:
            pass
    
    return info

# Convenience imports
__all__ = [
    "__version__",
    "get_version",
    "get_version_info", 
    "get_build_info",
    "is_rust_available",
]
