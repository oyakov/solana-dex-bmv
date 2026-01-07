from unittest.mock import Mock

from solana_dex_bmv.fiat import FiatManager, FiatPolicy


def test_fiat_manager_buffers_and_injections():
    policy = FiatPolicy(buffer_ratio=0.1, min_buffer=100, injection_amount=250)
    manager = FiatManager(policy)

    assert manager.required_buffer(2000) == 200
    assert manager.needs_injection(150, 2000) is True
    assert manager.injection_needed(150, 2000) == 250
    assert manager.needs_injection(300, 2000) is False
    assert manager.injection_needed(300, 2000) == 0


def test_fiat_manager_rebalance_uses_clients():
    policy = FiatPolicy(buffer_ratio=0.1, min_buffer=100, injection_amount=250)
    manager = FiatManager(policy)

    rpc_client = Mock()
    jito_client = Mock()
    rpc_client.get_usd_balance.return_value = 150

    injected = manager.rebalance(rpc_client, jito_client, target_exposure=2000)

    assert injected == 250
    rpc_client.get_usd_balance.assert_called_once_with()
    jito_client.send_injection.assert_called_once_with(250)
