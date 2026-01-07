from __future__ import annotations

from dataclasses import dataclass

from bot.infra.solana_client import SolanaClient
from bot.utils.logging import get_logger

logger = get_logger(__name__)


@dataclass(slots=True)
class RentRecovery:
    solana: SolanaClient

    async def sweep(self, owner: str) -> None:
        balance = await self.solana.get_balance(owner)
        logger.info("rent_recovery_checked", owner=owner, balance_lamports=balance)
