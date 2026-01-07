from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import aiosqlite

from bot.utils.logging import get_logger

logger = get_logger(__name__)


@dataclass(slots=True)
class DatabaseConfig:
    path: Path


class Database:
    def __init__(self, config: DatabaseConfig) -> None:
        self._path = config.path
        self._conn: aiosqlite.Connection | None = None

    async def connect(self) -> None:
        self._conn = await aiosqlite.connect(self._path)
        await self._conn.execute(
            """
            CREATE TABLE IF NOT EXISTS bot_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            """
        )
        await self._conn.commit()

    async def set_state(self, key: str, value: str) -> None:
        if self._conn is None:
            raise RuntimeError("Database not connected")
        await self._conn.execute(
            """
            INSERT INTO bot_state (key, value, updated_at)
            VALUES (?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = CURRENT_TIMESTAMP
            """,
            (key, value),
        )
        await self._conn.commit()

    async def get_state(self, key: str) -> str | None:
        if self._conn is None:
            raise RuntimeError("Database not connected")
        async with self._conn.execute(
            "SELECT value FROM bot_state WHERE key = ?",
            (key,),
        ) as cursor:
            row = await cursor.fetchone()
            return row[0] if row else None

    async def close(self) -> None:
        if self._conn is not None:
            await self._conn.close()
            self._conn = None
