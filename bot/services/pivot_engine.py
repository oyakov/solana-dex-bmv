from __future__ import annotations

from dataclasses import dataclass
from decimal import Decimal

from bot.domain.models import AssetPosition
from bot.utils.logging import get_logger

logger = get_logger(__name__)


@dataclass(slots=True)
class PivotEngine:
    target_allocation_usd: Decimal
    lookback_days: int = 365
    initial_fade_in_days: int = 30  # Duration of the fade-in period

    async def compute_pivot(
        self,
        positions: list[AssetPosition],
        market_data: list[tuple[Decimal, Decimal]],  # (price, volume)
        days_since_start: int = 0
    ) -> Decimal:
        """
        Computes the Pivot Point using VWAP and applies a linear fade-in adaptation.
        """
        if not market_data:
            logger.warning("no_market_data_for_vwap")
            return self.target_allocation_usd

        # Compute VWAP: Σ(Price * Volume) / Σ(Volume)
        total_value = sum((price * volume for price, volume in market_data), Decimal("0"))
        total_volume = sum((volume for _, volume in market_data), Decimal("0"))

        if total_volume == 0:
            logger.warning("total_volume_is_zero")
            vwap = market_data[-1][0] if market_data else Decimal("0")
        else:
            vwap = total_value / total_volume

        # Linear Fade-ins: transition from current market price to VWAP pivot
        current_price = market_data[-1][0]
        
        if days_since_start < self.initial_fade_in_days:
            fade_ratio = Decimal(days_since_start) / Decimal(self.initial_fade_in_days)
            pivot = (current_price * (Decimal("1") - fade_ratio)) + (vwap * fade_ratio)
            logger.info("fade_in_active", ratio=fade_ratio, pivot=pivot, vwap=vwap)
        else:
            pivot = vwap
            logger.info("fade_in_complete", pivot=pivot)

        total_holdings_usd = sum((position.notional_usd for position in positions), Decimal("0"))
        # The requirements say "Удержание цены токена в заданном восходящем ценовом канале через механизм Pivot Point (VWAP)"
        # This usually means the Pivot is the target price for the grid.
        
        logger.info("pivot_computed", vwap=vwap, current=current_price, final_pivot=pivot)
        return pivot
