# Order Grid Theory & Performance Visualization

This document explains the technical logic behind the asymmetric order grid and the performance indicators used to monitor the BMV Eco System Market Making Bot.

## Order Grid Anatomy

The bot operates on an **Asymmetric Asynchronous Grid** centered around a **VWAP Pivot**.

### 1. VWAP Pivot (True Average Price)
The dashed orange line in the visualization represents the **Volume Weighted Average Price (VWAP)**. 
- **Calculation**: $\sum (Price \times Volume) / \sum Volume$
- **Purpose**: It serves as the "magnetic center" for the grid, reflecting the fair market value based on actual trading activity rather than just the last price.

### 2. Asymmetric Channel
The grid is deliberately asymmetric to balance risk and liquidity capture:
- **Buy Side (Green)**: Extends **15% below** the pivot. It uses an exponential volume distribution where order size increases as the price drops further from the pivot to build a strong position during dips.
- **Sell Side (Red)**: Extends **30% above** the pivot. The wider range allows for "letting winners run" and capturing larger price swings on the upside.

### 3. Dynamic Rebalancing
When the price moves outside the defined thresholds (default 1.0%), the grid is recalculated and redeployed to stay centered on the evolving VWAP.

## Performance Indicators (KPIs)

The dashboard tracks several critical metrics to ensure the health and profitability of the trading service:

| Indicator | Description | Target |
| :--- | :--- | :--- |
| **Realized PnL** | Total profit or loss from completed trades (Settled). | Positive trend |
| **Fill Rate** | Percentage of placed orders that are partially or fully filled. | > 70% |
| **Active Depth** | Total USD value of all open orders currently on the book. | Config-dependent |
| **Bundle Latency** | Time taken for Jito bundles to be processed and confirmed. | < 100ms |
| **Sell Rate** | Ratio of sell fills to total fills (indicates market direction & bias). | Variable |

---

![Order Grid Visualization](file:///C:/Users/oyakov/.gemini/antigravity/brain/b6e3528a-a626-4235-9321-416ad41178cd/order_grid_visualization_mockup_1768091558213.png)
*Figure 1: Mockup of the projected order grid visualization against the price graph.*
