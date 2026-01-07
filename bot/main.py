from __future__ import annotations

import asyncio
from decimal import Decimal

from rich.console import Console

from bot.config.settings import settings
from bot.domain.models import AssetPosition, RiskLimits
from bot.infra.database import Database, DatabaseConfig
from bot.infra.http_client import HttpClient, HttpClientConfig
from bot.infra.solana_client import SolanaClient, SolanaClientConfig
from bot.services.channel_engine import ChannelEngine
from bot.services.control_panel import ControlPanel
from bot.services.fiat_manager import FiatManager
from bot.services.grid_builder import GridBuilder
from bot.services.monitoring import MonitoringService
from bot.services.order_manager import OrderManager
from bot.services.pivot_engine import PivotEngine
from bot.services.rent_recovery import RentRecovery
from bot.services.rebalance_orchestrator import RebalanceOrchestrator
from bot.services.risk_manager import RiskManager
from bot.services.wallet_manager import WalletManager
from bot.utils.logging import configure_logging, get_logger

logger = get_logger(__name__)


async def run_once() -> None:
    configure_logging(settings.logging.level)
    console = Console()

    solana = SolanaClient(
        SolanaClientConfig(
            rpc_url=settings.solana.rpc_url,
            commitment=settings.solana.commitment,
        )
    )
    http = HttpClient(
        HttpClientConfig(
            timeout_seconds=settings.http.timeout_seconds,
            user_agent=settings.http.user_agent,
        )
    )
    database = Database(DatabaseConfig(path=settings.database.path))
    await database.connect()

    wallet_manager = WalletManager(solana=solana, keypairs=[])
    pivot_engine = PivotEngine(target_allocation_usd=Decimal("1000"))
    grid_builder = GridBuilder(
        orders_per_side=settings.strategy.orders_per_side,
        buy_channel_width=settings.strategy.buy_channel_width,
        sell_channel_width=settings.strategy.sell_channel_width,
    )
    order_manager = OrderManager(open_orders={})
    risk_manager = RiskManager(limits=RiskLimits())
    fiat_manager = FiatManager(http=http)
    rebalance_orchestrator = RebalanceOrchestrator(
        fiat_manager=fiat_manager, risk_manager=risk_manager
    )
    channel_engine = ChannelEngine(channels=["ops", "trading"])
    monitoring = MonitoringService(solana=solana)
    rent_recovery = RentRecovery(solana=solana)
    control_panel = ControlPanel(console=console)

    control_panel.announce("Solana DEX BMV bot starting")
    await channel_engine.broadcast("bot-online")

    snapshot = await wallet_manager.snapshot(
        settings.solana.default_fee_payer or "11111111111111111111111111111111"
    )
    await rent_recovery.sweep(snapshot.owner)
    positions = [AssetPosition(symbol="SOL", quantity=Decimal("2"), notional_usd=Decimal("300"))]
    pivot = await pivot_engine.compute_pivot(
        positions, market_data=[(Decimal("155"), Decimal("1000"))]
    )
    await database.set_state("pivot", str(pivot))

    grid = await grid_builder.build(mid_price=Decimal("150"), total_size=Decimal("0.1"))
    await order_manager.place_grid(grid)

    await rebalance_orchestrator.evaluate(positions, settings.strategy.rebalance_threshold_bps)
    await monitoring.heartbeat()

    await order_manager.cancel_all()

    await http.close()
    await solana.close()
    await database.close()


def main() -> None:
    asyncio.run(run_once())


if __name__ == "__main__":
    main()
