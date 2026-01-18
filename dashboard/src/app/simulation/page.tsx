"use client";

import React, { useState } from "react";
import {
    FlaskConical,
    Play,
    TrendingUp,
    TrendingDown,
    Activity,
    Zap,
    Clock,
    RefreshCcw,
    ArrowLeft
} from "lucide-react";
import Link from "next/link";
import { motion } from "framer-motion";
import Sidebar from "../../components/Sidebar";
import { getAuthHeaders } from "../../utils/auth";

interface SimulationOrder {
    price: number;
    amount: number;
    side: "buy" | "sell";
    wallet_index: number;
}

interface SimulationResult {
    scenario_name: string;
    prices: number[];
    projected_orders: SimulationOrder[];
    total_buy_orders: number;
    total_sell_orders: number;
    price_range: { min: number; max: number };
    average_spread: number;
}

type ScenarioType = "upward_saw" | "downward_saw" | "sideways" | "flash_crash" | "pump_and_dump" | "gradual_rise";

export default function SimulationPage() {
    const [scenario, setScenario] = useState<ScenarioType>("upward_saw");
    const [basePrice, setBasePrice] = useState("0.00001");
    const [steps, setSteps] = useState("50");
    const [volatility, setVolatility] = useState("0.05");
    const [result, setResult] = useState<SimulationResult | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const scenarios: { value: ScenarioType; label: string; icon: React.ReactNode; description: string }[] = [
        { value: "upward_saw", label: "Upward Saw", icon: <TrendingUp className="text-green-400" />, description: "Gradual uptrend with pullbacks" },
        { value: "downward_saw", label: "Downward Saw", icon: <TrendingDown className="text-red-400" />, description: "Gradual downtrend with bounces" },
        { value: "sideways", label: "Sideways", icon: <Activity className="text-yellow-400" />, description: "Range-bound movement" },
        { value: "flash_crash", label: "Flash Crash", icon: <Zap className="text-rose-400" />, description: "Sudden price collapse & recovery" },
        { value: "pump_and_dump", label: "Pump & Dump", icon: <TrendingUp className="text-orange-400" />, description: "Sharp rise then selloff" },
        { value: "gradual_rise", label: "Gradual Rise", icon: <TrendingUp className="text-cyan-400" />, description: "Steady upward movement" },
    ];

    const runSimulation = async () => {
        setLoading(true);
        setError(null);
        try {
            const res = await fetch("/api/simulation", {
                method: "POST",
                headers: {
                    ...getAuthHeaders(),
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    scenario,
                    base_price: basePrice,
                    steps: parseInt(steps),
                    volatility: volatility,
                }),
            });

            if (res.status === 401) {
                window.location.href = "/login";
                return;
            }

            if (res.ok) {
                const data = await res.json();
                setResult(data);
            } else {
                setError("Simulation failed");
            }
        } catch (err) {
            console.error("Simulation error:", err);
            setError("Network error");
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="flex h-screen overflow-hidden bg-[#020617] text-slate-100 font-sans selection:bg-cyan-500/30">
            {/* Dynamic Background */}
            <div className="absolute inset-0 overflow-hidden pointer-events-none">
                <div className="absolute -top-[20%] -left-[10%] w-[60%] h-[60%] bg-orange-500/10 blur-[120px] rounded-full animate-pulse" />
                <div className="absolute -bottom-[20%] -right-[10%] w-[50%] h-[50%] bg-purple-500/10 blur-[120px] rounded-full animate-pulse delay-1000" />
            </div>

            {/* Sidebar */}
            <Sidebar />

            {/* Main Content */}
            <main className="flex-1 overflow-y-auto p-8 relative z-10 scrollbar-hide">
                <div className="max-w-[1400px] mx-auto">
                    {/* Header */}
                    <div className="flex flex-col md:flex-row md:items-center justify-between mb-10 gap-4">
                        <div>
                            <div className="flex items-center gap-2 mb-2">
                                <Link href="/" className="p-1 px-2 bg-white/5 rounded-lg border border-white/5 hover:bg-white/10 text-slate-400 hover:text-white transition-all text-[10px] font-black uppercase tracking-widest flex items-center gap-1">
                                    <ArrowLeft size={10} />
                                    Back
                                </Link>
                            </div>
                            <h2 className="text-3xl font-black tracking-tight mb-1">Simulation Lab</h2>
                            <p className="text-slate-400 text-sm flex items-center gap-2">
                                <FlaskConical size={14} className="text-orange-400" />
                                Backtest Market Scenarios
                            </p>
                        </div>
                        <div className="px-5 py-2 bg-orange-500/10 border border-orange-500/20 rounded-full">
                            <span className="text-[10px] font-black uppercase tracking-widest text-orange-400 flex items-center gap-2">
                                <FlaskConical size={10} />
                                Simulation Engine Active
                            </span>
                        </div>
                    </div>

                    <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
                        {/* Configuration Panel */}
                        <div className="glass-panel rounded-[2rem] p-8 border border-white/5">
                            <h3 className="text-xl font-black mb-6 flex items-center gap-3">
                                <div className="w-1.5 h-6 bg-orange-400 rounded-full" />
                                Configuration
                            </h3>

                            {/* Scenario Selection */}
                            <div className="mb-6">
                                <label className="text-[10px] font-black uppercase tracking-widest text-slate-500 block mb-3">Market Scenario</label>
                                <div className="grid grid-cols-2 gap-2">
                                    {scenarios.map((s) => (
                                        <button
                                            key={s.value}
                                            onClick={() => setScenario(s.value)}
                                            className={`p-3 rounded-xl border transition-all text-left ${scenario === s.value
                                                ? 'bg-orange-500/10 border-orange-500/30 text-orange-400'
                                                : 'bg-white/5 border-white/5 text-slate-400 hover:bg-white/10'
                                                }`}
                                        >
                                            <div className="flex items-center gap-2 mb-1">
                                                {s.icon}
                                                <span className="text-xs font-bold">{s.label}</span>
                                            </div>
                                            <p className="text-[9px] text-slate-500">{s.description}</p>
                                        </button>
                                    ))}
                                </div>
                            </div>

                            {/* Parameters */}
                            <div className="space-y-4 mb-6">
                                <div>
                                    <label className="text-[10px] font-black uppercase tracking-widest text-slate-500 block mb-2">Base Price (SOL)</label>
                                    <input
                                        type="text"
                                        value={basePrice}
                                        onChange={(e) => setBasePrice(e.target.value)}
                                        className="w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-sm font-mono focus:outline-none focus:border-orange-500/50"
                                    />
                                </div>
                                <div>
                                    <label className="text-[10px] font-black uppercase tracking-widest text-slate-500 block mb-2">Steps</label>
                                    <input
                                        type="number"
                                        value={steps}
                                        onChange={(e) => setSteps(e.target.value)}
                                        className="w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-sm font-mono focus:outline-none focus:border-orange-500/50"
                                    />
                                </div>
                                <div>
                                    <label className="text-[10px] font-black uppercase tracking-widest text-slate-500 block mb-2">Volatility</label>
                                    <input
                                        type="text"
                                        value={volatility}
                                        onChange={(e) => setVolatility(e.target.value)}
                                        className="w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-sm font-mono focus:outline-none focus:border-orange-500/50"
                                    />
                                </div>
                            </div>

                            {/* Run Button */}
                            <button
                                onClick={runSimulation}
                                disabled={loading}
                                className="w-full py-4 bg-gradient-to-r from-orange-500 to-rose-500 rounded-2xl font-black uppercase tracking-widest text-sm flex items-center justify-center gap-2 hover:opacity-90 transition-opacity disabled:opacity-50"
                            >
                                {loading ? (
                                    <RefreshCcw size={16} className="animate-spin" />
                                ) : (
                                    <Play size={16} />
                                )}
                                {loading ? "Running..." : "Run Simulation"}
                            </button>

                            {error && (
                                <p className="text-red-400 text-sm mt-4 text-center">{error}</p>
                            )}
                        </div>

                        {/* Results Panel */}
                        <div className="lg:col-span-2 glass-panel rounded-[2rem] p-8 border border-white/5">
                            <h3 className="text-xl font-black mb-6 flex items-center gap-3">
                                <div className="w-1.5 h-6 bg-cyan-400 rounded-full" />
                                Simulation Results
                            </h3>

                            {!result ? (
                                <div className="flex items-center justify-center h-64 text-slate-500">
                                    <div className="text-center">
                                        <FlaskConical size={48} className="mx-auto mb-4 opacity-30" />
                                        <p className="font-medium">Configure and run a simulation to see results</p>
                                    </div>
                                </div>
                            ) : (
                                <div className="space-y-6">
                                    {/* Summary Cards */}
                                    <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                                        <div className="bg-white/5 rounded-xl p-4">
                                            <p className="text-[10px] font-black uppercase tracking-widest text-slate-500 mb-1">Scenario</p>
                                            <p className="text-lg font-bold text-slate-200">{result.scenario_name}</p>
                                        </div>
                                        <div className="bg-white/5 rounded-xl p-4">
                                            <p className="text-[10px] font-black uppercase tracking-widest text-slate-500 mb-1">Buy Orders</p>
                                            <p className="text-lg font-bold text-green-400">{result.total_buy_orders}</p>
                                        </div>
                                        <div className="bg-white/5 rounded-xl p-4">
                                            <p className="text-[10px] font-black uppercase tracking-widest text-slate-500 mb-1">Sell Orders</p>
                                            <p className="text-lg font-bold text-red-400">{result.total_sell_orders}</p>
                                        </div>
                                        <div className="bg-white/5 rounded-xl p-4">
                                            <p className="text-[10px] font-black uppercase tracking-widest text-slate-500 mb-1">Avg Spread</p>
                                            <p className="text-lg font-bold text-cyan-400">{(result.average_spread * 100).toFixed(2)}%</p>
                                        </div>
                                    </div>

                                    {/* Price Range */}
                                    <div className="bg-white/5 rounded-xl p-4">
                                        <p className="text-[10px] font-black uppercase tracking-widest text-slate-500 mb-3">Price Range</p>
                                        <div className="flex items-center gap-4">
                                            <div className="flex-1 h-2 bg-white/10 rounded-full relative">
                                                <div
                                                    className="absolute h-full bg-gradient-to-r from-green-500 to-red-500 rounded-full"
                                                    style={{ width: "100%" }}
                                                />
                                            </div>
                                            <div className="flex gap-4 text-sm font-mono">
                                                <span className="text-green-400">{result.price_range.min.toFixed(8)}</span>
                                                <span className="text-slate-500">→</span>
                                                <span className="text-red-400">{result.price_range.max.toFixed(8)}</span>
                                            </div>
                                        </div>
                                    </div>

                                    {/* Orders Table */}
                                    <div>
                                        <p className="text-[10px] font-black uppercase tracking-widest text-slate-500 mb-3">Projected Orders ({result.projected_orders.length})</p>
                                        <div className="max-h-64 overflow-y-auto">
                                            <table className="w-full text-sm">
                                                <thead className="sticky top-0 bg-[#020617]">
                                                    <tr className="border-b border-white/5">
                                                        <th className="text-left py-2 text-slate-500 font-medium">Side</th>
                                                        <th className="text-right py-2 text-slate-500 font-medium">Price</th>
                                                        <th className="text-right py-2 text-slate-500 font-medium">Amount</th>
                                                        <th className="text-right py-2 text-slate-500 font-medium">Wallet</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {result.projected_orders.slice(0, 20).map((order, idx) => (
                                                        <motion.tr
                                                            key={idx}
                                                            initial={{ opacity: 0 }}
                                                            animate={{ opacity: 1 }}
                                                            transition={{ delay: idx * 0.02 }}
                                                            className="border-b border-white/5"
                                                        >
                                                            <td className="py-2">
                                                                <span className={`px-2 py-0.5 rounded text-[10px] font-black uppercase ${order.side === 'buy' ? 'bg-green-500/20 text-green-400' : 'bg-red-500/20 text-red-400'}`}>
                                                                    {order.side}
                                                                </span>
                                                            </td>
                                                            <td className="text-right py-2 font-mono text-slate-300">{order.price.toFixed(8)}</td>
                                                            <td className="text-right py-2 font-mono text-slate-300">{order.amount.toFixed(2)}</td>
                                                            <td className="text-right py-2 font-mono text-slate-500">#{order.wallet_index}</td>
                                                        </motion.tr>
                                                    ))}
                                                </tbody>
                                            </table>
                                            {result.projected_orders.length > 20 && (
                                                <p className="text-center text-slate-500 text-xs mt-2">
                                                    + {result.projected_orders.length - 20} more orders
                                                </p>
                                            )}
                                        </div>
                                    </div>
                                </div>
                            )}
                        </div>
                    </div>
                </div>
            </main>
        </div>
    );
}
