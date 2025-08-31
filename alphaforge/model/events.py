# AlphaForge Events
"""
Event types for AlphaForge trading system.
"""

from typing import Optional, Dict, Any, List
from dataclasses import dataclass, field
from abc import ABC, abstractmethod
from alphaforge.model.identifiers import (
    ClientOrderId, VenueOrderId, InstrumentId, StrategyId, 
    TraderId, AccountId, PositionId, TradeId
)
from alphaforge.model.enums import (
    OrderSide, OrderType, OrderStatus, TimeInForce,
    PositionSide, LiquiditySide, AggressorSide
)
from alphaforge.model.data import Price, Quantity
from alphaforge.core.time import UnixNanos


class Event(ABC):
    """Base class for all events."""
    
    @property
    @abstractmethod
    def event_type(self) -> str:
        """The event type identifier."""
        pass
    
    @property
    @abstractmethod
    def ts_event(self) -> UnixNanos:
        """Event timestamp."""
        pass
    
    @property
    @abstractmethod
    def ts_init(self) -> UnixNanos:
        """Event initialization timestamp."""
        pass


@dataclass(frozen=True)
class OrderEvent(Event):
    """Base class for order-related events."""
    
    client_order_id: ClientOrderId
    venue_order_id: Optional[VenueOrderId]
    account_id: AccountId
    instrument_id: InstrumentId
    strategy_id: StrategyId
    ts_event: UnixNanos
    ts_init: UnixNanos = field(default_factory=UnixNanos.now)
    
    @property
    def event_type(self) -> str:
        return self.__class__.__name__


@dataclass(frozen=True)
class OrderSubmitted(OrderEvent):
    """Event when order is submitted to venue."""
    pass


@dataclass(frozen=True)
class OrderAccepted(OrderEvent):
    """Event when order is accepted by venue."""
    pass


@dataclass(frozen=True)
class OrderRejected(OrderEvent):
    """Event when order is rejected by venue."""
    
    reason: str
    
    @property
    def event_type(self) -> str:
        return "OrderRejected"


@dataclass(frozen=True)
class OrderCanceled(OrderEvent):
    """Event when order is canceled."""
    pass


@dataclass(frozen=True)
class OrderExpired(OrderEvent):
    """Event when order expires."""
    pass


@dataclass(frozen=True)
class OrderTriggered(OrderEvent):
    """Event when contingent order is triggered."""
    pass


@dataclass(frozen=True)
class OrderPendingUpdate(OrderEvent):
    """Event when order update is pending."""
    pass


@dataclass(frozen=True)
class OrderPendingCancel(OrderEvent):
    """Event when order cancel is pending."""
    pass


@dataclass(frozen=True)
class OrderModifyRejected(OrderEvent):
    """Event when order modification is rejected."""
    
    reason: str


@dataclass(frozen=True)
class OrderCancelRejected(OrderEvent):
    """Event when order cancellation is rejected."""
    
    reason: str


@dataclass(frozen=True)
class OrderFilled(OrderEvent):
    """Event when order is filled (partially or fully)."""
    
    trade_id: TradeId
    order_side: OrderSide
    order_type: OrderType
    last_qty: Quantity
    last_px: Price
    leaves_qty: Quantity
    cum_qty: Quantity
    avg_px: Optional[Price]
    commission: Quantity
    commission_currency: str
    liquidity_side: LiquiditySide
    
    # Optional fields
    position_id: Optional[PositionId] = None
    venue_position_id: Optional[str] = None
    slippage: Optional[Price] = None
    info: Dict[str, Any] = field(default_factory=dict)


@dataclass(frozen=True)
class PositionEvent(Event):
    """Base class for position events."""
    
    position_id: PositionId
    account_id: AccountId
    instrument_id: InstrumentId
    strategy_id: StrategyId
    ts_event: UnixNanos
    ts_init: UnixNanos = field(default_factory=UnixNanos.now)
    
    @property
    def event_type(self) -> str:
        return self.__class__.__name__


@dataclass(frozen=True)
class PositionOpened(PositionEvent):
    """Event when position is opened."""
    
    side: PositionSide
    quantity: Quantity
    peak_qty: Quantity
    last_qty: Quantity
    last_px: Price
    currency: str
    avg_px_open: Price
    realized_return: Quantity
    realized_pnl: Quantity
    unrealized_pnl: Quantity
    ts_opened: UnixNanos
    duration_ns: int = 0


@dataclass(frozen=True)
class PositionChanged(PositionEvent):
    """Event when position quantity changes."""
    
    side: PositionSide
    quantity: Quantity
    peak_qty: Quantity
    last_qty: Quantity
    last_px: Price
    currency: str
    avg_px_open: Price
    realized_return: Quantity
    realized_pnl: Quantity
    unrealized_pnl: Quantity
    ts_opened: UnixNanos
    duration_ns: int


@dataclass(frozen=True)
class PositionClosed(PositionEvent):
    """Event when position is closed."""
    
    side: PositionSide
    quantity: Quantity
    peak_qty: Quantity
    last_qty: Quantity
    last_px: Price
    currency: str
    avg_px_open: Price
    avg_px_close: Price
    realized_return: Quantity
    realized_pnl: Quantity
    ts_opened: UnixNanos
    ts_closed: UnixNanos
    duration_ns: int


