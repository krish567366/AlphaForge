# Test AlphaForge Core Functionality
"""
Basic tests to verify AlphaForge components work correctly.
"""

import pytest
from alphaforge.core import uuid4_new, UnixNanos, AtomicTime
from alphaforge.core.exceptions import ValidationError
from alphaforge.model.identifiers import InstrumentId, ClientOrderId, AccountId
from alphaforge.model.enums import OrderSide, OrderType, OrderStatus
from alphaforge.model.data import Price, Quantity


class TestCoreUtils:
    """Test core utility functions."""
    
    def test_uuid_generation(self):
        """Test UUID generation."""
        uuid1 = uuid4_new()
        uuid2 = uuid4_new()
        
        assert isinstance(uuid1, str)
        assert isinstance(uuid2, str)
        assert uuid1 != uuid2
        assert len(uuid1) == 36  # UUID4 string length
    
    def test_unix_nanos(self):
        """Test UnixNanos time handling."""
        now1 = UnixNanos.now()
        now2 = UnixNanos.now()
        
        assert isinstance(now1.value, int)
        assert isinstance(now2.value, int)
        assert now2.value >= now1.value
        
        # Test from/to conversions
        millis = now1.to_millis()
        from_millis = UnixNanos.from_millis(millis)
        assert abs(from_millis.value - now1.value) < 1_000_000  # Within 1ms
    
    def test_atomic_time(self):
        """Test atomic time operations."""
        atomic = AtomicTime()
        time1 = atomic.get()
        
        import time
        time.sleep(0.001)  # Sleep 1ms
        
        atomic.update_to_now()
        time2 = atomic.get()
        
        assert time2.value > time1.value


class TestIdentifiers:
    """Test identifier types."""
    
    def test_instrument_id(self):
        """Test InstrumentId creation and validation."""
        # Valid instrument ID
        inst_id = InstrumentId("BTCUSD.BINANCE")
        assert inst_id.symbol == "BTCUSD"
        assert inst_id.venue == "BINANCE"
        assert str(inst_id) == "BTCUSD.BINANCE"
        
        # From parts
        inst_id2 = InstrumentId.from_parts("ETHUSD", "COINBASE")
        assert inst_id2.value == "ETHUSD.COINBASE"
        
        # Invalid formats
        with pytest.raises(ValidationError):
            InstrumentId("INVALID")
        
        with pytest.raises(ValidationError):
            InstrumentId("TOO.MANY.PARTS")
        
        with pytest.raises(ValidationError):
            InstrumentId(".EMPTY_SYMBOL")
    
    def test_client_order_id(self):
        """Test ClientOrderId creation."""
        # From string
        order_id = ClientOrderId("ORDER_123")
        assert order_id.value == "ORDER_123"
        
        # Generated
        generated = ClientOrderId.generate()
        assert len(generated.value) == 36  # UUID length
        
        # Empty validation
        with pytest.raises(ValidationError):
            ClientOrderId("")
        
        # Too long validation  
        with pytest.raises(ValidationError):
            ClientOrderId("X" * 65)
    
    def test_account_id(self):
        """Test AccountId creation."""
        account = AccountId("BINANCE", "12345")
        assert account.issuer == "BINANCE"
        assert account.number == "12345"
        assert account.value == "BINANCE-12345"
        
        # Empty validation
        with pytest.raises(ValidationError):
            AccountId("", "123")
        
        with pytest.raises(ValidationError):
            AccountId("BINANCE", "")


class TestEnums:
    """Test enumeration types."""
    
    def test_order_enums(self):
        """Test order-related enums."""
        assert OrderSide.BUY == 1
        assert OrderSide.SELL == 2
        
        assert OrderType.MARKET == 1
        assert OrderType.LIMIT == 2
        
        assert OrderStatus.INITIALIZED == 0
        assert OrderStatus.FILLED == 14


class TestDataTypes:
    """Test data types like Price and Quantity."""
    
    def test_price_creation(self):
        """Test Price creation and operations."""
        # From string
        price1 = Price.from_str("123.45", precision=2)
        assert price1.as_str() == "123.45"
        assert price1.as_double() == 123.45
        
        # From float
        price2 = Price.from_float(99.99, precision=2)
        assert price2.as_str() == "99.99"
        
        # From int (raw value)
        price3 = Price.from_int(12345, precision=2)  # Represents 123.45
        assert price3.as_str() == "123.45"
        
        # Arithmetic
        price_sum = price1 + price2
        assert price_sum.as_double() == pytest.approx(223.44)
        
        price_diff = price1 - price2
        assert price_diff.as_double() == pytest.approx(23.46)
    
    def test_quantity_creation(self):
        """Test Quantity creation and operations."""
        # From string
        qty1 = Quantity.from_str("1000", precision=0)
        assert qty1.as_str() == "1000"
        assert qty1.as_double() == 1000.0
        
        # From float
        qty2 = Quantity.from_float(500.5, precision=1)
        assert qty2.as_str() == "500.5"
        
        # Arithmetic
        qty_sum = qty1 + Quantity.from_int(5000, precision=0)
        assert qty_sum.as_double() == 6000.0
    
    def test_price_quantity_validation(self):
        """Test Price and Quantity validation."""
        # Invalid precision
        with pytest.raises(ValueError):
            Price.from_int(100, precision=-1)
        
        with pytest.raises(ValueError):
            Price.from_int(100, precision=10)
        
        # Invalid string format
        with pytest.raises(ValueError):
            Price.from_str("not_a_number")
        
        # Different precision arithmetic should fail
        price1 = Price.from_int(100, precision=2)
        price2 = Price.from_int(100, precision=3)
        
        with pytest.raises(ValueError):
            price1 + price2


class TestPerformance:
    """Basic performance tests."""
    
    def test_price_performance(self):
        """Test Price operations performance."""
        import time
        
        # Create many prices
        start_time = time.perf_counter()
        prices = [Price.from_float(100.0 + i * 0.01, precision=2) for i in range(10000)]
        creation_time = time.perf_counter() - start_time
        
        # Perform arithmetic
        start_time = time.perf_counter() 
        total = prices[0]
        for price in prices[1:1000]:  # Subset to avoid overflow
            total = total + price
        arithmetic_time = time.perf_counter() - start_time
        
        # Basic performance assertions (should be very fast)
        assert creation_time < 1.0  # Should create 10k prices in < 1 second
        assert arithmetic_time < 0.1  # Should do 1k additions in < 0.1 seconds
        
        print(f"Price creation: {creation_time:.4f}s for 10k items")
        print(f"Price arithmetic: {arithmetic_time:.4f}s for 1k operations")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
