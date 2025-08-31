# AlphaForge Core Time module (fallback)
"""
Time handling utilities for AlphaForge trading system.
This is the Python fallback when Rust extensions are not available.
"""

import time
from datetime import datetime, timezone
from typing import Union


class UnixNanos:
    """Unix timestamp in nanoseconds."""
    
    def __init__(self, value: int):
        """Create from nanoseconds since Unix epoch."""
        self.value = value
    
    @classmethod
    def now(cls) -> "UnixNanos":
        """Get current time in nanoseconds."""
        return cls(int(time.time_ns()))
    
    @classmethod
    def from_millis(cls, millis: int) -> "UnixNanos":
        """Create from milliseconds."""
        return cls(millis * 1_000_000)
    
    @classmethod
    def from_micros(cls, micros: int) -> "UnixNanos":
        """Create from microseconds."""
        return cls(micros * 1_000)
    
    @classmethod
    def from_seconds(cls, seconds: Union[int, float]) -> "UnixNanos":
        """Create from seconds."""
        return cls(int(seconds * 1_000_000_000))
    
    @classmethod
    def from_datetime(cls, dt: datetime) -> "UnixNanos":
        """Create from datetime object."""
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=timezone.utc)
        return cls(int(dt.timestamp() * 1_000_000_000))
    
    def to_datetime(self) -> datetime:
        """Convert to datetime object."""
        return datetime.fromtimestamp(self.value / 1_000_000_000, tz=timezone.utc)
    
    def to_millis(self) -> int:
        """Convert to milliseconds."""
        return self.value // 1_000_000
    
    def to_micros(self) -> int:
        """Convert to microseconds."""
        return self.value // 1_000
    
    def to_seconds(self) -> float:
        """Convert to seconds."""
        return self.value / 1_000_000_000
    
    def __str__(self) -> str:
        return str(self.value)
    
    def __repr__(self) -> str:
        return f"UnixNanos({self.value})"
    
    def __eq__(self, other: object) -> bool:
        if not isinstance(other, UnixNanos):
            return False
        return self.value == other.value
    
    def __lt__(self, other: "UnixNanos") -> bool:
        return self.value < other.value
    
    def __le__(self, other: "UnixNanos") -> bool:
        return self.value <= other.value
    
    def __gt__(self, other: "UnixNanos") -> bool:
        return self.value > other.value
    
    def __ge__(self, other: "UnixNanos") -> bool:
        return self.value >= other.value
    
    def __add__(self, other: int) -> "UnixNanos":
        """Add nanoseconds."""
        return UnixNanos(self.value + other)
    
    def __sub__(self, other: Union["UnixNanos", int]) -> Union["UnixNanos", int]:
        """Subtract nanoseconds or another UnixNanos."""
        if isinstance(other, UnixNanos):
            return self.value - other.value
        return UnixNanos(self.value - other)
    
    def __hash__(self) -> int:
        return hash(self.value)


# Atomic time for shared state (simplified Python version)
class AtomicTime:
    """Atomic timestamp for thread-safe access."""
    
    def __init__(self, initial_time: UnixNanos = None):
        """Initialize with optional time."""
        import threading
        self._lock = threading.Lock()
        self._time = initial_time or UnixNanos.now()
    
    def get(self) -> UnixNanos:
        """Get current atomic time."""
        with self._lock:
            return UnixNanos(self._time.value)
    
    def set(self, time_ns: UnixNanos) -> None:
        """Set atomic time."""
        with self._lock:
            self._time = time_ns
    
    def update_to_now(self) -> None:
        """Update to current system time."""
        with self._lock:
            self._time = UnixNanos.now()
    
    def __str__(self) -> str:
        return str(self.get().value)
    
    def __repr__(self) -> str:
        return f"AtomicTime({self.get().value})"
