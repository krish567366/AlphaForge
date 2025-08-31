"""
AlphaForge Core Module

High-performance core components implemented in Rust with Python bindings.
"""

# Try to import Rust extensions
try:
    from .rust import *
    RUST_EXTENSIONS_AVAILABLE = True
except ImportError as e:
    RUST_EXTENSIONS_AVAILABLE = False
    import warnings
    warnings.warn(
        f"Rust extensions not available: {e}. "
        "Install alphaforge with Rust support for optimal performance.",
        ImportWarning
    )

__all__ = ["RUST_EXTENSIONS_AVAILABLE"]
