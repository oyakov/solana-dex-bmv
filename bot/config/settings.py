from __future__ import annotations

from pathlib import Path
from typing import Literal

from pydantic import BaseModel, Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class SolanaSettings(BaseModel):
    rpc_url: str = "https://api.mainnet-beta.solana.com"
    commitment: Literal["processed", "confirmed", "finalized"] = "confirmed"
    default_fee_payer: str | None = None


class DatabaseSettings(BaseModel):
    path: Path = Field(default=Path("bot_state.sqlite"))


class HttpSettings(BaseModel):
    timeout_seconds: float = 10.0
    user_agent: str = "solana-dex-bmv-bot/0.1"


class LoggingSettings(BaseModel):
    level: str = "INFO"


class StrategySettings(BaseModel):
    pivot_interval_seconds: float = 30.0
    grid_spacing_bps: int = 25
    rebalance_threshold_bps: int = 50


class BotSettings(BaseSettings):
    model_config = SettingsConfigDict(env_prefix="BOT_", env_nested_delimiter="__")

    solana: SolanaSettings = SolanaSettings()
    database: DatabaseSettings = DatabaseSettings()
    http: HttpSettings = HttpSettings()
    logging: LoggingSettings = LoggingSettings()
    strategy: StrategySettings = StrategySettings()

    run_mode: Literal["paper", "live"] = "paper"
    metrics_port: int = 9100


settings = BotSettings()
