import pytest

from solana_dex_bmv.risk import RiskLimits, RiskManager


def test_risk_manager_allows_within_limits():
    limits = RiskLimits(max_order_size=5, max_position=10, max_notional=500)
    manager = RiskManager(limits)

    manager.validate_order(order_size=2, current_position=3, price=100)


def test_risk_manager_rejects_violations():
    limits = RiskLimits(max_order_size=5, max_position=10, max_notional=500)
    manager = RiskManager(limits)

    with pytest.raises(ValueError, match="max_order_size"):
        manager.validate_order(order_size=6, current_position=0, price=50)

    with pytest.raises(ValueError, match="max_position"):
        manager.validate_order(order_size=4, current_position=7, price=50)

    with pytest.raises(ValueError, match="max_notional"):
        manager.validate_order(order_size=5, current_position=0, price=200)
