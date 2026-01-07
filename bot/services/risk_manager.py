from __future__ import annotations

from dataclasses import dataclass
from decimal import Decimal

from bot.domain.models import RiskLimits
from bot.utils.logging import get_logger

logger = get_logger(__name__)


@dataclass(slots=True)
class RiskManager:
    limits: RiskLimits

    def check_notional(self, total_notional: Decimal) -> bool:
        within = abs(total_notional) <= self.limits.max_notional_usd
        logger.info(
            "risk_check",
            total_notional=total_notional,
            max_notional=self.limits.max_notional_usd,
            within=within,
        )
        return within
