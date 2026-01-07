import pytest
from unittest.mock import AsyncMock, MagicMock, patch
from decimal import Decimal
from bot.services.wallet_manager import WalletManager
from bot.services.pivot_engine import PivotEngine
from bot.services.grid_builder import GridBuilder
from bot.services.order_manager import OrderManager
from bot.domain.models import AssetPosition

@pytest.mark.asyncio
async def test_bot_rebalance_integration():
    # 1. Setup mocks
    solana_mock = MagicMock()
    # Mock the internal method called by WalletManager.snapshot
    solana_mock.get_balance = AsyncMock(return_value=1000000)
    
    from solders.keypair import Keypair
    dummy_keypair = Keypair()
    wallet_manager = WalletManager(solana=solana_mock, keypairs=[dummy_keypair])
    pivot_engine = PivotEngine(target_allocation_usd=Decimal("1000"))
    grid_builder = GridBuilder(orders_per_side=2)
    order_manager = OrderManager(open_orders={})
    
    # 2. Simulate workflow with patching to avoid read-only slot issues
    with patch.object(OrderManager, 'place_grid', new_callable=AsyncMock) as mocked_place:
        # Get snapshot
        snapshot = await wallet_manager.snapshot("test_owner")
        assert snapshot.owner == "test_owner"
        assert snapshot.balance_lamports == 1000000
        
        # Compute pivot
        positions = [
            AssetPosition(symbol="SOL", quantity=Decimal("2"), notional_usd=Decimal("300"))
        ]
        market_data = [(Decimal("150"), Decimal("1000"))]
        pivot = await pivot_engine.compute_pivot(positions, market_data)
        assert pivot == Decimal("150")
        
        # Build and place grid
        grid = await grid_builder.build(mid_price=Decimal("150"), total_size=Decimal("0.1"))
        await order_manager.place_grid(grid)
        
        # Verify
        mocked_place.assert_called_once_with(grid)
        assert len(grid) == 4
