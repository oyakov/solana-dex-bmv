import pytest
import os
import tempfile
from pathlib import Path
from bot.infra.database import Database, DatabaseConfig

@pytest.mark.asyncio
async def test_sqlite_state_persistence():
    """Verifies that state is correctly saved and retrieved from SQLite."""
    with tempfile.NamedTemporaryFile(suffix=".sqlite", delete=False) as tmp:
        db_path = Path(tmp.name)
    
    try:
        config = DatabaseConfig(path=db_path)
        db = Database(config)
        await db.connect()
        
        # 1. Set State
        test_key = "test_pivot"
        test_value = "105.5"
        await db.set_state(test_key, test_value)
        
        # 2. Close and Reconnect (simulate crash/restart)
        await db.close()
        
        db2 = Database(config)
        await db2.connect()
        
        # 3. Retrieve State
        recovered_value = await db2.get_state(test_key)
        assert recovered_value == test_value
        
        # 4. Update State
        new_value = "110.0"
        await db2.set_state(test_key, new_value)
        recovered_updated = await db2.get_state(test_key)
        assert recovered_updated == new_value
        
        await db2.close()
        
    finally:
        if db_path.exists():
            os.remove(db_path)

@pytest.mark.asyncio
async def test_high_frequency_updates():
    """Verifies database stability under concurrent-like updates."""
    with tempfile.NamedTemporaryFile(suffix=".sqlite", delete=False) as tmp:
        db_path = Path(tmp.name)
        
    try:
        db = Database(DatabaseConfig(path=db_path))
        await db.connect()
        
        key = "perf_test"
        for i in range(100):
            await db.set_state(key, str(i))
            
        final_val = await db.get_state(key)
        assert final_val == "99"
        
        await db.close()
    finally:
        if db_path.exists():
            os.remove(db_path)
