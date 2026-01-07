import pytest

from solana_dex_bmv.grid import build_grid


def test_grid_builder_count_and_spacing():
    orders = build_grid(center_price=100, spacing=2, levels=16)
    assert len(orders) == 32

    buy_prices = [order.price for order in orders if order.side == "buy"]
    sell_prices = [order.price for order in orders if order.side == "sell"]

    assert buy_prices[0] == 98
    assert sell_prices[0] == 102
    assert buy_prices[-1] == 68
    assert sell_prices[-1] == 132

    buy_spacings = [buy_prices[i] - buy_prices[i + 1] for i in range(len(buy_prices) - 1)]
    sell_spacings = [sell_prices[i + 1] - sell_prices[i] for i in range(len(sell_prices) - 1)]

    assert all(spacing == 2 for spacing in buy_spacings)
    assert all(spacing == 2 for spacing in sell_spacings)


def test_grid_builder_rejects_non_positive_prices():
    with pytest.raises(ValueError, match="non-positive grid prices"):
        build_grid(center_price=10, spacing=2, levels=6)
