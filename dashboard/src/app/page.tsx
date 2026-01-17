"use client";

import React, { useState, useEffect, useMemo } from "react";
import {
  Zap,
  Shield,
  RefreshCcw,
  Activity,
  Wallet,
  TrendingUp,
  LayoutDashboard,
  Settings,
  Flame,
  Globe,
  Database,
  BarChart3,
  Clock,
  Users,
  Scale,
  ShieldCheck,
  TrendingDown,
  LogOut
} from "lucide-react";
import {
  ReferenceArea
} from "recharts";
import D3AreaChart from "../components/D3AreaChart";
import D3DepthChart from "../components/D3DepthChart";
import Sidebar from "../components/Sidebar";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { motion, AnimatePresence } from "framer-motion";
import { getAuthHeaders, logout } from "../utils/auth";


interface PriceTick {
  timestamp: number;
  asset_price: string;
  sol_price: string;
}

export default function Dashboard() {
  const [stats, setStats] = useState({
    pivot_price: "0.00",
    buy_channel_width: "0.00",
    sell_channel_width: "0.00",
    active_wallets: 0,
    kill_switch_active: false,
    total_sol_balance: 0,
    total_usdc_balance: 0,
    spread_bps: 0,
    imbalance_index: 0,
    top_holders_percent: 0,
    safe_haven_index: 1.0,
    support_50: "0.00",
    support_90: "0.00",
    resistance_50: "0.00",
    resistance_90: "0.00",
    bids: [] as { price: number; size: number }[],
    asks: [] as { price: number; size: number }[],
  });
  const [history, setHistory] = useState<PriceTick[]>([]);
  const [loading, setLoading] = useState(true);

  const [mounted, setMounted] = useState(false);
  const [time, setTime] = useState("");

  const chartData = useMemo(() => {
    if (history.length === 0) return [];
    return history
      .filter(tick => parseFloat(tick.asset_price) > 0) // Filter out zero prices
      .map(tick => ({
        time: new Date(tick.timestamp * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
        asset: parseFloat(tick.asset_price),
        sol: parseFloat(tick.sol_price),
      }));
  }, [history]);

  const formatPrice = (val: string | number) => {
    if (!mounted) return "0.000000";
    const num = Number(val);

    // If stats price is 0, attempt to fallback to last history price
    let displayNum = num;
    if (displayNum === 0 && chartData.length > 0) {
      displayNum = chartData[chartData.length - 1].asset;
    }

    if (isNaN(displayNum) || displayNum === 0) return "0.000000";

    if (displayNum < 0.1) {
      // Use toFixed(9) and parseFloat to trim trailing zeros
      return parseFloat(displayNum.toFixed(9)).toString();
    }
    return displayNum.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 6 });
  };

  useEffect(() => {
    setMounted(true);
    const updateTime = () => {
      setTime(new Date().toLocaleTimeString([], { hour12: false }));
    };
    updateTime();
    const interval = setInterval(updateTime, 1000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    const fetchData = async () => {
      try {
        // In local dev, we might need to point to the server IP or use a proxy
        const statsRes = await fetch(`/api/stats`, {
          headers: getAuthHeaders(),
        });

        if (statsRes.status === 401) {
          window.location.href = "/login";
          return;
        }

        if (statsRes.ok) {
          const data = await statsRes.json();
          setStats(prev => ({ ...prev, ...data }));
        }

        const historyRes = await fetch(`/api/history`, {
          headers: getAuthHeaders(),
        });

        if (historyRes.status === 401) {
          window.location.href = "/login";
          return;
        }

        if (historyRes.ok) {
          const data = await historyRes.json();
          setHistory(data);
        }
      } catch (error) {
        console.error("Failed to fetch dashboard data:", error);
      } finally {
        setLoading(false);
      }
    };

    fetchData();
    const interval = setInterval(fetchData, 5000); // Poll every 5s instead of 1s
    return () => clearInterval(interval);
  }, []);

  const handleControl = async (action: string) => {
    try {
      await fetch(`/api/control`, {
        method: "POST",
        headers: getAuthHeaders(),
        body: JSON.stringify({ action }),
      });

      alert(`Action ${action} triggered successfully`);
    } catch (error) {
      console.error("Control action failed:", error);
    }
  };

  const yAxisFormatter = (v: number) => {
    if (v === 0) return "0.0";
    if (v < 0.1) {
      // Remove trailing zeros for high-precision small prices
      return parseFloat(v.toFixed(9)).toString();
    }
    return v.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 4 });
  };
  const chartMargin = { top: 40, right: 30, bottom: 40, left: 100 };

  return (
    <div className="flex h-screen overflow-hidden bg-[#020617] text-slate-100 font-sans selection:bg-cyan-500/30">
      {/* Dynamic Background */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        <div className="absolute -top-[20%] -left-[10%] w-[60%] h-[60%] bg-cyan-500/10 blur-[120px] rounded-full animate-pulse" />
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
              <h2 className="text-3xl font-black tracking-tight mb-1">Trading Command Center</h2>
              <p className="text-slate-400 text-sm flex items-center gap-2">
                <Globe size={14} className="text-cyan-400" />
                Live from Solana Mainnet Beta
              </p>
            </div>
            <div className="flex items-center gap-3">
              <div className="flex items-center gap-2 px-4 py-2 bg-white/5 rounded-full border border-white/5">
                <Clock size={14} className="text-slate-500" />
                <span className="text-xs font-mono text-slate-300">
                  {mounted ? time : "00:00:00"}
                </span>
              </div>
              <button className="px-6 py-2 bg-cyan-500 text-black font-black text-xs uppercase tracking-widest rounded-full hover:bg-cyan-400 transition-colors shadow-lg shadow-cyan-500/20">
                Deploy Grid
              </button>
            </div>
          </div>

          {/* Header Stats */}
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
            <StatCard
              label="Asset Pivot"
              value={`${formatPrice(stats.pivot_price)} SOL`}
              subValue="Seeded VWAP Mid-Point"
              icon={<TrendingUp className="text-cyan-400" />}
              trend="+1.2%"
              isNeon
            />
            <StatCard
              label="SOL Balance"
              value={`${(stats.total_sol_balance ?? 0).toFixed(4)} SOL`}
              subValue="Total Swarm Reserve"
              icon={<Wallet className="text-emerald-400" />}
              status="Liquid"
            />
            <StatCard
              label="USDC Balance"
              value={`$${(stats.total_usdc_balance ?? 0).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`}
              subValue="Stablecoin Liquidity"
              icon={<Database className="text-blue-400" />}
              status="Ready"
            />
            <StatCard
              label="SOL/USDC"
              value={`$${chartData[chartData.length - 1]?.sol.toFixed(2) || "0.00"}`}
              subValue="Market Baseline"
              icon={<Activity className="text-purple-400" />}
              trend="-0.4%"
            />
          </div>

          {/* Enhanced Analytics Section */}
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
            <StatCard
              label="Whale Index"
              value={`${(stats.top_holders_percent ?? 0).toFixed(1)}%`}
              subValue="Concentration in Top 10"
              icon={<Users className="text-cyan-400" />}
              status={(stats.top_holders_percent ?? 0) > 50 ? "Centralized" : "Healthy"}
            />
            <StatCard
              label="Order Imbalance"
              value={`${((stats.imbalance_index ?? 0) * 100).toFixed(1)}%`}
              subValue={(stats.imbalance_index ?? 0) > 0 ? "Bid Dominance" : "Ask Dominance"}
              icon={<Scale className={`${(stats.imbalance_index ?? 0) > 0 ? 'text-emerald-400' : 'text-rose-400'}`} />}
            />
            <StatCard
              label="Safe Haven Index"
              value={`${(stats.safe_haven_index ?? 1.0).toFixed(2)}x`}
              subValue="Beta vs Solana Index"
              icon={<ShieldCheck className="text-blue-400" />}
            />
            <StatCard
              label="Market Spread"
              value={`${(stats.spread_bps ?? 0).toFixed(2)} bps`}
              subValue="Real-time Liquidity Gap"
              icon={<Zap className="text-yellow-400" />}
              status={(stats.spread_bps ?? 0) < 10 ? "Tight" : "Wide"}
            />
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
            <StatCard
              label="Node Status"
              value={`${stats.active_wallets} Active`}
              subValue="Multi-Wallet Rotation"
              icon={<Activity className="text-blue-400" />}
              status="Online"
            />
            <StatCard
              label="Channel Width"
              value={`${stats.buy_channel_width}% / ${stats.sell_channel_width}%`}
              subValue="Dynamic Volatility Bound"
              icon={<Shield className="text-emerald-400" />}
              status="Protected"
            />
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
            {/* Primary Chart Area */}
            <div className="lg:col-span-3 space-y-8">
              <div className="glass-panel rounded-[2rem] p-8 border border-white/5 relative overflow-hidden">
                <div className="absolute top-0 right-0 p-8 flex flex-col items-end gap-4">
                  <div className="flex gap-1.5 p-1 bg-[#0f172a] rounded-lg border border-white/5">
                    {['1H', '1D', '1W', 'ALL'].map((tf) => (
                      <button key={tf} className={`px-3 py-1 rounded-md text-[10px] font-bold transition-all ${tf === '1D' ? 'bg-cyan-500 text-black' : 'text-slate-500 hover:text-white'}`}>
                        {tf}
                      </button>
                    ))}
                  </div>
                  <div className="flex items-center gap-2 px-3 py-1 bg-yellow-500/10 border border-yellow-500/20 rounded-full">
                    <div className="w-1.5 h-1.5 rounded-full bg-yellow-400 animate-pulse" />
                    <span className="text-[10px] font-black uppercase tracking-widest text-yellow-400">Dry Run Mode</span>
                  </div>
                </div>

                <h3 className="text-xl font-black mb-8 flex items-center gap-3">
                  <div className="w-1.5 h-6 bg-cyan-400 rounded-full" />
                  Asset Price History
                </h3>

                <div className="h-[400px] w-full">
                  {mounted && (
                    <D3AreaChart
                      data={chartData}
                      dataKey="asset"
                      color="#22d3ee"
                      gradientId="colorAsset"
                      name="BMV Base Price"
                      pivotPrice={parseFloat(stats.pivot_price)}
                      buyChannelWidth={parseFloat(stats.buy_channel_width)}
                      sellChannelWidth={parseFloat(stats.sell_channel_width)}
                      yAxisFormatter={yAxisFormatter}
                      margin={chartMargin}
                    />
                  )}
                </div>
              </div>

              {/* Secondary SOL Chart */}
              <div className="glass-panel rounded-[2rem] p-8 border border-white/5">
                <h3 className="text-xl font-black mb-6 flex items-center gap-3">
                  <div className="w-1.5 h-6 bg-purple-400 rounded-full" />
                  SOL/USDC Correlation
                </h3>
                <div className="h-[400px] w-full">
                  {mounted && (
                    <D3AreaChart
                      data={chartData}
                      dataKey="sol"
                      color="#a855f7"
                      gradientId="colorSol"
                      name="SOL Correlation"
                      yAxisFormatter={yAxisFormatter}
                      margin={chartMargin}
                    />
                  )}
                </div>
              </div>
            </div>

            {/* Controls Side Panel */}
            <div className="lg:col-span-1 space-y-6">
              <div className="glass-panel rounded-[2rem] p-8 border border-white/5 h-full">
                <h3 className="text-lg font-black mb-8 flex items-center gap-2 text-slate-300">
                  <Zap size={18} className="text-yellow-400" />
                  Tactical Control
                </h3>

                <div className="space-y-4">
                  <TacticalButton
                    label="Force Rebalance"
                    icon={<RefreshCcw size={18} />}
                    color="cyan"
                    onClick={() => handleControl("rebalance")}
                  />
                  <TacticalButton
                    label="Kill Switch"
                    icon={<Shield size={18} />}
                    color="red"
                    onClick={() => handleControl("kill_switch")}
                    urgent
                  />

                  <div className="mt-10 pt-10 border-t border-white/5">
                    <h4 className="text-[10px] font-black uppercase tracking-[0.2em] text-slate-500 mb-6 font-mono flex items-center justify-between">
                      <span>Order Book V1</span>
                      <span className="text-cyan-400">Live</span>
                    </h4>
                    <div className="space-y-3">
                      {stats.asks.slice(0, 3).reverse().map((ask, i) => (
                        <DepthBar key={`ask-${i}`} side="sell" width={`w-[${Math.min(100, (ask.size / 100) * 100)}%]`} price={ask.price.toString()} size={ask.size.toFixed(1)} />
                      ))}
                      <div className="py-2 text-center relative">
                        <div className="absolute inset-0 flex items-center" aria-hidden="true">
                          <div className="w-full border-t border-white/5"></div>
                        </div>
                        <span className="relative z-10 text-[10px] bg-cyan-500/10 text-cyan-400 px-3 py-1 rounded-full font-black tracking-tighter ring-1 ring-cyan-500/20">
                          {formatPrice(stats.pivot_price)} MID
                        </span>
                      </div>
                      {stats.bids.slice(0, 3).map((bid, i) => (
                        <DepthBar key={`bid-${i}`} side="buy" width={`w-[${Math.min(100, (bid.size / 100) * 100)}%]`} price={bid.price.toString()} size={bid.size.toFixed(1)} />
                      ))}
                    </div>

                    <div className="mt-6 flex flex-col gap-2 p-4 glass-panel rounded-2xl border border-white/5 bg-cyan-500/5">
                      <div className="flex items-center justify-between text-[10px] font-black uppercase tracking-widest text-slate-500">
                        <span>Liquidity Concentr.</span>
                        <span className="text-cyan-400">Target Level</span>
                      </div>
                      <div className="space-y-2">
                        <div className="flex items-center justify-between text-[11px] font-mono">
                          <span className="text-slate-400">RESIST (90%)</span>
                          <span className="text-rose-400 font-bold">{formatPrice(stats.resistance_90)}</span>
                        </div>
                        <div className="flex items-center justify-between text-[11px] font-mono">
                          <span className="text-slate-400">RESIST (50%)</span>
                          <span className="text-rose-400/80">{formatPrice(stats.resistance_50)}</span>
                        </div>
                        <div className="flex items-center justify-between text-[11px] font-mono pt-2 border-t border-white/5">
                          <span className="text-slate-400">SUPPORT (50%)</span>
                          <span className="text-emerald-400/80">{formatPrice(stats.support_50)}</span>
                        </div>
                        <div className="flex items-center justify-between text-[11px] font-mono">
                          <span className="text-slate-400">SUPPORT (90%)</span>
                          <span className="text-emerald-400 font-bold">{formatPrice(stats.support_90)}</span>
                        </div>
                      </div>
                    </div>

                    <div className="mt-8 h-[120px] w-full glass-panel rounded-xl p-2 border border-white/5 overflow-hidden">
                      <D3DepthChart bids={stats.bids} asks={stats.asks} />
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}


function StatCard({ label, value, subValue, icon, isNeon = false, trend, status }: any) {
  return (
    <div className={`glass-panel rounded-3xl p-6 border border-white/5 hover:border-white/10 transition-all flex flex-col justify-between group`}>
      <div className="flex items-start justify-between mb-6">
        <div className="p-2.5 bg-white/5 rounded-xl group-hover:bg-white/10 transition-colors">{icon}</div>
        {trend && (
          <span className={`text-[10px] px-2 py-1 rounded-full font-black ${trend.startsWith('+') ? 'bg-emerald-500/10 text-emerald-400' : 'bg-red-500/10 text-red-400'}`}>
            {trend}
          </span>
        )}
        {status && (
          <span className="text-[10px] px-2 py-1 bg-blue-500/10 text-blue-400 rounded-full font-black uppercase tracking-widest">
            {status}
          </span>
        )}
      </div>
      <div>
        <p className="text-slate-500 text-xs font-bold uppercase tracking-widest mb-1">{label}</p>
        <h4 className={`text-2xl font-black tracking-tight ${isNeon ? 'text-cyan-400' : 'text-slate-100'}`}>{value}</h4>
        <p className="text-[10px] text-slate-500 mt-1 font-medium">{subValue}</p>
      </div>
    </div>
  );
}

function TacticalButton({ label, icon, color, onClick, urgent, disabled }: any) {
  const colorClasses = {
    cyan: "bg-cyan-500/10 text-cyan-400 border-cyan-500/20 hover:bg-cyan-500/20 shadow-cyan-500/5",
    red: "bg-red-500/10 text-red-400 border-red-500/20 hover:bg-red-500/20 shadow-red-500/5",
  }[color as 'cyan' | 'red'];

  const glowClasses = {
    cyan: "shadow-cyan-500/20",
    red: "shadow-red-500/20",
  }[color as 'cyan' | 'red'];

  return (
    <button
      onClick={onClick}
      className={`w-full flex items-center justify-between p-4 rounded-2xl border transition-all active:scale-95 group shadow-lg ${colorClasses} ${urgent ? 'animate-pulse' : ''}`}
    >
      <div className="flex items-center gap-3">
        <span className="group-hover:scale-110 transition-transform">{icon}</span>
        <span className="font-black text-xs uppercase tracking-widest">{label}</span>
      </div>
      <div className={`w-1.5 h-1.5 rounded-full ${color === 'cyan' ? 'bg-cyan-400' : 'bg-red-400'} ${glowClasses}`} />
    </button>
  );
}

function DepthBar({ side, width, price, size }: any) {
  return (
    <div className="flex items-center justify-between text-[10px] font-bold group px-2 py-1 rounded-lg hover:bg-white/5 transition-colors">
      <span className={`w-12 font-mono ${side === 'buy' ? 'text-emerald-400' : 'text-red-400'}`}>{price}</span>
      <div className="flex-1 mx-4 h-1 rounded-full bg-white/5 overflow-hidden">
        <motion.div
          initial={{ width: 0 }}
          animate={{ width: width === 'w-[30%]' ? '30%' : width === 'w-[15%]' ? '15%' : width === 'w-[65%]' ? '65%' : '40%' }}
          className={`h-full rounded-full ${side === 'buy' ? 'bg-emerald-500/40' : 'bg-red-500/40'}`}
        />
      </div>
      <span className="w-12 text-right text-slate-500 font-mono tracking-tighter">{size}</span>
    </div>
  );
}
