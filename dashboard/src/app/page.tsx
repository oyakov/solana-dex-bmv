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
  Clock
} from "lucide-react";
import {
  ReferenceArea
} from "recharts";
import D3AreaChart from "../components/D3AreaChart";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { motion, AnimatePresence } from "framer-motion";

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
  });
  const [history, setHistory] = useState<PriceTick[]>([]);
  const [loading, setLoading] = useState(true);

  const [mounted, setMounted] = useState(false);
  const [time, setTime] = useState("");

  const formatPrice = (val: string | number) => {
    const num = Number(val);
    if (isNaN(num)) return "0.000000";
    if (num < 1) {
      // If it's extremely small, show up to 9 decimals
      return num.toFixed(9);
    }
    return num.toLocaleString(undefined, { minimumFractionDigits: 6, maximumFractionDigits: 9 });
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
        const hostname = typeof window !== 'undefined' ? window.location.hostname : 'localhost';
        const host = hostname === 'localhost' ? '127.0.0.1' : hostname;
        const statsRes = await fetch(`http://${host}:8080/stats`);
        if (statsRes.ok) {
          const data = await statsRes.json();
          setStats(data);
        }

        const historyRes = await fetch(`http://${host}:8080/history`);
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
    const interval = setInterval(fetchData, 10000);
    return () => clearInterval(interval);
  }, []);

  const chartData = useMemo(() => {
    if (history.length === 0) return [];
    return history.map(tick => ({
      time: new Date(tick.timestamp * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
      asset: parseFloat(tick.asset_price),
      sol: parseFloat(tick.sol_price),
    }));
  }, [history]);

  const handleControl = async (action: string) => {
    try {
      const host = window.location.hostname;
      await fetch(`http://${host}:8080/control`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ action }),
      });
      alert(`Action ${action} triggered successfully`);
    } catch (error) {
      console.error("Control action failed:", error);
    }
  };

  return (
    <div className="flex h-screen overflow-hidden bg-[#020617] text-slate-100 font-sans selection:bg-cyan-500/30">
      {/* Dynamic Background */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        <div className="absolute -top-[20%] -left-[10%] w-[60%] h-[60%] bg-cyan-500/10 blur-[120px] rounded-full animate-pulse" />
        <div className="absolute -bottom-[20%] -right-[10%] w-[50%] h-[50%] bg-purple-500/10 blur-[120px] rounded-full animate-pulse delay-1000" />
      </div>

      {/* Sidebar */}
      <aside className="w-64 glass-panel border-r border-white/5 flex flex-col p-6 z-20 relative backdrop-blur-xl">
        <div className="flex items-center gap-3 mb-10 pl-2">
          <div className="p-2.5 bg-gradient-to-br from-cyan-400 to-blue-600 rounded-xl shadow-lg shadow-cyan-500/20">
            <Flame className="w-6 h-6 text-white" />
          </div>
          <div>
            <h1 className="text-xl font-black tracking-tighter bg-clip-text text-transparent bg-gradient-to-r from-white to-white/60">
              BMV ECO
            </h1>
            <p className="text-[10px] uppercase tracking-[0.2em] font-bold text-cyan-400">System v0.3.4</p>
          </div>
        </div>

        <nav className="flex-1 space-y-1">
          <NavItem icon={<LayoutDashboard size={18} />} label="Overview" active href="/" />
          <NavItem icon={<Clock size={18} />} label="Latency" href="/latency" />
          <NavItem icon={<Activity size={18} />} label="Strategy Logs" />
          <NavItem icon={<BarChart3 size={18} />} label="Performance" />
          <NavItem icon={<Wallet size={18} />} label="Wallet Swarm" href="/wallets" />
          <NavItem icon={<Database size={18} />} label="API Docs" />
        </nav>

        <div className="mt-auto pt-6 border-t border-white/5">
          <div className="p-4 rounded-2xl bg-white/5 border border-white/5 flex items-center gap-3 active:scale-95 transition-transform cursor-pointer">
            <div className="w-10 h-10 rounded-full bg-gradient-to-br from-cyan-400 to-purple-500 p-[2px]">
              <div className="w-full h-full rounded-full bg-[#0f172a] flex items-center justify-center">
                <span className="font-bold text-xs">OY</span>
              </div>
            </div>
            <div className="text-sm overflow-hidden">
              <p className="font-bold truncate text-slate-200">oyakov.sol</p>
              <div className="flex items-center gap-1">
                <div className="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse" />
                <p className="text-white/40 text-[10px] font-bold uppercase tracking-wider">Connected</p>
              </div>
            </div>
          </div>
        </div>
      </aside>

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
              subValue="Seeded VWAP Strategy"
              icon={<TrendingUp className="text-cyan-400" />}
              trend="+1.2%"
              isNeon
            />
            <StatCard
              label="SOL/USDC"
              value={`$${chartData[chartData.length - 1]?.sol.toFixed(2) || "0.00"}`}
              subValue="Market Baseline"
              icon={<Activity className="text-purple-400" />}
              trend="-0.4%"
            />
            <StatCard
              label="Node Status"
              value={`${stats.active_wallets} Active`}
              subValue="Multi-Wallet Rotation"
              icon={<Wallet className="text-blue-400" />}
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
                    <h4 className="text-[10px] font-black uppercase tracking-[0.2em] text-slate-500 mb-6">Market Microstructure</h4>
                    <div className="space-y-3">
                      <DepthBar side="sell" width="w-[30%]" price="100.4" size="1.4k" />
                      <DepthBar side="sell" width="w-[15%]" price="100.2" size="0.8k" />
                      <div className="py-2 text-center">
                        <span className="text-[10px] bg-cyan-500/10 text-cyan-400 px-3 py-1 rounded-full font-bold">100.1 MID</span>
                      </div>
                      <DepthBar side="buy" width="w-[65%]" price="100.0" size="4.2k" />
                      <DepthBar side="buy" width="w-[40%]" price="99.8" size="2.1k" />
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

function NavItem({ icon, label, active: propActive = false, href }: { icon: React.ReactNode, label: string, active?: boolean, href?: string }) {
  const pathname = usePathname();
  const active = href ? pathname === href : propActive;

  const content = (
    <div className={`
      flex items-center gap-3 px-5 py-3.5 rounded-2xl transition-all cursor-pointer group relative
      ${active ? 'bg-cyan-500/10 text-cyan-400 shadow-inner' : 'text-slate-500 hover:text-slate-200 hover:bg-white/5'}
    `}>
      {active && <motion.div layoutId="nav-active" className="absolute inset-0 bg-cyan-500/5 rounded-2xl border border-cyan-500/20" />}
      <span className="relative z-10">{icon}</span>
      <span className="font-bold text-sm tracking-tight relative z-10">{label}</span>
      {active && <div className="ml-auto w-1.5 h-1.5 rounded-full bg-cyan-400 shadow-[0_0_8px_rgba(34,211,238,0.8)]" />}
    </div>
  );

  if (href) {
    return <Link href={href}>{content}</Link>;
  }

  return content;
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
