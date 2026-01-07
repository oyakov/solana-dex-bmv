import pytest

from solana_dex_bmv.analytics import Trade, pivot_price, vwap_price


def test_vwap_price_applies_fees_and_costs():
    trades = [Trade(price=100, size=2), Trade(price=110, size=1)]
    base_vwap = (100 * 2 + 110 * 1) / 3
    expected = base_vwap * 1.005 + 0.25
    assert vwap_price(trades, fee_bps=50, fixed_cost=0.25) == pytest.approx(expected)


def test_pivot_price_applies_fees_and_costs():
    base_pivot = (120 + 90 + 100) / 3
    expected = base_pivot * 1.002 + 0.1
    assert pivot_price(120, 90, 100, fee_bps=20, fixed_cost=0.1) == pytest.approx(expected)
