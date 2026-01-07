import pytest
from decimal import Decimal
from bot.services.pivot_engine import PivotEngine
from bot.domain.models import AssetPosition

@pytest.mark.asyncio
async def test_compute_pivot_empty_positions():
    engine = PivotEngine(target_allocation_usd=Decimal("1000"))
    # Should return target_allocation_usd if no market data
    pivot = await engine.compute_pivot([], [])
    assert pivot == Decimal("1000")

@pytest.mark.asyncio
async def test_compute_pivot_with_positions():
    engine = PivotEngine(target_allocation_usd=Decimal("1000"), initial_fade_in_days=30)
    positions = [
        AssetPosition(symbol="SOL", quantity=Decimal("2"), notional_usd=Decimal("300")),
    ]
    # VWAP = (100*10 + 110*10) / 20 = 2100 / 20 = 105
    market_data = [(Decimal("100"), Decimal("10")), (Decimal("110"), Decimal("10"))]
    
    # Case 1: Fade-in complete
    pivot = await engine.compute_pivot(positions, market_data, days_since_start=31)
    assert pivot == Decimal("105")

@pytest.mark.asyncio
async def test_compute_pivot_fade_in():
    engine = PivotEngine(target_allocation_usd=Decimal("1000"), initial_fade_in_days=10)
    positions = []
    market_data = [(Decimal("100"), Decimal("10")), (Decimal("110"), Decimal("10"))] # VWAP 105, current 110
    
    # Halfway fade-in: 5 days out of 10. 
    # Ratio = 0.5. Pivot = 110 * 0.5 + 105 * 0.5 = 107.5
    pivot = await engine.compute_pivot(positions, market_data, days_since_start=5)
    assert pivot == Decimal("107.5")
