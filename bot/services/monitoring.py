from __future__ import annotations

from dataclasses import dataclass

from bot.infra.solana_client import SolanaClient
from bot.utils.logging import get_logger

logger = get_logger(__name__)


@dataclass(slots=True)
class MonitoringService:
    solana: SolanaClient

    async def heartbeat(self) -> bool:
        healthy = await self.solana.health()
        logger.info("monitoring_heartbeat", healthy=healthy)
        return healthy
