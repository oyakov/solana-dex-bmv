from __future__ import annotations

from dataclasses import dataclass
from decimal import Decimal

from bot.domain.models import AssetPosition
from bot.services.fiat_manager import FiatManager
from bot.services.risk_manager import RiskManager
from bot.utils.logging import get_logger

logger = get_logger(__name__)


@dataclass(slots=True)
class RebalanceOrchestrator:
    fiat_manager: FiatManager
    risk_manager: RiskManager

    async def evaluate(self, positions: list[AssetPosition], threshold_bps: int) -> bool:
        total_notional = sum((position.notional_usd for position in positions), Decimal("0"))
        within_limits = self.risk_manager.check_notional(total_notional)
        if not within_limits:
            logger.warning("rebalance_blocked", total_notional=total_notional)
            return False
        quote = await self.fiat_manager.get_usd_quote("USD/SOL")
        logger.info("rebalance_quote", provider=quote.provider, price=quote.price)
        drift_bps = abs(total_notional) / max(Decimal("1"), quote.price) * Decimal("10000")
        rebalance_needed = drift_bps >= Decimal(threshold_bps)
        logger.info("rebalance_evaluated", drift_bps=drift_bps, needed=rebalance_needed)
        return rebalance_needed
