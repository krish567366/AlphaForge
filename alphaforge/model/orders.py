# AlphaForge Orders
"""
Order types and order management for AlphaForge trading system.
"""

from typing import Optional, Dict, Any, List
from dataclasses import dataclass, field
from alphaforge.model.identifiers import (
    ClientOrderId, VenueOrderId, InstrumentId, StrategyId, 
    TraderId, AccountId, PositionId
)
from alphaforge.model.enums import (
    OrderSide, OrderType, OrderStatus, TimeInForce, 
    TriggerType, ContingencyType, LiquiditySide
)
from alphaforge.model.data import Price, Quantity
from alphaforge.core.time import UnixNanos


@dataclass
class Order:
    """Base order class."""
    
    # Required fields
    client_order_id: ClientOrderId
    strategy_id: StrategyId
    instrument_id: InstrumentId
    order_side: OrderSide
    order_type: OrderType
    quantity: Quantity
    
    # Optional fields with defaults
    trader_id: Optional[TraderId] = None
    account_id: Optional[AccountId] = None
    price: Optional[Price] = None
    trigger_price: Optional[Price] = None
    trigger_type: TriggerType = TriggerType.NO_TRIGGER
    limit_offset: Optional[Price] = None
    trailing_offset: Optional[Price] = None
    trailing_offset_type: int = 0  # 0=PRICE, 1=BASIS_POINTS
    time_in_force: TimeInForce = TimeInForce.GTC
    expire_time: Optional[UnixNanos] = None
    reduce_only: bool = False
    quote_quantity: bool = False
    display_qty: Optional[Quantity] = None
    
    # Order linking
    parent_order_id: Optional[ClientOrderId] = None
    contingency_type: ContingencyType = ContingencyType.NO_CONTINGENCY
    order_list_id: Optional[str] = None
    linked_order_ids: List[ClientOrderId] = field(default_factory=list)
    
    # Execution tracking
    venue_order_id: Optional[VenueOrderId] = None
    position_id: Optional[PositionId] = None
    exec_algorithm_id: Optional[str] = None
    exec_algorithm_params: Dict[str, Any] = field(default_factory=dict)
    exec_spawn_id: Optional[ClientOrderId] = None
    
    # Status and fills
    status: OrderStatus = OrderStatus.INITIALIZED
    filled_qty: Quantity = field(default_factory=lambda: Quantity.from_int(0))
    leaves_qty: Optional[Quantity] = None
    avg_px: Optional[Price] = None
    slippage: Optional[Price] = None
    
    # Risk management
    emulation_trigger: int = 0  # 0=NO_TRIGGER, 1=BID_ASK, 2=LAST, etc
    trigger_instrument_id: Optional[InstrumentId] = None
    
    # Timestamps
    init_id: str = ""
    ts_init: UnixNanos = field(default_factory=UnixNanos.now)
    ts_last: UnixNanos = field(default_factory=UnixNanos.now)
    
    # Tags and metadata
    tags: List[str] = field(default_factory=list)
    info: Dict[str, Any] = field(default_factory=dict)
    
    def __post_init__(self):
        """Initialize calculated fields after object creation."""
        if self.leaves_qty is None:
            self.leaves_qty = self.quantity - self.filled_qty
        
        # Generate init_id if not provided
        if not self.init_id:
            from alphaforge.core import uuid4_new
            self.init_id = uuid4_new()
    
    @property
    def is_buy(self) -> bool:
        """Check if this is a buy order."""
        return self.order_side == OrderSide.BUY
    
    @property
    def is_sell(self) -> bool:
        """Check if this is a sell order."""
        return self.order_side == OrderSide.SELL
    
    @property
    def is_passive(self) -> bool:
        """Check if this is a passive order type."""
        return self.order_type in (OrderType.LIMIT, OrderType.STOP_LIMIT, 
                                  OrderType.LIMIT_IF_TOUCHED)
    
    @property
    def is_aggressive(self) -> bool:
        """Check if this is an aggressive order type."""
        return self.order_type in (OrderType.MARKET, OrderType.STOP,
                                  OrderType.MARKET_TO_LIMIT, OrderType.MARKET_IF_TOUCHED)
    
    @property
    def is_contingent(self) -> bool:
        """Check if this order has contingent triggers."""
        return (self.trigger_type != TriggerType.NO_TRIGGER or
                self.contingency_type != ContingencyType.NO_CONTINGENCY)
    
    @property
    def is_parent_order(self) -> bool:
        """Check if this is a parent order."""
        return bool(self.linked_order_ids)
    
    @property
    def is_child_order(self) -> bool:
        """Check if this is a child order."""
        return self.parent_order_id is not None
    
    @property
    def is_open(self) -> bool:
        """Check if order is in an open state."""
        return self.status in (OrderStatus.ACCEPTED, OrderStatus.PENDING_NEW,
                              OrderStatus.PENDING_UPDATE, OrderStatus.PENDING_CANCEL,
                              OrderStatus.PARTIALLY_FILLED, OrderStatus.TRIGGERED)
    
    @property
    def is_closed(self) -> bool:
        """Check if order is in a closed state."""
        return self.status in (OrderStatus.FILLED, OrderStatus.CANCELED,
                              OrderStatus.REJECTED, OrderStatus.EXPIRED)
    
    @property
    def is_inflight(self) -> bool:
        """Check if order is currently in flight (pending operations)."""
        return self.status in (OrderStatus.PENDING_NEW, OrderStatus.PENDING_UPDATE,
                              OrderStatus.PENDING_CANCEL)
    
    @property
    def is_working(self) -> bool:
        """Check if order is working in the market."""
        return self.status in (OrderStatus.ACCEPTED, OrderStatus.TRIGGERED,
                              OrderStatus.PARTIALLY_FILLED)
    
    @property
    def would_reduce_only(self) -> bool:
        """Check if order would only reduce position."""
        return self.reduce_only
    
    def apply_fill(self, fill_qty: Quantity, fill_price: Price) -> None:
        """Apply a fill to this order."""
        if fill_qty.value <= 0:
            raise ValueError("Fill quantity must be positive")
        
        if self.filled_qty + fill_qty > self.quantity:
            raise ValueError("Fill quantity exceeds order quantity")
        
        self.filled_qty += fill_qty
        self.leaves_qty = self.quantity - self.filled_qty
        
        # Update average price
        if self.avg_px is None:
            self.avg_px = fill_price
        else:
            # Weighted average
            total_filled = self.filled_qty.value
            prev_filled = total_filled - fill_qty.value
            if total_filled > 0:
                weighted_avg = ((self.avg_px.value * prev_filled) + 
                               (fill_price.value * fill_qty.value)) / total_filled
                self.avg_px = Price.from_int(int(weighted_avg), self.avg_px.precision)
        
        # Update status
        if self.leaves_qty.value == 0:
            self.status = OrderStatus.FILLED
        elif self.filled_qty.value > 0:
            self.status = OrderStatus.PARTIALLY_FILLED
        
        self.ts_last = UnixNanos.now()
    
    def cancel(self) -> None:
        """Cancel this order."""
        if self.is_closed:
            raise ValueError(f"Cannot cancel order in {self.status} state")
        
        self.status = OrderStatus.CANCELED
        self.ts_last = UnixNanos.now()
    
    def expire(self) -> None:
        """Expire this order."""
        if self.is_closed:
            raise ValueError(f"Cannot expire order in {self.status} state")
        
        self.status = OrderStatus.EXPIRED  
        self.ts_last = UnixNanos.now()
    
    def update_quantity(self, new_quantity: Quantity) -> None:
        """Update order quantity."""
        if new_quantity <= self.filled_qty:
            raise ValueError("New quantity cannot be less than filled quantity")
        
        self.quantity = new_quantity
        self.leaves_qty = self.quantity - self.filled_qty
        self.ts_last = UnixNanos.now()
    
    def __str__(self) -> str:
        return (f"Order({self.client_order_id}, {self.instrument_id}, "
                f"{self.order_side.name}, {self.order_type.name}, "
                f"{self.quantity}, {self.status.name})")
    
    def __repr__(self) -> str:
        return self.__str__()


