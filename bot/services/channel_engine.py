from __future__ import annotations

from dataclasses import dataclass

from bot.utils.logging import get_logger

logger = get_logger(__name__)


@dataclass(slots=True)
class ChannelEngine:
    channels: list[str]

    async def broadcast(self, message: str) -> None:
        for channel in self.channels:
            logger.info("channel_broadcast", channel=channel, message=message)
