# AlphaForge Exception Types
"""
Custom exception hierarchy for AlphaForge trading system.
"""

class AlphaForgeError(Exception):
    """Base exception for AlphaForge trading system."""
    pass

class ValidationError(AlphaForgeError):
    """Configuration or data validation error."""
    pass

class NetworkError(AlphaForgeError):
    """Network communication error."""
    pass

class SerializationError(AlphaForgeError):
    """Data serialization/deserialization error."""
    pass

class ComponentError(AlphaForgeError):
    """Component lifecycle or state error."""
    pass

class MessageBusError(AlphaForgeError):
    """Message bus operation error."""
    pass

class ExecutionError(AlphaForgeError):
    """Order execution error."""
    pass

class RiskError(AlphaForgeError):
    """Risk management error."""
    pass

class DataError(AlphaForgeError):
    """Market data processing error."""
    pass
