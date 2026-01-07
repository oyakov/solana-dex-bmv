from __future__ import annotations

from dataclasses import dataclass

from solana.rpc.async_api import AsyncClient
from solana.rpc.commitment import Commitment
from solders.pubkey import Pubkey

from bot.utils.logging import get_logger

logger = get_logger(__name__)


@dataclass(slots=True)
class SolanaClientConfig:
    rpc_url: str
    commitment: Commitment


class SolanaClient:
    def __init__(self, config: SolanaClientConfig) -> None:
        self._client = AsyncClient(config.rpc_url, commitment=config.commitment)

    async def get_balance(self, owner: str) -> int:
        pubkey = Pubkey.from_string(owner)
        response = await self._client.get_balance(pubkey)
        return int(response.value)

    async def health(self) -> bool:
        response = await self._client.is_blockhash_valid(
            (await self._client.get_latest_blockhash()).value.blockhash
        )
        return bool(response.value)

    async def close(self) -> None:
        await self._client.close()
