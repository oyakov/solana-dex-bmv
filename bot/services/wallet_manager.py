from __future__ import annotations

from dataclasses import dataclass

from bot.domain.models import WalletSnapshot
from bot.infra.solana_client import SolanaClient
from bot.utils.logging import get_logger

logger = get_logger(__name__)


import random
from solders.keypair import Keypair

@dataclass(slots=True)
class WalletManager:
    solana: SolanaClient
    keypairs: list[Keypair]

    async def get_next_wallet(self) -> Keypair:
        """
        Returns a random wallet from the pool to avoid on-chain patterns.
        """
        if not self.keypairs:
            raise ValueError("no_keypairs_configured")
            
        wallet = random.choice(self.keypairs)
        logger.info("wallet_selected", address=str(wallet.pubkey()))
        return wallet

    async def snapshot(self, owner: str) -> WalletSnapshot:
        balance = await self.solana.get_balance(owner)
        # In a real implementation, we would also fetch token balances
        logger.info("wallet_snapshot", owner=owner, balance_lamports=balance)
        return WalletSnapshot(owner=owner, balance_lamports=balance, token_balances={})

    async def get_total_sol_balance(self) -> Decimal:
        """
        Aggregates balance across all managed wallets.
        """
        total_lamports = 0
        for kp in self.keypairs:
            balance = await self.solana.get_balance(str(kp.pubkey()))
            total_lamports += balance
        
        total_sol = Decimal(total_lamports) / Decimal("1000000000")
        logger.info("total_sol_balance", sol=total_sol)
        return total_sol
