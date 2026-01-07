import pytest
from bot.services.jito_manager import JitoManager

def test_jito_tip_calculation():
    manager = JitoManager(base_tip_lamports=1000)
    
    # Test low congestion
    tip_low = asyncio.run(manager.calculate_tip("low"))
    assert tip_low == 500
    
    # Test high congestion
    tip_high = asyncio.run(manager.calculate_tip("high"))
    assert tip_high == 2000
    
    # Test default
    tip_default = asyncio.run(manager.calculate_tip("unknown"))
    assert tip_default == 1000

def test_jito_bundle_assembly():
    manager = JitoManager()
    txs = ["tx1", "tx2"]
    tip_tx = "tip_tx"
    
    bundle = manager.build_bundle(txs, tip_tx)
    assert len(bundle) == 3
    assert bundle[-1] == "tip_tx"
    assert "tx1" in bundle
    assert "tx2" in bundle

def test_jito_bundle_empty_txs():
    manager = JitoManager()
    with pytest.raises(ValueError, match="bundle_must_have_at_least_one_transaction"):
        manager.build_bundle([], "tip_tx")

import asyncio
