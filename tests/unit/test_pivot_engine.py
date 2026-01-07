import pytest
from decimal import Decimal
from bot.services.pivot_engine import PivotEngine
from bot.domain.models import AssetPosition

@pytest.mark.asyncio
async def test_compute_pivot_empty_positions():
    engine = PivotEngine(target_allocation_usd=Decimal("1000"))
    pivot = await engine.compute_pivot([])
    assert pivot == Decimal("1000")

@pytest.mark.asyncio
async def test_compute_pivot_with_positions():
    engine = PivotEngine(target_allocation_usd=Decimal("1000"))
    positions = [
        AssetPosition(symbol="SOL", quantity=Decimal("2"), notional_usd=Decimal("300")),
        AssetPosition(symbol="BMV", quantity=Decimal("100"), notional_usd=Decimal("150")),
    ]
    pivot = await engine.compute_pivot(positions)
    assert pivot == Decimal("550") # 1000 - (300 + 150)

@pytest.mark.asyncio
async def test_compute_pivot_negative_pivot():
    engine = PivotEngine(target_allocation_usd=Decimal("1000"))
    positions = [
        AssetPosition(symbol="SOL", quantity=Decimal("10"), notional_usd=Decimal("1500")),
    ]
    pivot = await engine.compute_pivot(positions)
    assert pivot == Decimal("-500")
