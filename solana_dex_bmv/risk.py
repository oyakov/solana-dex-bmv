"""Risk management helpers."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class RiskLimits:
    max_order_size: float
    max_position: float
    max_notional: float


class RiskManager:
    def __init__(self, limits: RiskLimits) -> None:
        self._limits = limits

    @property
    def limits(self) -> RiskLimits:
        return self._limits

    def validate_order(self, order_size: float, current_position: float, price: float) -> None:
        if abs(order_size) > self._limits.max_order_size:
            raise ValueError("order size exceeds max_order_size")
        projected_position = current_position + order_size
        if abs(projected_position) > self._limits.max_position:
            raise ValueError("order exceeds max_position")
        notional = abs(order_size) * price
        if notional > self._limits.max_notional:
            raise ValueError("order exceeds max_notional")
