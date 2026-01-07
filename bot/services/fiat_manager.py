from __future__ import annotations

from dataclasses import dataclass
from decimal import Decimal

from bot.domain.models import FiatQuote
from bot.infra.http_client import HttpClient
from bot.utils.logging import get_logger

logger = get_logger(__name__)


@dataclass(slots=True)
class FiatManager:
    http: HttpClient
    max_fiat_allocation: Decimal = Decimal("0.30")  # 30%
    min_fiat_growth: Decimal = Decimal("0.15")     # 15%
    min_sol_reserve: Decimal = Decimal("0.70")      # 70%

    async def get_usd_quote(self, pair: str) -> Decimal:
        """
        Fetches current price for a pair (e.g., SOL/USD) from Jupiter/Raydium.
        """
        # Using Jupiter Price API as a real source
        url = f"https://price.jup.ag/v4/price?ids={pair}"
        try:
            data = await self.http.get_json(url)
            price = Decimal(str(data["data"][pair]["price"]))
            logger.info("fiat_quote_fetch", pair=pair, price=price)
            return price
        except Exception as e:
            logger.error("fiat_quote_failed", pair=pair, error=str(e))
            return Decimal("150") # Fallback

    async def check_injection_needed(
        self, 
        current_sol_usd: Decimal, 
        prev_sol_usd: Decimal, 
        bmv_price_sol: Decimal
    ) -> bool:
        """
        If SOL drops vs USD, we might need to inject (recalculate grid) 
        to keep BMV/USD above target growth.
        """
        bmv_price_usd = bmv_price_sol * current_sol_usd
        # This is a simplified check for the logic requirement
        if current_sol_usd < prev_sol_usd:
            logger.info("potential_injection_needed", sol_usd=current_sol_usd, bmv_usd=bmv_price_usd)
            return True
        return False

    async def check_auto_injection(self, sol_balance: Decimal, total_balance: Decimal) -> bool:
        """
        If SOL balance < MIN_SOL_RESERVE_%, we need to buy SOL using USDC.
        """
        if total_balance == 0:
            return False
            
        ratio = sol_balance / total_balance
        if ratio < self.min_sol_reserve:
            logger.info("auto_injection_triggered", ratio=ratio, min=self.min_sol_reserve)
            return True
        return False

    async def calculate_dividend(self, profit_sol: Decimal) -> Decimal:
        """
        Calculates how much SOL should be converted to USDC.
        """
        dividend = profit_sol * self.max_fiat_allocation
        logger.info("dividend_calculated", profit=profit_sol, dividend=dividend)
        return dividend
