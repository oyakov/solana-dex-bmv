from __future__ import annotations

from dataclasses import dataclass

from rich.console import Console

from bot.utils.logging import get_logger

logger = get_logger(__name__)


@dataclass(slots=True)
class ControlPanel:
    console: Console

    def announce(self, message: str) -> None:
        self.console.print(f"[bold green]{message}[/bold green]")
        logger.info("control_panel_announce", message=message)
