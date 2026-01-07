from __future__ import annotations

from dataclasses import dataclass
from decimal import Decimal

from bot.domain.models import GridLevel, OrderSide
from bot.utils.logging import get_logger

logger = get_logger(__name__)


@dataclass(slots=True)
class GridBuilder:
    orders_per_side: int = 16
    buy_channel_width: Decimal = Decimal("0.15")  # 15%
    sell_channel_width: Decimal = Decimal("0.30")  # 30%

    async def build(self, mid_price: Decimal, total_size: Decimal) -> list[GridLevel]:
        """
        Builds a grid of 32 orders (16 BUY, 16 SELL).
        The BUY zone is denser/narrower for support.
        The SELL zone is wider to encourage growth.
        """
        if self.orders_per_side <= 0:
            return []
            
        grid: list[GridLevel] = []
        # BUY orders (Support)
        # We distribute orders over the buy_channel_width
        buy_step = (mid_price * self.buy_channel_width) / Decimal(self.orders_per_side)
        size_per_order = total_size / Decimal(self.orders_per_side * 2) # Simplistic equal weight for now

        for i in range(1, self.orders_per_side + 1):
            price = mid_price - (buy_step * i)
            grid.append(GridLevel(price=price, size=size_per_order, side=OrderSide.BUY))

        # SELL orders (Growth)
        sell_step = (mid_price * self.sell_channel_width) / Decimal(self.orders_per_side)
        for i in range(1, self.orders_per_side + 1):
            price = mid_price + (sell_step * i)
            grid.append(GridLevel(price=price, size=size_per_order, side=OrderSide.SELL))

        logger.info(
            "grid_built", 
            mid_price=mid_price, 
            buy_levels=self.orders_per_side, 
            sell_levels=self.orders_per_side,
            buy_width=self.buy_channel_width,
            sell_width=self.sell_channel_width
        )
        return grid
