import pytest
from decimal import Decimal
from bot.services.grid_builder import GridBuilder
from bot.domain.models import OrderSide

@pytest.mark.asyncio
async def test_grid_extreme_volatility():
    """Verifies that GridBuilder handles extreme channel widths (e.g. 99% crash)."""
    # 90% buy channel, 500% sell channel
    builder = GridBuilder(orders_per_side=5, buy_channel_width=Decimal("0.90"), sell_channel_width=Decimal("5.0"))
    mid_price = Decimal("100")
    total_size = Decimal("10")
    
    grid = await builder.build(mid_price, total_size)
    assert len(grid) == 10
    
    # Lowest buy price: 100 - (100 * 0.9 / 5 * 5) = 100 - 90 = 10
    buy_prices = [l.price for l in grid if l.side == OrderSide.BUY]
    assert min(buy_prices) == Decimal("10") 
    assert Decimal("10") in buy_prices

@pytest.mark.asyncio
async def test_grid_dust_size():
    """Verifies handling of extremely small total sizes (dust)."""
    builder = GridBuilder(orders_per_side=10)
    mid_price = Decimal("100")
    # total_size = 0.000000001 (1 lamport if it were SOL)
    total_size = Decimal("0.000000001")
    
    grid = await builder.build(mid_price, total_size)
    assert len(grid) == 20
    # Each order size = total_size / (10 * 2) = 0.000000001 / 20 = 0.00000000005
    for level in grid:
        assert level.size > 0

@pytest.mark.asyncio
async def test_grid_near_zero_price():
    """Verifies stability when mid-price is very low."""
    builder = GridBuilder(orders_per_side=5, buy_channel_width=Decimal("0.1"))
    mid_price = Decimal("0.00001")
    total_size = Decimal("1000")
    
    grid = await builder.build(mid_price, total_size)
    assert len(grid) == 10
    for level in grid:
        assert level.price > 0
        assert level.price < Decimal("0.00002")
