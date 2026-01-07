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

    async def get_usd_quote(self, pair: str) -> FiatQuote:
        # Placeholder for a real FX feed; using a stable mock response.
        logger.info("fiat_quote_fetch", pair=pair)
        return FiatQuote(provider="mock", pair=pair, price=Decimal("150"))
