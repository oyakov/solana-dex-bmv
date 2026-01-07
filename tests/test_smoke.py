from bot.config.settings import settings


def test_settings_defaults() -> None:
    assert settings.run_mode in {"paper", "live"}
