import pytest
from decimal import Decimal
from bot.services.grid_builder import GridBuilder
from bot.domain.models import OrderSide

@pytest.mark.asyncio
async def test_build_grid_basic():
    builder = GridBuilder(spacing_bps=100, levels=2) # 1% spacing
    mid_price = Decimal("100")
    size = Decimal("1")
    grid = await builder.build(mid_price, size)
    
    assert len(grid) == 4
    # Level 1
    assert grid[0].price == Decimal("99") # 100 * (1 - 0.01)
    assert grid[0].side == OrderSide.BUY
    assert grid[1].price == Decimal("101") # 100 * (1 + 0.01)
    assert grid[1].side == OrderSide.SELL
    # Level 2
    assert grid[2].price == Decimal("98") # 100 * (1 - 0.02)
    assert grid[3].price == Decimal("102") # 100 * (1 + 0.02)

@pytest.mark.asyncio
async def test_build_grid_zero_levels():
    builder = GridBuilder(spacing_bps=100, levels=0)
    grid = await builder.build(Decimal("100"), Decimal("1"))
    assert len(grid) == 0
