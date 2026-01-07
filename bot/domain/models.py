from __future__ import annotations

from dataclasses import dataclass
from decimal import Decimal
from enum import Enum
from typing import Literal

from pydantic import BaseModel, Field


class OrderSide(str, Enum):
    BUY = "buy"
    SELL = "sell"


class OrderStatus(str, Enum):
    PENDING = "pending"
    OPEN = "open"
    FILLED = "filled"
    CANCELED = "canceled"
    FAILED = "failed"


class AssetPosition(BaseModel):
    symbol: str
    quantity: Decimal = Field(default=Decimal("0"))
    notional_usd: Decimal = Field(default=Decimal("0"))


class GridLevel(BaseModel):
    price: Decimal
    size: Decimal
    side: OrderSide


class RiskLimits(BaseModel):
    max_notional_usd: Decimal = Field(default=Decimal("1000"))
    max_open_orders: int = 20


class FiatQuote(BaseModel):
    provider: str
    pair: Literal["USD/SOL", "USD/USDC"]
    price: Decimal


@dataclass(slots=True)
class WalletSnapshot:
    owner: str
    balance_lamports: int
    token_balances: dict[str, int]
