# Conda Environment Setup

This project uses **Conda** to manage its Python environment and dependencies, ensuring isolation and reproducibility.

## Prerequisites

- **Conda** (Anaconda or Miniconda) must be installed.
- Access to the `conda` command in your terminal.

## Environment Details

- **Name**: `solana-dex-bmv`
- **Python Version**: `3.11`
- **Main Dependencies**: `solana`, `solders`, `anchorpy`, `structlog`, `rich`, `aiosqlite`.

## Setup Instructions

### 1. Create the Environment

```powershell
conda create --name solana-dex-bmv python=3.11 -y
```

### 2. Activate the Environment

```powershell
conda activate solana-dex-bmv
```

### 3. Install Dependencies

```powershell
# Core dependencies including specific version fixes
pip install pytest-xprocess==0.18.1 py==1.11.0 pytest-asyncio pytest-mock pytest-cov solana solders aiosqlite structlog pyyaml pydantic aiohttp anchorpy-core anchorpy rich pydantic-settings python-dotenv
```

## Running the Bot and Tests

### Run Tests

```powershell
conda run -n solana-dex-bmv python -m pytest
```

### Run Bot

To start the bot in dry-run mode:

```powershell
conda run -n solana-dex-bmv python -m bot.main --dry-run
```

To start the production bot:

```powershell
conda run -n solana-dex-bmv python -m bot.main
```

## Troubleshooting

- **ModuleNotFoundError**: Ensure you have activated the environment with `conda activate solana-dex-bmv`.
- **pytest-xprocess errors**: We use version `0.18.1` to maintain compatibility with `anchorpy`. Do not upgrade it to `1.x` unless the project is migrated.
