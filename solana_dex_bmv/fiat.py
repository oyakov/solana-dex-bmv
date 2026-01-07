"""Fiat buffer and injection management."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Protocol


class RpcClient(Protocol):
    def get_usd_balance(self) -> float:  # pragma: no cover - protocol definition
        ...


class JitoClient(Protocol):
    def send_injection(self, amount: float) -> str:  # pragma: no cover - protocol definition
        ...


@dataclass(frozen=True)
class FiatPolicy:
    buffer_ratio: float
    min_buffer: float
    injection_amount: float


class FiatManager:
    def __init__(self, policy: FiatPolicy) -> None:
        self._policy = policy

    @property
    def policy(self) -> FiatPolicy:
        return self._policy

    def required_buffer(self, target_exposure: float) -> float:
        return max(self._policy.min_buffer, target_exposure * self._policy.buffer_ratio)

    def needs_injection(self, available_usd: float, target_exposure: float) -> bool:
        return available_usd < self.required_buffer(target_exposure)

    def injection_needed(self, available_usd: float, target_exposure: float) -> float:
        if self.needs_injection(available_usd, target_exposure):
            return self._policy.injection_amount
        return 0.0

    def rebalance(self, rpc_client: RpcClient, jito_client: JitoClient, target_exposure: float) -> float:
        available = rpc_client.get_usd_balance()
        amount = self.injection_needed(available, target_exposure)
        if amount > 0:
            jito_client.send_injection(amount)
        return amount
