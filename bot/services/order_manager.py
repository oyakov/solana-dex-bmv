from __future__ import annotations

from dataclasses import dataclass
from decimal import Decimal

from bot.domain.models import GridLevel, OrderStatus
from bot.utils.logging import get_logger

logger = get_logger(__name__)


@dataclass(slots=True)
class OrderTicket:
    order_id: str
    status: OrderStatus
    price: Decimal
    size: Decimal


@dataclass(slots=True)
class OrderManager:
    open_orders: dict[str, OrderTicket]

    async def place_grid(self, grid: list[GridLevel]) -> list[OrderTicket]:
        tickets: list[OrderTicket] = []
        for index, level in enumerate(grid, start=1):
            order_id = f"grid-{index}"
            ticket = OrderTicket(
                order_id=order_id,
                status=OrderStatus.OPEN,
                price=level.price,
                size=level.size,
            )
            self.open_orders[order_id] = ticket
            tickets.append(ticket)
            logger.info("order_placed", order_id=order_id, side=level.side.value)
        return tickets

    async def cancel_all(self) -> None:
        for order_id in list(self.open_orders):
            logger.info("order_canceled", order_id=order_id)
            self.open_orders.pop(order_id, None)