@dataclass
class MarketOrder(Order):
    """Market order - executes immediately at best available price."""
    
    def __post_init__(self):
        self.order_type = OrderType.MARKET
        self.time_in_force = TimeInForce.IOC  # Market orders are typically IOC
        super().__post_init__()


@dataclass
class LimitOrder(Order):
    """Limit order - executes at specified price or better."""
    
    def __post_init__(self):
        if self.price is None:
            raise ValueError("Limit order must have a price")
        self.order_type = OrderType.LIMIT
        super().__post_init__()


@dataclass
class StopOrder(Order):
    """Stop order - becomes market order when trigger price hit."""
    
    def __post_init__(self):
        if self.trigger_price is None:
            raise ValueError("Stop order must have a trigger price")
        self.order_type = OrderType.STOP
        if self.trigger_type == TriggerType.NO_TRIGGER:
            self.trigger_type = TriggerType.LAST
        super().__post_init__()


@dataclass
class StopLimitOrder(Order):
    """Stop-limit order - becomes limit order when trigger price hit."""
    
    def __post_init__(self):
        if self.trigger_price is None:
            raise ValueError("Stop-limit order must have a trigger price")
        if self.price is None:
            raise ValueError("Stop-limit order must have a limit price")
        self.order_type = OrderType.STOP_LIMIT
        if self.trigger_type == TriggerType.NO_TRIGGER:
            self.trigger_type = TriggerType.LAST
        super().__post_init__()


