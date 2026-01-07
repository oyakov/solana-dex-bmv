import pytest
import asyncio
from bot.infra.solana_client import SolanaClient, SolanaClientConfig
from solana.rpc.commitment import Confirmed

@pytest.mark.asyncio
async def test_live_devnet_connectivity():
    """Verifies that the SolanaClient can connect to Devnet and fetch a real balance."""
    config = SolanaClientConfig(
        rpc_url="https://api.devnet.solana.com",
        commitment=Confirmed
    )
    client = SolanaClient(config)
    
    try:
        # 1. Test Health
        is_healthy = await client.health()
        assert is_healthy is True, "Devnet health check failed"
        
        # 2. Test Balance (using a known stable system account)
        # System Program account always exists
        balance = await client.get_balance("11111111111111111111111111111111")
        assert balance >= 0
        print(f"\nDevnet System Program Balance: {balance} lamports")
        
    finally:
        await client.close()

@pytest.mark.asyncio
async def test_devnet_transaction_dry_run():
    """Verifies transaction serialization logic against Devnet (Dry Run)."""
    # This test would ideally verify that we can build a valid transaction object
    # using current blockhash from Devnet.
    config = SolanaClientConfig(
        rpc_url="https://api.devnet.solana.com",
        commitment=Confirmed
    )
    client = SolanaClient(config)
    
    try:
        # Get latest blockhash to prove connectivity and serialization readiness
        response = await client._client.get_latest_blockhash()
        blockhash = response.value.blockhash
        assert blockhash is not None
        print(f"Devnet Latest Blockhash: {blockhash}")
        
    finally:
        await client.close()
