# AlphaForge - High-Performance Algorithmic Trading Platform
"""
AlphaForge is a high-performance algorithmic trading platform built with a hybrid 
Python/Rust architecture for institutional-grade trading applications.

Key Features:
- >1M messages/second throughput
- <10μs order book latency  
- Event-driven architecture
- Zero-copy operations
- Comprehensive backtesting and live trading
- Multi-venue support with unified APIs
"""

__version__ = "1.0.0"
__author__ = "AlphaForge Team"
__email__ = "team@alphaforge.io"
__license__ = "MIT"

# Core imports
from alphaforge.core import *
from alphaforge.model import *

# Version info
VERSION = __version__
ALPHAFORGE_VERSION = __version__

# Performance constants
TARGET_THROUGHPUT_MPS = 1_000_000  # 1M messages per second
TARGET_LATENCY_NANOS = 10_000      # 10 microseconds
