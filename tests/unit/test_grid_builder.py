import pytest
from decimal import Decimal
from bot.services.grid_builder import GridBuilder
from bot.domain.models import OrderSide

@pytest.mark.asyncio
async def test_build_grid_basic():
    builder = GridBuilder(orders_per_side=2, buy_channel_width=Decimal("0.1"), sell_channel_width=Decimal("0.2"))
    mid_price = Decimal("100")
    size = Decimal("1")
    grid = await builder.build(mid_price, size)
    
    assert len(grid) == 4
    # buy_step = (100 * 0.1) / 2 = 5
    # prices: 100-5=95, 100-10=90
    # sell_step = (100 * 0.2) / 2 = 10
    # prices: 100+10=110, 100+20=120
    
    buy_prices = sorted([l.price for l in grid if l.side == OrderSide.BUY], reverse=True)
    sell_prices = sorted([l.price for l in grid if l.side == OrderSide.SELL])
    
    assert buy_prices == [Decimal("95"), Decimal("90")]
    assert sell_prices == [Decimal("110"), Decimal("120")]

@pytest.mark.asyncio
async def test_build_grid_zero_levels():
    builder = GridBuilder(orders_per_side=0)
    grid = await builder.build(Decimal("100"), Decimal("1"))
    assert len(grid) == 0
