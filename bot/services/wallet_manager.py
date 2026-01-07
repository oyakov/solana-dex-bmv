from __future__ import annotations

from dataclasses import dataclass

from bot.domain.models import WalletSnapshot
from bot.infra.solana_client import SolanaClient
from bot.utils.logging import get_logger

logger = get_logger(__name__)


@dataclass(slots=True)
class WalletManager:
    solana: SolanaClient

    async def snapshot(self, owner: str) -> WalletSnapshot:
        balance = await self.solana.get_balance(owner)
        logger.info("wallet_snapshot", owner=owner, balance_lamports=balance)
        return WalletSnapshot(owner=owner, balance_lamports=balance, token_balances={})
