from __future__ import annotations
from dataclasses import dataclass
from decimal import Decimal
from bot.utils.logging import get_logger

logger = get_logger(__name__)

@dataclass(slots=True)
class JitoManager:
    base_tip_lamports: int = 5_000_000  # 0.005 SOL
    
    async def calculate_tip(self, network_congestion: str = "normal") -> int:
        """
        Calculates the appropriate Jito tip based on network state.
        In a production app, this would poll Jito's Tip API.
        """
        multipliers = {
            "low": 0.5,
            "normal": 1.0,
            "high": 2.0,
            "extreme": 5.0
        }
        multiplier = multipliers.get(network_congestion, 1.0)
        final_tip = int(self.base_tip_lamports * multiplier)
        logger.info("jito_tip_calculated", congestion=network_congestion, tip=final_tip)
        return final_tip

    def build_bundle(self, transactions: list[str], tip_tx: str) -> list[str]:
        """
        Assembles a list of transactions into a Jito bundle.
        The tip transaction MUST be included in the bundle.
        """
        if not transactions:
            raise ValueError("bundle_must_have_at_least_one_transaction")
            
        # Per Jito docs, the tip transaction can be anywhere, but usually it's last.
        bundle = list(transactions) + [tip_tx]
        
        if len(bundle) > 5:
            # Jito currently limits to 5 transactions per bundle (mostly)
            logger.warning("bundle_size_exceeds_recommended_limit", size=len(bundle))
            
        return bundle
