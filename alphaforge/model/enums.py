# AlphaForge Enumerations
"""
Enumeration types for AlphaForge trading system.
"""

from enum import Enum, IntEnum


class OrderSide(IntEnum):
    """Order side enumeration."""
    NO_ORDER_SIDE = 0
    BUY = 1
    SELL = 2


class OrderType(IntEnum):
    """Order type enumeration."""
    NO_ORDER_TYPE = 0
    MARKET = 1
    LIMIT = 2
    STOP = 3
    STOP_LIMIT = 4
    MARKET_TO_LIMIT = 5
    MARKET_IF_TOUCHED = 6
    LIMIT_IF_TOUCHED = 7
    TRAILING_STOP_MARKET = 8
    TRAILING_STOP_LIMIT = 9


class OrderStatus(IntEnum):
    """Order status enumeration."""
    INITIALIZED = 0
    INVALID = 1
    DENIED = 2
    EMULATED = 3
    RELEASED = 4
    PENDING_UPDATE = 5
    PENDING_CANCEL = 6
    ACCEPTED = 7
    REJECTED = 8
    CANCELED = 9
    EXPIRED = 10
    TRIGGERED = 11
    PENDING_NEW = 12
    PARTIALLY_FILLED = 13
    FILLED = 14


class TimeInForce(IntEnum):
    """Time in force enumeration."""
    NO_TIME_IN_FORCE = 0
    DAY = 1
    GTC = 2  # Good Till Cancel
    IOC = 3  # Immediate Or Cancel
    FOK = 4  # Fill Or Kill
    GTD = 5  # Good Till Date
    AT_THE_OPEN = 6
    AT_THE_CLOSE = 7


class PositionSide(IntEnum):
    """Position side enumeration."""
    NO_POSITION_SIDE = 0
    FLAT = 1
    LONG = 2
    SHORT = 3


class LiquiditySide(IntEnum):
    """Liquidity side enumeration."""
    NO_LIQUIDITY_SIDE = 0
    MAKER = 1
    TAKER = 2


class OrderBookAction(IntEnum):
    """Order book action enumeration."""
    ADD = 1
    UPDATE = 2
    DELETE = 3
    CLEAR = 4


class MarketStatus(IntEnum):
    """Market status enumeration."""
    CLOSED = 0
    PRE_OPEN = 1
    OPEN = 2
    PAUSE = 3
    HALT = 4
    RESUME = 5
    PRE_CLOSE = 6


class InstrumentClass(IntEnum):
    """Instrument class enumeration."""
    SPOT = 1
    SWAP = 2
    FUTURE = 3
    FORWARD = 4
    CFD = 5
    BOND = 6
    OPTION = 7
    WARRANT = 8


class AssetClass(IntEnum):
    """Asset class enumeration."""
    FX = 1
    EQUITY = 2
    COMMODITY = 3
    DEBT = 4
    INDEX = 5
    CRYPTOCURRENCY = 6
    ALTERNATIVE = 7


class CurrencyType(IntEnum):
    """Currency type enumeration."""
    ISO = 1
    CRYPTO = 2
    COMMODITY_BACKED = 3


class AccountType(IntEnum):
    """Account type enumeration."""
    CASH = 1
    MARGIN = 2
    BETTING = 3


class OmsType(IntEnum):
    """Order Management System type."""
    UNSPECIFIED = 0
    NETTING = 1
    HEDGING = 2


class StrategyStatus(IntEnum):
    """Strategy status enumeration."""
    INITIALIZED = 0
    STARTING = 1
    RUNNING = 2
    STOPPING = 3
    STOPPED = 4
    DISPOSED = 5


class ComponentStatus(IntEnum):
    """Component status enumeration."""
    PRE_INITIALIZED = 0
    READY = 1
    STARTING = 2
    RUNNING = 3
    STOPPING = 4
    STOPPED = 5
    RESUMING = 6
    RESETTING = 7
    DISPOSING = 8
    DISPOSED = 9
    DEGRADED = 10
    FAULTED = 11


class LogLevel(IntEnum):
    """Logging level enumeration."""
    DEBUG = 10
    INFO = 20
    WARNING = 30
    ERROR = 40
    CRITICAL = 50


class AggressorSide(IntEnum):
    """Aggressor side for trades."""
    NO_AGGRESSOR = 0
    BUYER = 1
    SELLER = 2


class BookType(IntEnum):
    """Order book type enumeration."""
    L1_TBBO = 1  # Top of Book Best Bid/Offer
    L2_MBP = 2   # Market By Price
    L3_MBO = 3   # Market By Order


