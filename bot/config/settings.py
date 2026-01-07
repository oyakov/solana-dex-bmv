from __future__ import annotations

from decimal import Decimal
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
    orders_per_side: int = 16
    buy_channel_width: Decimal = Decimal("0.15")
    sell_channel_width: Decimal = Decimal("0.30")
    lookback_days: int = 365
    initial_fade_in_days: int = 30


class JitoSettings(BaseModel):
    enabled: bool = False
    api_url: str = "https://mainnet.block-engine.jito.wtf/api/v1/bundles"
    tip_lamports: int = 5000000  # 0.005 SOL


class ManagementSettings(BaseModel):
    max_fiat_allocation: Decimal = Decimal("0.30")
    min_fiat_growth: Decimal = Decimal("0.15")
    min_sol_reserve: Decimal = Decimal("0.70")
    rent_recovery_interval_hours: int = 1


class BotSettings(BaseSettings):
    model_config = SettingsConfigDict(
        env_prefix="BOT_", 
        env_nested_delimiter="__",
        env_file=".env",
        extra="ignore"
    )

    solana: SolanaSettings = SolanaSettings()
    database: DatabaseSettings = DatabaseSettings()
    http: HttpSettings = HttpSettings()
    logging: LoggingSettings = LoggingSettings()
    strategy: StrategySettings = StrategySettings()
    jito: JitoSettings = JitoSettings()
    management: ManagementSettings = ManagementSettings()

    run_mode: Literal["paper", "live"] = "paper"
    metrics_port: int = 9100


settings = BotSettings()
