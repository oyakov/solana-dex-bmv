"""Grid builder helpers."""

from __future__ import annotations

from dataclasses import dataclass
from typing import List


@dataclass(frozen=True)
class GridOrder:
    side: str
    price: float
    level: int


def build_grid(center_price: float, spacing: float, levels: int = 16) -> List[GridOrder]:
    if center_price <= 0:
        raise ValueError("center_price must be positive")
    if spacing <= 0:
        raise ValueError("spacing must be positive")
    if levels <= 0:
        raise ValueError("levels must be positive")
    if center_price - spacing * levels <= 0:
        raise ValueError("levels would create non-positive grid prices")

    orders: List[GridOrder] = []
    for level in range(1, levels + 1):
        orders.append(GridOrder(side="buy", price=center_price - spacing * level, level=level))
        orders.append(GridOrder(side="sell", price=center_price + spacing * level, level=level))
    return orders
