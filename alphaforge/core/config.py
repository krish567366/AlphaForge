# AlphaForge Configuration System
"""
Configuration management for AlphaForge trading system.
"""

import os
import json
from pathlib import Path
from typing import Dict, Any, Optional, Union
from dataclasses import dataclass, field
import logging

from alphaforge.core.exceptions import ValidationError


@dataclass
class AlphaForgeConfig:
    """
    Main configuration class for AlphaForge system.
    """
    
    # System configuration
    trader_id: str = "ALPHAFORGE-001"
    instance_id: str = "default"
    log_level: str = "INFO"
    
    # Performance configuration
    message_bus_capacity: int = 10000
    order_book_cache_size: int = 1000
    enable_high_precision: bool = True
    enable_simd_optimizations: bool = True
    
    # Network configuration
    websocket_heartbeat_interval_s: int = 30
    websocket_reconnect_max_attempts: int = 10
    websocket_reconnect_interval_s: int = 5
    
    # Risk management
    max_position_size: float = 1000000.0
    max_order_size: float = 100000.0
    daily_loss_limit: float = 50000.0
    
    # Data configuration
    cache_directory: str = "~/.alphaforge/cache"
    log_directory: str = "~/.alphaforge/logs"
    data_compression: bool = True
    
    # Exchange configurations
    exchanges: Dict[str, Dict[str, Any]] = field(default_factory=dict)
    
    def __post_init__(self):
        """Validate configuration after initialization."""
        self.validate()
        
        # Expand paths
        self.cache_directory = os.path.expanduser(self.cache_directory)
        self.log_directory = os.path.expanduser(self.log_directory)
        
        # Create directories if they don't exist
        Path(self.cache_directory).mkdir(parents=True, exist_ok=True)
        Path(self.log_directory).mkdir(parents=True, exist_ok=True)
    
    def validate(self) -> None:
        """Validate configuration parameters."""
        if not self.trader_id:
            raise ValidationError("trader_id cannot be empty")
            
        if not self.instance_id:
            raise ValidationError("instance_id cannot be empty")
            
        if self.log_level not in ("DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL"):
            raise ValidationError(f"Invalid log_level: {self.log_level}")
            
        if self.message_bus_capacity <= 0:
            raise ValidationError("message_bus_capacity must be positive")
            
        if self.max_position_size <= 0:
            raise ValidationError("max_position_size must be positive")
            
        if self.max_order_size <= 0:
            raise ValidationError("max_order_size must be positive")
            
        if self.daily_loss_limit <= 0:
            raise ValidationError("daily_loss_limit must be positive")
    
    @classmethod
    def from_file(cls, config_path: Union[str, Path]) -> "AlphaForgeConfig":
        """Load configuration from JSON file."""
        config_path = Path(config_path)
        
        if not config_path.exists():
            raise ValidationError(f"Configuration file not found: {config_path}")
            
        try:
            with open(config_path, 'r') as f:
                data = json.load(f)
            return cls(**data)
        except json.JSONDecodeError as e:
            raise ValidationError(f"Invalid JSON in configuration file: {e}")
        except TypeError as e:
            raise ValidationError(f"Invalid configuration format: {e}")
    
    @classmethod
    def from_env(cls) -> "AlphaForgeConfig":
        """Load configuration from environment variables."""
        config = {}
        
        # Map environment variables to config fields
        env_mapping = {
            "ALPHAFORGE_TRADER_ID": "trader_id",
            "ALPHAFORGE_INSTANCE_ID": "instance_id", 
            "ALPHAFORGE_LOG_LEVEL": "log_level",
            "ALPHAFORGE_MESSAGE_BUS_CAPACITY": ("message_bus_capacity", int),
            "ALPHAFORGE_ORDER_BOOK_CACHE_SIZE": ("order_book_cache_size", int),
            "ALPHAFORGE_ENABLE_HIGH_PRECISION": ("enable_high_precision", bool),
            "ALPHAFORGE_MAX_POSITION_SIZE": ("max_position_size", float),
            "ALPHAFORGE_MAX_ORDER_SIZE": ("max_order_size", float),
            "ALPHAFORGE_DAILY_LOSS_LIMIT": ("daily_loss_limit", float),
            "ALPHAFORGE_CACHE_DIRECTORY": "cache_directory",
            "ALPHAFORGE_LOG_DIRECTORY": "log_directory",
        }
        
        for env_var, field_info in env_mapping.items():
            value = os.getenv(env_var)
            if value is not None:
                if isinstance(field_info, tuple):
                    field_name, field_type = field_info
                    try:
                        if field_type == bool:
                            config[field_name] = value.lower() in ("true", "1", "yes", "on")
                        else:
                            config[field_name] = field_type(value)
                    except ValueError as e:
                        raise ValidationError(f"Invalid value for {env_var}: {value}")
                else:
                    config[field_info] = value
        
        return cls(**config)
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert configuration to dictionary."""
        return {
            "trader_id": self.trader_id,
            "instance_id": self.instance_id,
            "log_level": self.log_level,
            "message_bus_capacity": self.message_bus_capacity,
            "order_book_cache_size": self.order_book_cache_size,
            "enable_high_precision": self.enable_high_precision,
            "enable_simd_optimizations": self.enable_simd_optimizations,
            "websocket_heartbeat_interval_s": self.websocket_heartbeat_interval_s,
            "websocket_reconnect_max_attempts": self.websocket_reconnect_max_attempts,
            "websocket_reconnect_interval_s": self.websocket_reconnect_interval_s,
            "max_position_size": self.max_position_size,
            "max_order_size": self.max_order_size,
            "daily_loss_limit": self.daily_loss_limit,
            "cache_directory": self.cache_directory,
            "log_directory": self.log_directory,
            "data_compression": self.data_compression,
            "exchanges": self.exchanges,
        }
    
    def save_to_file(self, config_path: Union[str, Path]) -> None:
        """Save configuration to JSON file."""
        config_path = Path(config_path)
        config_path.parent.mkdir(parents=True, exist_ok=True)
        
        with open(config_path, 'w') as f:
            json.dump(self.to_dict(), f, indent=2)
    
    def add_exchange_config(self, name: str, config: Dict[str, Any]) -> None:
        """Add exchange-specific configuration."""
        self.exchanges[name] = config
    
    def get_exchange_config(self, name: str) -> Optional[Dict[str, Any]]:
        """Get exchange-specific configuration."""
        return self.exchanges.get(name)
    
    def __repr__(self) -> str:
        return f"AlphaForgeConfig(trader_id='{self.trader_id}', instance_id='{self.instance_id}')"
