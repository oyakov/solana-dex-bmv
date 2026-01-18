# BMV Dashboard - Playwright E2E Tests

Comprehensive browser automation tests for the BMV Dashboard using Playwright.

## Test Coverage

### 1. `dashboard.e2e.spec.js` - UI Tests (~50 tests)
- **Authentication**: Login/logout, invalid password, session protection
- **Sidebar Navigation**: All menu items, page navigation, consistency
- **Command Center**: Stat cards, order book, system status
- **Latency Report**: Service cards, latency chart, status indicators
- **Simulation Lab**: Scenarios, configuration, results panel
- **Wallet Swarm**: Wallet cards, inject button, status indicators
- **Token Holders**: Metrics, holders table, distribution chart
- **Responsive Design**: Mobile, tablet, desktop viewports
- **Visual Regression**: Full-page screenshots for all pages

### 2. `api.e2e.spec.js` - API Tests (~15 tests)
- **Endpoint Tests**: /api/stats, /api/latency, /api/holders, /api/wallets
- **Auth Tests**: Token validation, rejection of invalid tokens
- **Data Validation**: Numeric values, percentage ranges
- **Performance Tests**: Response time validation

## Quick Start

```bash
# Install dependencies
npm install

# Run all tests
npm test

# Run with browser visible
npm run test:headed

# Run with Playwright UI
npm run test:ui
```

## Available Scripts

| Command | Description |
|---------|-------------|
| `npm test` | Run all tests in headless mode |
| `npm run test:headed` | Run with browser visible |
| `npm run test:ui` | Open Playwright Test UI |
| `npm run test:dashboard` | Run only dashboard UI tests |
| `npm run test:api` | Run only API tests |
| `npm run test:chrome` | Run only in Chrome |
| `npm run test:firefox` | Run only in Firefox |
| `npm run test:mobile` | Run mobile viewport tests |
| `npm run report` | Open HTML test report |

## Prerequisites

1. Dashboard must be running at `http://localhost`
2. Bot backend must be running
3. Password must be `admin123` (or update in test files)

## Test Results

After running tests:
- HTML Report: `test-results/html-report/`
- JSON Results: `test-results/results.json`
- Screenshots: `screenshots/`
- Failure Artifacts: `test-results/artifacts/`

## Configuration

Edit `playwright.config.js` to modify:
- Browser selection
- Timeout values
- Base URL
- Screenshot/video settings