class PriceType(IntEnum):
    """Price type enumeration."""
    BID = 1
    ASK = 2
    MID = 3
    LAST = 4


class BarAggregation(IntEnum):
    """Bar aggregation method."""
    TICK = 1
    TICK_IMBALANCE = 2
    TICK_RUNS = 3
    VOLUME = 4
    VOLUME_IMBALANCE = 5
    VOLUME_RUNS = 6
    VALUE = 7
    VALUE_IMBALANCE = 8
    VALUE_RUNS = 9
    MILLISECOND = 10
    SECOND = 11
    MINUTE = 12
    HOUR = 13
    DAY = 14
    WEEK = 15
    MONTH = 16


class TriggerType(IntEnum):
    """Trigger type for conditional orders."""
    NO_TRIGGER = 0
    DEFAULT = 1
    BID_ASK = 2
    LAST = 3
    DOUBLE_LAST = 4
    DOUBLE_BIDASK = 5
    LAST_OR_BID_ASK = 6
    MID_POINT = 7
    MARK_PRICE = 8
    INDEX_PRICE = 9


class ContingencyType(IntEnum):
    """Contingency type for related orders."""
    NO_CONTINGENCY = 0
    OTO = 1  # One-Triggers-Other
    OCO = 2  # One-Cancels-Other
    OUO = 3  # One-Updates-Other


# String mappings for display
ORDER_SIDE_MAP = {
    OrderSide.NO_ORDER_SIDE: "NO_ORDER_SIDE",
    OrderSide.BUY: "BUY", 
    OrderSide.SELL: "SELL"
}

ORDER_TYPE_MAP = {
    OrderType.NO_ORDER_TYPE: "NO_ORDER_TYPE",
    OrderType.MARKET: "MARKET",
    OrderType.LIMIT: "LIMIT",
    OrderType.STOP: "STOP",
    OrderType.STOP_LIMIT: "STOP_LIMIT",
    OrderType.MARKET_TO_LIMIT: "MARKET_TO_LIMIT",
    OrderType.MARKET_IF_TOUCHED: "MARKET_IF_TOUCHED",
    OrderType.LIMIT_IF_TOUCHED: "LIMIT_IF_TOUCHED",
    OrderType.TRAILING_STOP_MARKET: "TRAILING_STOP_MARKET",
    OrderType.TRAILING_STOP_LIMIT: "TRAILING_STOP_LIMIT"
}

ORDER_STATUS_MAP = {
    OrderStatus.INITIALIZED: "INITIALIZED",
    OrderStatus.INVALID: "INVALID",
    OrderStatus.DENIED: "DENIED",
    OrderStatus.EMULATED: "EMULATED",
    OrderStatus.RELEASED: "RELEASED",
    OrderStatus.PENDING_UPDATE: "PENDING_UPDATE",
    OrderStatus.PENDING_CANCEL: "PENDING_CANCEL",
    OrderStatus.ACCEPTED: "ACCEPTED",
    OrderStatus.REJECTED: "REJECTED",
    OrderStatus.CANCELED: "CANCELED",
    OrderStatus.EXPIRED: "EXPIRED",
    OrderStatus.TRIGGERED: "TRIGGERED",
    OrderStatus.PENDING_NEW: "PENDING_NEW",
    OrderStatus.PARTIALLY_FILLED: "PARTIALLY_FILLED",
    OrderStatus.FILLED: "FILLED"
}

TIME_IN_FORCE_MAP = {
    TimeInForce.NO_TIME_IN_FORCE: "NO_TIME_IN_FORCE",
    TimeInForce.DAY: "DAY",
    TimeInForce.GTC: "GTC",
    TimeInForce.IOC: "IOC",
    TimeInForce.FOK: "FOK",
    TimeInForce.GTD: "GTD",
    TimeInForce.AT_THE_OPEN: "AT_THE_OPEN",
    TimeInForce.AT_THE_CLOSE: "AT_THE_CLOSE"
}

POSITION_SIDE_MAP = {
    PositionSide.NO_POSITION_SIDE: "NO_POSITION_SIDE",
    PositionSide.FLAT: "FLAT",
    PositionSide.LONG: "LONG",
    PositionSide.SHORT: "SHORT"
}

LIQUIDITY_SIDE_MAP = {
    LiquiditySide.NO_LIQUIDITY_SIDE: "NO_LIQUIDITY_SIDE",
    LiquiditySide.MAKER: "MAKER",
    LiquiditySide.TAKER: "TAKER"
}
