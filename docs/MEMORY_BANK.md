# Memory Bank: BMV Eco System Market Making Bot

The Memory Bank is the central source of truth for the BMV Market Making Bot project. It provides context, architecture patterns, and progress tracking to ensure consistency and speed in development.

## Core References
- [**Project Brief**](memory-bank/projectBrief.md): High-level mission and strategy.
- [**Product Context**](memory-bank/productContext.md): Business logic and the problems being solved.
- [**System Patterns**](memory-bank/systemPatterns.md): Architecture, design patterns, and technical decisions.
- [**Tech Context**](memory-bank/techContext.md): Tech stack, libraries, and infrastructure.

## Runtime Status
- [**Active Context**](memory-bank/activeContext.md): Current focus, recent changes, and immediate tasks.
- [**Progress**](memory-bank/progress.md): Roadmap status and phase completion tracking.

## Operational Guides
- [README](../README.md): Quick start and execution guide.
- [Conda Setup](CONDA.md): Environment management using Conda.
- [Docker Guide](DOCKER.md): Running the bot in containerized environments.
- [Testing Guide](TESTING.md): Testing strategy and execution instructions.
- [Agent Workflows](WORKFLOWS.md): Available AI agent workflows.

## Technical Documentation
- [Main Requirements](requirements/BMV%20Eco%20System%20Market%20Making%20Bot%20—%20Требования.md): Primary project requirements.
- [Technical Spec](customer/Technical%20Spec.md): Detailed technical specification from the customer.
- [Trading Algorithm](requirements/old/Алгоритм%20динамического%20построения%20торговой%20сетки%20ордеров.md): Mathematical details of the grid builder.

## Visual Assets & Diagrams
- [Structural Architecture (Text)](requirements/диаграмма-структурная.txt): Text-based architecture diagram.
- [Structural Architecture (Image)](requirements/диаграмма-структурная.png)
- [General Concept (Image)](requirements/диаграмма-общий-концепт.png)

## Infrastructure & Deployment
- [Lab Data (Regxa)](deploy/Regxa2core2gig/lab-data.md): Deployment-specific data and labs.

## Archive & Legacy
- [Legacy Requirements Folder](requirements/old/): Contains older versions (v2.4 to v2.6) and original draft documents.

## Core Practices
1. **Jito-First**: Every transaction must be MEV-protected via Jito Bundles.
2. **Asyncio Only**: No blocking calls in the main event loop. Use `run_in_executor` for CPU-heavy tasks.
3. **Structured Logging**: Use `structlog` for all events to facilitate analytics.
4. **Safety First**: Every action must pass through the Circuit Breaker and Risk Manager.
5. **Multi-Wallet Rotation**: Never use a single wallet for all orders to avoid detection and limits.
