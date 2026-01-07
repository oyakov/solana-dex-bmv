from __future__ import annotations

from dataclasses import dataclass

import aiohttp

from bot.utils.logging import get_logger

logger = get_logger(__name__)


@dataclass(slots=True)
class HttpClientConfig:
    timeout_seconds: float
    user_agent: str


class HttpClient:
    def __init__(self, config: HttpClientConfig) -> None:
        timeout = aiohttp.ClientTimeout(total=config.timeout_seconds)
        self._session = aiohttp.ClientSession(
            timeout=timeout,
            headers={"User-Agent": config.user_agent},
        )

    async def get_json(self, url: str) -> dict:
        async with self._session.get(url) as response:
            response.raise_for_status()
            return await response.json()

    async def close(self) -> None:
        await self._session.close()
