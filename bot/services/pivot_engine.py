from __future__ import annotations

from dataclasses import dataclass
from decimal import Decimal

from bot.domain.models import AssetPosition
from bot.utils.logging import get_logger

logger = get_logger(__name__)


@dataclass(slots=True)
class PivotEngine:
    target_allocation_usd: Decimal

    async def compute_pivot(self, positions: list[AssetPosition]) -> Decimal:
        total = sum((position.notional_usd for position in positions), Decimal("0"))
        pivot = self.target_allocation_usd - total
        logger.info("pivot_computed", target=self.target_allocation_usd, total=total, pivot=pivot)
        return pivot
