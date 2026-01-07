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
        try:
            response = await self._client.get_version()
            return response.value is not None
        except Exception as e:
            logger.error("health_check_failed", error=str(e))
            return False

    async def send_bundle(self, transactions: list[str], jito_api_url: str) -> str:
        """
        Sends a bundle of serialized transactions to Jito's Block Engine.
        """
        payload = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendBundle",
            "params": [transactions]
        }
        
        logger.info("sending_jito_bundle", count=len(transactions), api=jito_api_url)
        # Using a simplified aiohttp call directly if not using the main http_client
        # In a real app, we'd reuse the HttpClient wrapper.
        import aiohttp
        async with aiohttp.ClientSession() as session:
            async with session.post(jito_api_url, json=payload) as response:
                result = await response.json()
                if "error" in result:
                    logger.error("jito_bundle_error", error=result["error"])
                    raise Exception(f"Jito error: {result['error']}")
                bundle_id = result["result"]
                logger.info("jito_bundle_sent", bundle_id=bundle_id)
                return str(bundle_id)

    async def close(self) -> None:
        await self._client.close()
