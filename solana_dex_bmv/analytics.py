"""Price analytics helpers."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Iterable


@dataclass(frozen=True)
class Trade:
    price: float
    size: float


def _apply_costs(price: float, fee_bps: float, fixed_cost: float) -> float:
    return price * (1 + fee_bps / 10_000) + fixed_cost


def vwap_price(trades: Iterable[Trade], fee_bps: float = 0.0, fixed_cost: float = 0.0) -> float:
    trades_list = list(trades)
    if not trades_list:
        raise ValueError("trades cannot be empty")
    total_notional = sum(trade.price * trade.size for trade in trades_list)
    total_size = sum(trade.size for trade in trades_list)
    if total_size == 0:
        raise ValueError("total trade size cannot be zero")
    base_vwap = total_notional / total_size
    return _apply_costs(base_vwap, fee_bps, fixed_cost)


def pivot_price(
    high: float,
    low: float,
    close: float,
    fee_bps: float = 0.0,
    fixed_cost: float = 0.0,
) -> float:
    base_pivot = (high + low + close) / 3
    return _apply_costs(base_pivot, fee_bps, fixed_cost)
