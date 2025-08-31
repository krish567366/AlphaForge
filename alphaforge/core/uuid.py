# AlphaForge Core UUID utilities
"""
UUID generation utilities for AlphaForge trading system.
"""

import uuid


def uuid4_new() -> str:
    """Generate a new UUID4 string."""
    return str(uuid.uuid4())


def uuid4_bytes() -> bytes:
    """Generate a new UUID4 as bytes."""
    return uuid.uuid4().bytes


def is_valid_uuid4(uuid_str: str) -> bool:
    """Check if string is a valid UUID4."""
    try:
        uuid_obj = uuid.UUID(uuid_str, version=4)
        return str(uuid_obj) == uuid_str
    except (ValueError, TypeError):
        return False
