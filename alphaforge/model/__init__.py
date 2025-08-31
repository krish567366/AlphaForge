# AlphaForge Model Module
"""
Domain model and data structures for AlphaForge trading system.
"""

from alphaforge.model.identifiers import (
    InstrumentId,
    AccountId, 
    ClientOrderId,
    VenueOrderId,
    TradeId,
    PositionId,
    StrategyId,
    TraderId,
)

from alphaforge.model.enums import (
    OrderSide,
    OrderType,
    OrderStatus,
    TimeInForce,
    InstrumentClass,
    AggressorSide,
)

from alphaforge.model.data import (
    TradeTick,
    QuoteTick,
    Bar,
    OrderBookData,
)

from alphaforge.model.orders import (
    Order,
    MarketOrder,
    LimitOrder,
    StopOrder,
)

from alphaforge.model.events import (
    OrderEvent,
    TradeEvent,
    DataEvent,
)

# Import Rust components when available  
try:
    from alphaforge_pyo3.model import (
        Price,
        Quantity,
        OrderBook,
    )
    RUST_MODEL_AVAILABLE = True
except ImportError:
    RUST_MODEL_AVAILABLE = False
    # Use Python fallback implementations
    from alphaforge.model.fallback import Price, Quantity, OrderBook

__all__ = [
    # Identifiers
    "InstrumentId",
    "AccountId",
    "ClientOrderId", 
    "VenueOrderId",
    "TradeId",
    "PositionId",
    "StrategyId",
    "TraderId",
    
    # Enums
    "OrderSide",
    "OrderType", 
    "OrderStatus",
    "TimeInForce",
    "InstrumentClass",
    "AggressorSide",
    
    # Data types
    "TradeTick",
    "QuoteTick",
    "Bar",
    "OrderBookData",
    
    # Orders
    "Order",
    "MarketOrder",
    "LimitOrder", 
    "StopOrder",
    
    # Events
    "OrderEvent",
    "TradeEvent",
    "DataEvent",
    
    # High-performance types (Rust or fallback)
    "Price",
    "Quantity", 
    "OrderBook",
    
    # Flags
    "RUST_MODEL_AVAILABLE",
]
