from decimal import Decimal
from bot.services.risk_manager import RiskManager
from bot.domain.models import RiskLimits

def test_check_notional_within_limits():
    limits = RiskLimits(max_notional_usd=Decimal("1000"))
    manager = RiskManager(limits=limits)
    assert manager.check_notional(Decimal("500")) is True
    assert manager.check_notional(Decimal("1000")) is True
    assert manager.check_notional(Decimal("-500")) is True
    assert manager.check_notional(Decimal("-1000")) is True

def test_check_notional_outside_limits():
    limits = RiskLimits(max_notional_usd=Decimal("1000"))
    manager = RiskManager(limits=limits)
    assert manager.check_notional(Decimal("1000.01")) is False
    assert manager.check_notional(Decimal("2000")) is False
    assert manager.check_notional(Decimal("-1000.01")) is False