@dataclass(frozen=True)
class AccountEvent(Event):
    """Base class for account events."""
    
    account_id: AccountId
    currency: str
    ts_event: UnixNanos
    ts_init: UnixNanos = field(default_factory=UnixNanos.now)
    
    @property
    def event_type(self) -> str:
        return self.__class__.__name__


@dataclass(frozen=True)
class AccountState(AccountEvent):
    """Account state update event."""
    
    account_type: int
    balance_total: Dict[str, Quantity]
    balance_free: Dict[str, Quantity]
    balance_locked: Dict[str, Quantity]
    margin_init: Quantity
    margin_maint: Quantity
    reported: bool
    info: Dict[str, Any] = field(default_factory=dict)


@dataclass(frozen=True)
class MarketDataEvent(Event):
    """Base class for market data events."""
    
    instrument_id: InstrumentId
    ts_event: UnixNanos
    ts_init: UnixNanos = field(default_factory=UnixNanos.now)
    
    @property
    def event_type(self) -> str:
        return self.__class__.__name__


@dataclass(frozen=True)
class OrderBookEvent(MarketDataEvent):
    """Order book update event."""
    
    action: int  # ADD=1, UPDATE=2, DELETE=3, CLEAR=4
    side: int    # BID=1, ASK=2
    price: Price
    size: Quantity
    count: int = 1
    sequence: int = 0
    flags: int = 0


@dataclass(frozen=True)
class QuoteEvent(MarketDataEvent):
    """Quote tick event."""
    
    bid_price: Price
    ask_price: Price
    bid_size: Quantity
    ask_size: Quantity


@dataclass(frozen=True)
class TradeEvent(MarketDataEvent):
    """Trade tick event."""
    
    price: Price
    size: Quantity
    aggressor_side: AggressorSide
    trade_id: TradeId


@dataclass(frozen=True)
class BarEvent(MarketDataEvent):
    """Bar/candlestick event."""
    
    bar_type: str  # e.g., "1-MINUTE-BID"
    open: Price
    high: Price
    low: Price
    close: Price
    volume: Quantity
    is_revision: bool = False


@dataclass(frozen=True)
class InstrumentStatusEvent(MarketDataEvent):
    """Instrument status change event."""
    
    status: int
    halt_reason: Optional[str] = None
    trading_session: Optional[str] = None


@dataclass(frozen=True)
class VenueStatusEvent(Event):
    """Venue status change event."""
    
    venue: str
    status: int
    ts_event: UnixNanos
    ts_init: UnixNanos = field(default_factory=UnixNanos.now)
    
    @property
    def event_type(self) -> str:
        return "VenueStatusEvent"


@dataclass(frozen=True)
class InstrumentCloseEvent(Event):
    """Instrument close price event."""
    
    instrument_id: InstrumentId
    close_price: Price
    close_type: int  # DAILY=1, WEEKLY=2, MONTHLY=3
    ts_event: UnixNanos
    ts_init: UnixNanos = field(default_factory=UnixNanos.now)
    
    @property
    def event_type(self) -> str:
        return "InstrumentCloseEvent"


@dataclass
class EventStore:
    """Event storage and retrieval."""
    
    def __init__(self):
        self._events: List[Event] = []
        self._event_index: Dict[str, List[int]] = {}
    
    def add_event(self, event: Event) -> None:
        """Add an event to the store."""
        index = len(self._events)
        self._events.append(event)
        
        # Index by event type
        event_type = event.event_type
        if event_type not in self._event_index:
            self._event_index[event_type] = []
        self._event_index[event_type].append(index)
    
    def get_events(self, event_type: Optional[str] = None) -> List[Event]:
        """Get events by type, or all events if type is None."""
        if event_type is None:
            return self._events.copy()
        
        indices = self._event_index.get(event_type, [])
        return [self._events[i] for i in indices]
    
    def get_events_for_order(self, client_order_id: ClientOrderId) -> List[OrderEvent]:
        """Get all events for a specific order."""
        order_events = []
        for event in self._events:
            if (isinstance(event, OrderEvent) and 
                event.client_order_id == client_order_id):
                order_events.append(event)
        return order_events
    
    def get_events_for_position(self, position_id: PositionId) -> List[PositionEvent]:
        """Get all events for a specific position."""
        position_events = []
        for event in self._events:
            if (isinstance(event, PositionEvent) and 
                event.position_id == position_id):
                position_events.append(event)
        return position_events
    
    def get_events_for_instrument(self, instrument_id: InstrumentId) -> List[Event]:
        """Get all events for a specific instrument."""
        instrument_events = []
        for event in self._events:
            if hasattr(event, 'instrument_id') and event.instrument_id == instrument_id:
                instrument_events.append(event)
        return instrument_events
    
    def clear(self) -> None:
        """Clear all events."""
        self._events.clear()
        self._event_index.clear()
    
    @property
    def count(self) -> int:
        """Get total number of events."""
        return len(self._events)
