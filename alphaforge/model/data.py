# AlphaForge Data Types
"""
Core data types for AlphaForge trading system.
"""

from typing import Optional, Dict, Any, List
from dataclasses import dataclass, field
from alphaforge.model.identifiers import InstrumentId, TradeId
from alphaforge.model.enums import AggressorSide, OrderBookAction, BarAggregation
from alphaforge.core.time import UnixNanos


# Try importing the Rust extensions first, fallback to Python implementations
try:
    from alphaforge_pyo3 import Price, Quantity  # type: ignore
except ImportError:
    # Python fallback implementations - will be defined at end of file
    pass


@dataclass(frozen=True)
class Quote:
    """Quote data with bid/ask prices and sizes."""
    instrument_id: InstrumentId
    bid_price: "Price"
    ask_price: "Price" 
    bid_size: "Quantity"
    ask_size: "Quantity"
    ts_event: UnixNanos
    ts_init: UnixNanos = field(default_factory=UnixNanos.now)


@dataclass(frozen=True)
class Trade:
    """Trade tick data."""
    instrument_id: InstrumentId
    price: "Price"
    size: "Quantity" 
    aggressor_side: AggressorSide
    trade_id: TradeId
    ts_event: UnixNanos
    ts_init: UnixNanos = field(default_factory=UnixNanos.now)


@dataclass(frozen=True)
class OrderBookData:
    """Order book update data."""
    instrument_id: InstrumentId
    action: OrderBookAction
    order: Optional["BookOrder"] = None
    flags: int = 0
    sequence: int = 0
    ts_event: UnixNanos = field(default_factory=UnixNanos.now)
    ts_init: UnixNanos = field(default_factory=UnixNanos.now)


@dataclass(frozen=True)
class BookOrder:
    """Order book level entry."""
    side: int  # 1=bid, 2=ask
    price: "Price"
    size: "Quantity"
    order_id: int = 0


@dataclass(frozen=True)
class OrderBookSnapshot:
    """Full order book snapshot."""
    instrument_id: InstrumentId
    bids: List[BookOrder]
    asks: List[BookOrder]
    ts_event: UnixNanos
    ts_init: UnixNanos = field(default_factory=UnixNanos.now)


@dataclass(frozen=True)
class Bar:
    """OHLCV bar data."""
    open: "Price"
    high: "Price"
    low: "Price"
    close: "Price"
    volume: "Quantity"
    ts_event: UnixNanos
    ts_init: UnixNanos = field(default_factory=UnixNanos.now)


@dataclass(frozen=True)
class Instrument:
    """Base instrument definition."""
    id: InstrumentId
    native_symbol: str
    asset_class: int
    instrument_class: int
    quote_currency: str
    base_currency: Optional[str] = None
    price_precision: int = 5
    size_precision: int = 0
    price_increment: Optional["Price"] = None
    size_increment: Optional["Quantity"] = None
    multiplier: Optional["Quantity"] = None
    lot_size: Optional["Quantity"] = None
    max_quantity: Optional["Quantity"] = None
    min_quantity: Optional["Quantity"] = None
    max_price: Optional["Price"] = None
    min_price: Optional["Price"] = None
    margin_init: Optional["Quantity"] = None
    margin_maint: Optional["Quantity"] = None
    maker_fee: Optional["Quantity"] = None
    taker_fee: Optional["Quantity"] = None
    ts_event: UnixNanos = field(default_factory=UnixNanos.now)
    ts_init: UnixNanos = field(default_factory=UnixNanos.now)
    info: Dict[str, Any] = field(default_factory=dict)


@dataclass(frozen=True)
class Currency:
    """Currency definition."""
    code: str
    precision: int
    iso4217: int
    name: str
    currency_type: int


@dataclass(frozen=True)
class Account:
    """Trading account information."""
    account_id: str
    account_type: int
    base_currency: Currency
    is_cash_account: bool
    calculate_account_state: bool
    info: Dict[str, Any] = field(default_factory=dict)


