"use client";

import React from "react";
import {
    ResponsiveContainer,
    ComposedChart,
    Line,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    Scatter,
    Cell,
    Area,
} from "recharts";

interface Order {
    price: number;
    size: number;
    side: "buy" | "sell";
}

interface PricePoint {
    timestamp: number;
    price: number;
}

interface SimulationChartProps {
    priceHistory: PricePoint[];
    orders: Order[];
}

export default function SimulationChart({ priceHistory, orders }: SimulationChartProps) {
    // Combine price history and orders into a format suitable for ComposedChart
    // We want the price line to be continuous, and orders to be markers at specific prices.
    // For visualization, we'll map orders to the nearest timestamp or just show them as a separate scatter layer

    // Sort orders by price for easier visualization if needed, but scatter usually takes x/y
    const buyOrders = orders.filter(o => o.side === "buy");
    const sellOrders = orders.filter(o => o.side === "sell");

    const CustomTooltip = ({ active, payload, label }: any) => {
        if (active && payload && payload.length) {
            const data = payload[0].payload;
            return (
                <div className="bg-slate-900/90 border border-white/10 p-3 rounded-lg backdrop-blur-md shadow-2xl">
                    <p className="text-[10px] font-black uppercase text-slate-500 mb-1">
                        Step {label}
                    </p>
                    <p className="text-sm font-mono text-cyan-400">
                        Price: {data.price.toFixed(8)}
                    </p>
                    {data.isOrder && (
                        <div className="mt-2 pt-2 border-t border-white/5">
                            <span className={`text-[10px] font-black uppercase ${data.side === 'buy' ? 'text-green-400' : 'text-red-400'}`}>
                                {data.side} Order
                            </span>
                            <p className="text-[10px] text-slate-300">Size: {data.size.toFixed(4)}</p>
                        </div>
                    )}
                </div>
            );
        }
        return null;
    };

    // To show orders on the chart, we need to associate them with an index or timestamp
    // Since projected_grids is per step, but the user wants to see "how it looks"
    // we'll visualize the price history and overlay orders.

    // For the simulation result, projected_grids[last] matches the "final" state
    // But the user might want to see the "density" of orders.

    return (
        <div className="h-80 w-full mt-6 mb-10">
            <ResponsiveContainer width="100%" height="100%">
                <ComposedChart data={priceHistory} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
                    <defs>
                        <linearGradient id="priceGradient" x1="0" y1="0" x2="0" y2="1">
                            <stop offset="5%" stopColor="#22d3ee" stopOpacity={0.3} />
                            <stop offset="95%" stopColor="#22d3ee" stopOpacity={0} />
                        </linearGradient>
                    </defs>
                    <CartesianGrid strokeDasharray="3 3" stroke="#ffffff08" vertical={false} />
                    <XAxis
                        dataKey="timestamp"
                        hide
                    />
                    <YAxis
                        domain={['auto', 'auto']}
                        orientation="right"
                        tick={{ fill: '#64748b', fontSize: 10, fontFamily: 'monospace' }}
                        tickFormatter={(val) => val.toFixed(8)}
                        axisLine={false}
                        tickLine={false}
                    />
                    <Tooltip content={<CustomTooltip />} />

                    <Area
                        type="monotone"
                        dataKey="price"
                        stroke="#22d3ee"
                        strokeWidth={2}
                        fillOpacity={1}
                        fill="url(#priceGradient)"
                        isAnimationActive={true}
                    />

                    {/* Overlay Buy Orders */}
                    <Scatter
                        name={t("buyOrders")}
                        data={buyOrders.map(o => ({ ...o, timestamp: priceHistory[priceHistory.length - 1]?.timestamp || 0 }))}
                        fill="#4ade80"
                    >
                        {buyOrders.map((entry, index) => (
                            <Cell
                                key={`cell-buy-${index}`}
                                fill="#4ade80"
                                r={Math.min(8, 2 + entry.size * 2)} // Size reflects liquidity
                                fillOpacity={0.6}
                            />
                        ))}
                    </Scatter>

                    {/* Overlay Sell Orders */}
                    <Scatter
                        name={t("sellOrders")}
                        data={sellOrders.map(o => ({ ...o, timestamp: priceHistory[priceHistory.length - 1]?.timestamp || 0 }))}
                        fill="#f87171"
                    >
                        {sellOrders.map((entry, index) => (
                            <Cell
                                key={`cell-sell-${index}`}
                                fill="#f87171"
                                r={Math.min(8, 2 + entry.size * 2)} // Size reflects liquidity
                                fillOpacity={0.6}
                            />
                        ))}
                    </Scatter>
                </ComposedChart>
            </ResponsiveContainer>
        </div>
    );
}
