# AlphaForge Logging System
"""
High-performance logging system with Rust integration.
"""

import logging
import sys
from pathlib import Path
from typing import Optional
from enum import Enum

try:
    # Try to use Rust logging if available
    import alphaforge_pyo3
    RUST_LOGGING_AVAILABLE = True
except ImportError:
    RUST_LOGGING_AVAILABLE = False


class LogColor(Enum):
    """Log color enumeration for console output."""
    NORMAL = "NORMAL"
    GREEN = "GREEN" 
    BLUE = "BLUE"
    YELLOW = "YELLOW"
    RED = "RED"
    MAGENTA = "MAGENTA"
    CYAN = "CYAN"


class AlphaForgeLogger:
    """
    High-performance logger with optional Rust backend.
    """
    
    def __init__(self, name: str, level: str = "INFO"):
        self.name = name
        self._python_logger = logging.getLogger(name)
        self._python_logger.setLevel(getattr(logging, level))
        
        # Set up console handler if not already configured
        if not self._python_logger.handlers:
            handler = logging.StreamHandler(sys.stdout)
            formatter = logging.Formatter(
                '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
            )
            handler.setFormatter(formatter)
            self._python_logger.addHandler(handler)
    
    def debug(self, message: str, color: LogColor = LogColor.NORMAL) -> None:
        """Log DEBUG level message."""
        if RUST_LOGGING_AVAILABLE:
            # Use Rust logging for performance
            try:
                alphaforge_pyo3.log_debug(self.name, message, color.value)
                return
            except AttributeError:
                pass  # Fallback to Python logging
        
        self._python_logger.debug(message)
    
    def info(self, message: str, color: LogColor = LogColor.NORMAL) -> None:
        """Log INFO level message."""
        if RUST_LOGGING_AVAILABLE:
            try:
                alphaforge_pyo3.log_info(self.name, message, color.value)
                return
            except AttributeError:
                pass
        
        self._python_logger.info(message)
    
    def warning(self, message: str, color: LogColor = LogColor.YELLOW) -> None:
        """Log WARNING level message."""
        if RUST_LOGGING_AVAILABLE:
            try:
                alphaforge_pyo3.log_warning(self.name, message, color.value)
                return
            except AttributeError:
                pass
        
        self._python_logger.warning(message)
    
    def error(self, message: str, color: LogColor = LogColor.RED) -> None:
        """Log ERROR level message."""
        if RUST_LOGGING_AVAILABLE:
            try:
                alphaforge_pyo3.log_error(self.name, message, color.value)
                return
            except AttributeError:
                pass
        
        self._python_logger.error(message)
    
    def critical(self, message: str, color: LogColor = LogColor.RED) -> None:
        """Log CRITICAL level message."""
        if RUST_LOGGING_AVAILABLE:
            try:
                alphaforge_pyo3.log_critical(self.name, message, color.value)
                return
            except AttributeError:
                pass
        
        self._python_logger.critical(message)


# Global logger cache
_loggers: dict[str, AlphaForgeLogger] = {}


def get_logger(name: str, level: str = "INFO") -> AlphaForgeLogger:
    """
    Get or create a logger instance.
    
    Args:
        name: Logger name
        level: Log level (DEBUG, INFO, WARNING, ERROR, CRITICAL)
        
    Returns:
        AlphaForgeLogger instance
    """
    if name not in _loggers:
        _loggers[name] = AlphaForgeLogger(name, level)
    return _loggers[name]


def configure_logging(
    level: str = "INFO",
    log_file: Optional[Path] = None,
    enable_rust_logging: bool = True,
) -> None:
    """
    Configure global logging settings.
    
    Args:
        level: Global log level
        log_file: Optional file path for log output
        enable_rust_logging: Whether to use Rust logging backend
    """
    # Configure Python logging
    logging.basicConfig(
        level=getattr(logging, level),
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
        handlers=[
            logging.StreamHandler(sys.stdout),
        ] + ([logging.FileHandler(log_file)] if log_file else [])
    )
    
    # Configure Rust logging if available
    if enable_rust_logging and RUST_LOGGING_AVAILABLE:
        try:
            alphaforge_pyo3.configure_logging(level)
        except AttributeError:
            logging.warning("Rust logging configuration not available")


# Default logger for the module
logger = get_logger(__name__)