@dataclass
class AccountBalance:
    """Account balance for a specific currency."""
    total: "Quantity"
    locked: "Quantity"
    free: "Quantity"


@dataclass
class MarginBalance:
    """Margin account balance information."""
    initial: "Quantity"
    maintenance: "Quantity"
    instrument_id: Optional[InstrumentId] = None


@dataclass(frozen=True)
class Tick:
    """Generic price tick."""
    instrument_id: InstrumentId
    price: "Price"
    size: "Quantity"
    ts_event: UnixNanos
    ts_init: UnixNanos = field(default_factory=UnixNanos.now)
@dataclass(frozen=True)
class TradeTick:
    """Trade execution tick."""
    instrument_id: InstrumentId
    price: "Price"
    size: "Quantity"
    ts_event: UnixNanos
    aggressor_side: AggressorSide
    trade_id: TradeId
    ts_init: UnixNanos = field(default_factory=UnixNanos.now)


@dataclass(frozen=True)
class QuoteTick:
    """Best bid/ask quote tick."""
    instrument_id: InstrumentId
    bid_price: "Price"
    ask_price: "Price"
    bid_size: "Quantity"
    ask_size: "Quantity"
    ts_event: UnixNanos
    ts_init: UnixNanos = field(default_factory=UnixNanos.now)


@dataclass(frozen=True)
class OrderBookLevel:
    """Order book price level."""
    price: "Price"
    size: "Quantity"
    count: int = 1


@dataclass(frozen=True)
class OrderBookDelta:
    """Order book incremental update."""
    instrument_id: InstrumentId
    action: OrderBookAction
    side: int
    level: OrderBookLevel
    ts_event: UnixNanos
    ts_init: UnixNanos = field(default_factory=UnixNanos.now)


@dataclass(frozen=True)
class Venue:
    """Trading venue information."""
    name: str
    venue_type: int
    oms_type: int
    account_type: int
    base_currency: Optional[Currency] = None
    starting_balances: List[AccountBalance] = field(default_factory=list)
    book_type: int = 2  # L2_MBP default
    routing: bool = False
    frozen_account: bool = False
    info: Dict[str, Any] = field(default_factory=dict)


