from __future__ import annotations

from dataclasses import dataclass
from decimal import Decimal

from bot.domain.models import GridLevel, OrderSide
from bot.utils.logging import get_logger

logger = get_logger(__name__)


@dataclass(slots=True)
class GridBuilder:
    spacing_bps: int
    levels: int

    async def build(self, mid_price: Decimal, size: Decimal) -> list[GridLevel]:
        grid: list[GridLevel] = []
        spacing = Decimal(self.spacing_bps) / Decimal("10000")
        for level in range(1, self.levels + 1):
            offset = spacing * level
            buy_price = mid_price * (Decimal("1") - offset)
            sell_price = mid_price * (Decimal("1") + offset)
            grid.append(GridLevel(price=buy_price, size=size, side=OrderSide.BUY))
            grid.append(GridLevel(price=sell_price, size=size, side=OrderSide.SELL))
        logger.info("grid_built", mid_price=mid_price, levels=len(grid))
        return grid
