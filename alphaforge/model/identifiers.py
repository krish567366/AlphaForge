# AlphaForge Identifier Types
"""
Identifier types for AlphaForge trading system.
"""

from typing import Optional
from alphaforge.core.exceptions import ValidationError


class InstrumentId:
    """Instrument identifier with symbol and venue."""
    
    def __init__(self, identifier: str):
        """
        Create instrument ID from symbol.venue format.
        
        Args:
            identifier: String in format "SYMBOL.VENUE"
        """
        if not identifier or '.' not in identifier:
            raise ValidationError(f"Invalid instrument identifier format: {identifier}")
            
        parts = identifier.split('.')
        if len(parts) != 2:
            raise ValidationError(f"Invalid instrument identifier format: {identifier}")
            
        self._symbol = parts[0].upper()
        self._venue = parts[1].upper()
        self._value = f"{self._symbol}.{self._venue}"
        
        if not self._symbol or not self._venue:
            raise ValidationError(f"Empty symbol or venue in identifier: {identifier}")
    
    @property
    def symbol(self) -> str:
        """Get the symbol component."""
        return self._symbol
    
    @property 
    def venue(self) -> str:
        """Get the venue component."""
        return self._venue
    
    @property
    def value(self) -> str:
        """Get the full identifier string."""
        return self._value
    
    @classmethod
    def from_parts(cls, symbol: str, venue: str) -> "InstrumentId":
        """Create from separate symbol and venue."""
        return cls(f"{symbol}.{venue}")
    
    def __str__(self) -> str:
        return self._value
    
    def __repr__(self) -> str:
        return f"InstrumentId('{self._value}')"
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, InstrumentId):
            return False
        return self._value == other._value
    
    def __hash__(self) -> int:
        return hash(self._value)


class AccountId:
    """Account identifier with issuer and number."""
    
    def __init__(self, issuer: str, number: str):
        """
        Create account ID.
        
        Args:
            issuer: Account issuer (exchange/broker)
            number: Account number
        """
        if not issuer or not number:
            raise ValidationError("Issuer and number cannot be empty")
            
        self._issuer = issuer.upper()
        self._number = number
        self._value = f"{self._issuer}-{self._number}"
    
    @property
    def issuer(self) -> str:
        """Get the issuer component."""
        return self._issuer
    
    @property
    def number(self) -> str:
        """Get the account number."""
        return self._number
    
    @property
    def value(self) -> str:
        """Get the full identifier."""
        return self._value
    
    def __str__(self) -> str:
        return self._value
    
    def __repr__(self) -> str:
        return f"AccountId('{self._value}')"
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, AccountId):
            return False
        return self._value == other._value
    
    def __hash__(self) -> int:
        return hash(self._value)


class ClientOrderId:
    """Client order identifier."""
    
    def __init__(self, value: str):
        """
        Create client order ID.
        
        Args:
            value: Order identifier string
        """
        if not value:
            raise ValidationError("Client order ID cannot be empty")
        if len(value) > 64:
            raise ValidationError("Client order ID too long (max 64 characters)")
            
        self._value = value
    
    @property
    def value(self) -> str:
        """Get the identifier value."""
        return self._value
    
    @classmethod
    def generate(cls) -> "ClientOrderId":
        """Generate a new UUID-based client order ID."""
        from alphaforge.core import uuid4_new
        return cls(uuid4_new())
    
    def __str__(self) -> str:
        return self._value
    
    def __repr__(self) -> str:
        return f"ClientOrderId('{self._value}')"
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, ClientOrderId):
            return False
        return self._value == other._value
    
    def __hash__(self) -> int:
        return hash(self._value)


class VenueOrderId:
    """Venue order identifier."""
    
    def __init__(self, value: str):
        """Create venue order ID."""
        if not value:
            raise ValidationError("Venue order ID cannot be empty")
        self._value = value
    
    @property
    def value(self) -> str:
        """Get the identifier value.""" 
        return self._value
    
    def __str__(self) -> str:
        return self._value
    
    def __repr__(self) -> str:
        return f"VenueOrderId('{self._value}')"
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, VenueOrderId):
            return False
        return self._value == other._value
    
    def __hash__(self) -> int:
        return hash(self._value)


class TradeId:
    """Trade identifier."""
    
    def __init__(self, value: str):
        """Create trade ID."""
        if not value:
            raise ValidationError("Trade ID cannot be empty")
        self._value = value
    
    @property
    def value(self) -> str:
        """Get the identifier value."""
        return self._value
    
    def __str__(self) -> str:
        return self._value
    
    def __repr__(self) -> str:
        return f"TradeId('{self._value}')"
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, TradeId):
            return False
        return self._value == other._value
    
    def __hash__(self) -> int:
        return hash(self._value)


class PositionId:
    """Position identifier."""
    
    def __init__(self, value: str):
        """Create position ID."""
        if not value:
            raise ValidationError("Position ID cannot be empty")
        self._value = value
    
    @property
    def value(self) -> str:
        """Get the identifier value."""
        return self._value
    
    def __str__(self) -> str:
        return self._value
    
    def __repr__(self) -> str:
        return f"PositionId('{self._value}')"
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, PositionId):
            return False
        return self._value == other._value
    
    def __hash__(self) -> int:
        return hash(self._value)


class StrategyId:
    """Strategy identifier."""
    
    def __init__(self, value: str):
        """Create strategy ID."""
        if not value:
            raise ValidationError("Strategy ID cannot be empty")
        if len(value) > 64:
            raise ValidationError("Strategy ID too long (max 64 characters)")
        self._value = value
    
    @property
    def value(self) -> str:
        """Get the identifier value."""
        return self._value
    
    def __str__(self) -> str:
        return self._value
    
    def __repr__(self) -> str:
        return f"StrategyId('{self._value}')"
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, StrategyId):
            return False
        return self._value == other._value
    
    def __hash__(self) -> int:
        return hash(self._value)


class TraderId:
    """Trader identifier."""
    
    def __init__(self, value: str):
        """Create trader ID.""" 
        if not value:
            raise ValidationError("Trader ID cannot be empty")
        if len(value) > 64:
            raise ValidationError("Trader ID too long (max 64 characters)")
        self._value = value
    
    @property
    def value(self) -> str:
        """Get the identifier value."""
        return self._value
    
    def __str__(self) -> str:
        return self._value
    
    def __repr__(self) -> str:
        return f"TraderId('{self._value}')"
    
    def __eq__(self, other) -> bool:
        if not isinstance(other, TraderId):
            return False
        return self._value == other._value
    
    def __hash__(self) -> int:
        return hash(self._value)