# Python fallback implementations if Rust extensions not available
if 'Price' not in globals():
    @dataclass(frozen=True, order=True)
    class Price:
        """Price with fixed precision arithmetic (Python fallback)."""
        value: int  # Internal representation in smallest units
        precision: int = 5
        
        def __post_init__(self):
            if self.precision < 0 or self.precision > 9:
                raise ValueError("Precision must be between 0 and 9")
        
        @classmethod
        def from_str(cls, value_str: str, precision: int = 5) -> "Price":
            """Create price from string representation."""
            try:
                # Handle decimal point
                if '.' in value_str:
                    integer_part, decimal_part = value_str.split('.')
                    decimal_part = decimal_part.ljust(precision, '0')[:precision]
                    raw_value = int(integer_part) * (10 ** precision) + int(decimal_part)
                else:
                    raw_value = int(value_str) * (10 ** precision)
                
                return cls(raw_value, precision)
            except (ValueError, TypeError) as e:
                raise ValueError(f"Invalid price format: {value_str}") from e
        
        @classmethod 
        def from_int(cls, value: int, precision: int = 5) -> "Price":
            """Create price from integer (in smallest units)."""
            return cls(value, precision)
        
        @classmethod
        def from_float(cls, value: float, precision: int = 5) -> "Price":
            """Create price from float."""
            if not isinstance(value, (int, float)):
                raise TypeError("Value must be numeric")
            raw_value = int(round(value * (10 ** precision)))
            return cls(raw_value, precision)
        
        def as_double(self) -> float:
            """Convert to float representation."""
            return self.value / (10 ** self.precision)
        
        def as_str(self) -> str:
            """Convert to string representation."""
            divisor = 10 ** self.precision
            integer_part = self.value // divisor
            decimal_part = abs(self.value) % divisor
            
            if self.precision == 0:
                return str(integer_part)
            else:
                return f"{integer_part}.{decimal_part:0{self.precision}d}"
        
        def __str__(self) -> str:
            return self.as_str()
        
        def __repr__(self) -> str:
            return f"Price('{self.as_str()}')"
        
        def __add__(self, other: "Price") -> "Price":
            if self.precision != other.precision:
                raise ValueError("Cannot add prices with different precision")
            return Price(self.value + other.value, self.precision)
        
        def __sub__(self, other: "Price") -> "Price":
            if self.precision != other.precision:
                raise ValueError("Cannot subtract prices with different precision") 
            return Price(self.value - other.value, self.precision)
        
        def __mul__(self, other: "Quantity") -> "Price":
            return Price(self.value * other.value, self.precision)
        
        def __truediv__(self, other: "Quantity") -> "Price":
            if other.value == 0:
                raise ZeroDivisionError("Cannot divide by zero")
            return Price(self.value // other.value, self.precision)


if 'Quantity' not in globals():
    @dataclass(frozen=True, order=True)
    class Quantity:
        """Quantity with fixed precision arithmetic (Python fallback)."""
        value: int  # Internal representation in smallest units
        precision: int = 0
        
        def __post_init__(self):
            if self.precision < 0 or self.precision > 9:
                raise ValueError("Precision must be between 0 and 9")
        
        @classmethod
        def from_str(cls, value_str: str, precision: int = 0) -> "Quantity":
            """Create quantity from string representation."""
            try:
                if '.' in value_str:
                    integer_part, decimal_part = value_str.split('.')
                    decimal_part = decimal_part.ljust(precision, '0')[:precision]
                    raw_value = int(integer_part) * (10 ** precision) + int(decimal_part)
                else:
                    raw_value = int(value_str) * (10 ** precision)
                
                return cls(raw_value, precision)
            except (ValueError, TypeError) as e:
                raise ValueError(f"Invalid quantity format: {value_str}") from e
        
        @classmethod
        def from_int(cls, value: int, precision: int = 0) -> "Quantity":
            """Create quantity from integer."""
            return cls(value, precision)
        
        @classmethod 
        def from_float(cls, value: float, precision: int = 0) -> "Quantity":
            """Create quantity from float."""
            raw_value = int(round(value * (10 ** precision)))
            return cls(raw_value, precision)
        
        def as_double(self) -> float:
            """Convert to float representation.""" 
            return self.value / (10 ** self.precision)
        
        def as_str(self) -> str:
            """Convert to string representation."""
            divisor = 10 ** self.precision
            integer_part = self.value // divisor
            decimal_part = abs(self.value) % divisor
            
            if self.precision == 0:
                return str(integer_part)
            else:
                return f"{integer_part}.{decimal_part:0{self.precision}d}"
        
        def __str__(self) -> str:
            return self.as_str()
        
        def __repr__(self) -> str:
            return f"Quantity('{self.as_str()}')"
        
        def __add__(self, other: "Quantity") -> "Quantity":
            if self.precision != other.precision:
                raise ValueError("Cannot add quantities with different precision")
            return Quantity(self.value + other.value, self.precision)
        
        def __sub__(self, other: "Quantity") -> "Quantity":
            if self.precision != other.precision:
                raise ValueError("Cannot subtract quantities with different precision")
            return Quantity(self.value - other.value, self.precision)
        
        def __mul__(self, other: "Quantity") -> "Quantity":
            return Quantity(self.value * other.value, max(self.precision, other.precision))
        
        def __truediv__(self, other: "Quantity") -> "Quantity":
            if other.value == 0:
                raise ZeroDivisionError("Cannot divide by zero")
            return Quantity(self.value // other.value, max(self.precision, other.precision))