@dataclass(frozen=True)
class OrderFill:
    """Order fill/execution event."""
    
    client_order_id: ClientOrderId
    venue_order_id: VenueOrderId
    trade_id: str
    position_id: Optional[PositionId]
    strategy_id: StrategyId
    instrument_id: InstrumentId
    order_side: OrderSide
    order_type: OrderType
    
    # Fill details
    last_qty: Quantity
    last_px: Price
    currency: str
    commission: Quantity
    liquidity_side: LiquiditySide
    
    # Cumulative totals
    cum_qty: Quantity
    leaves_qty: Quantity
    
    # Timestamps
    ts_event: UnixNanos
    ts_init: UnixNanos = field(default_factory=UnixNanos.now)
    
    # Optional fields
    venue_position_id: Optional[str] = None
    exec_algorithm_id: Optional[str] = None
    exec_spawn_id: Optional[ClientOrderId] = None
    info: Dict[str, Any] = field(default_factory=dict)


@dataclass
class OrderList:
    """Container for related orders (OCO, OTO, brackets, etc.)."""
    
    id: str
    orders: List[Order]
    contingency_type: ContingencyType
    
    # State tracking
    first: Optional[ClientOrderId] = None
    orders_filled: int = 0
    orders_working: int = 0
    
    def add_order(self, order: Order) -> None:
        """Add an order to this list."""
        order.order_list_id = self.id
        self.orders.append(order)
        
        if self.first is None:
            self.first = order.client_order_id
        
        if order.is_working:
            self.orders_working += 1
    
    def remove_order(self, client_order_id: ClientOrderId) -> Optional[Order]:
        """Remove an order from this list."""
        for i, order in enumerate(self.orders):
            if order.client_order_id == client_order_id:
                removed = self.orders.pop(i)
                if removed.is_working:
                    self.orders_working -= 1
                return removed
        return None
    
    @property
    def is_working(self) -> bool:
        """Check if any orders in the list are working."""
        return self.orders_working > 0
