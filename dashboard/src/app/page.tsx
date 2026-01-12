"use client";

import React, { useState, useEffect } from "react";
import {
  Zap,
  Shield,
  RefreshCcw,
  Activity,
  Wallet,
  TrendingUp,
  LayoutDashboard,
  Settings,
  Flame
} from "lucide-react";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  AreaChart,
  Area
} from "recharts";
import { motion, AnimatePresence } from "framer-motion";

// Mock data for the chart - in production, this comes from the DB
const MOCK_CHART_DATA = [
  { time: "09:00", price: 92.4, pnl: 1200 },
  { time: "10:00", price: 93.1, pnl: 1450 },
  { time: "11:00", price: 92.8, pnl: 1300 },
  { time: "12:00", price: 94.5, pnl: 1800 },
  { time: "13:00", price: 95.2, pnl: 2100 },
  { time: "14:00", price: 94.8, pnl: 1950 },
  { time: "15:00", price: 95.4, pnl: 2450 },
];

export default function Dashboard() {
  const [stats, setStats] = useState({
    pivot_price: "0.00",
    buy_channel_width: "0.00",
    sell_channel_width: "0.00",
    active_wallets: 0,
    kill_switch_active: false,
  });
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchStats = async () => {
      try {
        const response = await fetch("http://localhost:8080/stats");
        if (response.ok) {
          const data = await response.json();
          setStats(data);
        }
      } catch (error) {
        console.error("Failed to fetch stats:", error);
      } finally {
        setLoading(false);
      }
    };

    fetchStats();
    const interval = setInterval(fetchStats, 5000);
    return () => clearInterval(interval);
  }, []);

  const handleControl = async (action: string) => {
    try {
      await fetch("http://localhost:8080/control", {
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
    <div className="flex h-screen overflow-hidden font-sans">
      {/* Sidebar */}
      <aside className="w-64 glass-panel border-r border-white/5 flex flex-col p-6 z-10">
        <div className="flex items-center gap-3 mb-12">
          <div className="p-2 bg-cyan-500/20 rounded-lg">
            <Flame className="w-6 h-6 text-cyan-400" />
          </div>
          <h1 className="text-xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-cyan-400 to-purple-400">
            BMV BOT
          </h1>
        </div>

        <nav className="flex-1 space-y-2">
          <NavItem icon={<LayoutDashboard size={20} />} label="Dashboard" active />
          <NavItem icon={<TrendingUp size={20} />} label="Market History" />
          <NavItem icon={<Wallet size={20} />} label="Wallets" />
          <NavItem icon={<Settings size={20} />} label="Configuration" />
        </nav>

        <div className="mt-auto pt-6 border-t border-white/5">
          <div className="flex items-center gap-3 p-3 rounded-xl hover:bg-white/5 transition-colors cursor-pointer">
            <div className="w-8 h-8 rounded-full bg-gradient-to-br from-cyan-500 to-purple-500" />
            <div className="text-sm">
              <p className="font-medium">oyakov</p>
              <p className="text-white/40 text-xs">Admin</p>
            </div>
          </div>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 overflow-y-auto p-8 relative">
        <div className="max-w-6xl mx-auto">
          {/* Header Stats */}
          <div className="grid grid-cols-1 md:grid-cols-4 gap-6 mb-8">
            <StatCard
              label="Real-time PnL"
              value="+$2,450.50"
              subValue="+12.5% today"
              icon={<TrendingUp className="text-green-400" />}
            />
            <StatCard
              label="Pivot Price"
              value={`$${stats.pivot_price}`}
              subValue="Seeded VWAP"
              icon={<Activity className="text-cyan-400" />}
              isNeon
            />
            <StatCard
              label="Active Wallets"
              value={stats.active_wallets.toString()}
              subValue="Healthy Swarm"
              icon={<Wallet className="text-purple-400" />}
            />
            <StatCard
              label="Buy/Sell Channel"
              value={`${stats.buy_channel_width}% / ${stats.sell_channel_width}%`}
              subValue="Dynamic Asymmetry"
              icon={<Shield className="text-blue-400" />}
            />
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* Chart Area */}
            <div className="lg:col-span-2 glass-panel rounded-3xl p-8 relative overflow-hidden">
              <div className="flex items-center justify-between mb-8">
                <h3 className="text-lg font-semibold flex items-center gap-2">
                  <Activity size={18} className="text-cyan-400" />
                  Price Movement
                </h3>
                <div className="flex gap-2">
                  <button className="px-3 py-1 rounded-full bg-white/5 text-xs hover:bg-white/10 transition-colors">1H</button>
                  <button className="px-3 py-1 rounded-full bg-cyan-500/20 text-cyan-400 text-xs font-medium">1D</button>
                  <button className="px-3 py-1 rounded-full bg-white/5 text-xs hover:bg-white/10 transition-colors">1W</button>
                </div>
              </div>

              <div className="h-80 w-full">
                <ResponsiveContainer width="100%" height="100%">
                  <AreaChart data={MOCK_CHART_DATA}>
                    <defs>
                      <linearGradient id="colorPrice" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="5%" stopColor="#22d3ee" stopOpacity={0.3} />
                        <stop offset="95%" stopColor="#22d3ee" stopOpacity={0} />
                      </linearGradient>
                    </defs>
                    <CartesianGrid strokeDasharray="3 3" stroke="#ffffff05" vertical={false} />
                    <XAxis dataKey="time" stroke="#ffffff44" fontSize={12} tickLine={false} axisLine={false} />
                    <YAxis stroke="#ffffff44" fontSize={12} tickLine={false} axisLine={false} domain={['auto', 'auto']} />
                    <Tooltip
                      contentStyle={{ backgroundColor: '#0f172a', border: '1px solid #ffffff11', borderRadius: '12px' }}
                      itemStyle={{ color: '#22d3ee' }}
                    />
                    <Area type="monotone" dataKey="price" stroke="#22d3ee" strokeWidth={3} fillOpacity={1} fill="url(#colorPrice)" />
                  </AreaChart>
                </ResponsiveContainer>
              </div>
            </div>

            {/* Controls & Order Book */}
            <div className="space-y-6">
              {/* Trading Controls */}
              <div className="glass-panel rounded-3xl p-8">
                <h3 className="text-lg font-semibold mb-6 flex items-center gap-2">
                  <Zap size={18} className="text-purple-400" />
                  Bot Operations
                </h3>

                <div className="space-y-4">
                  <button
                    onClick={() => handleControl("rebalance")}
                    className="w-full flex items-center justify-between p-4 rounded-2xl bg-white/5 hover:bg-white/10 transition-all border border-white/5 group"
                  >
                    <div className="flex items-center gap-3">
                      <RefreshCcw className="text-cyan-400 group-hover:rotate-180 transition-transform duration-500" size={20} />
                      <span className="font-medium">Force Rebalance</span>
                    </div>
                    <span className="text-xs text-white/40">Manual Trigger</span>
                  </button>

                  <button
                    onClick={() => handleControl("kill_switch")}
                    className="w-full flex items-center justify-between p-4 rounded-2xl bg-red-500/10 hover:bg-red-500/20 transition-all border border-red-500/20 group"
                  >
                    <div className="flex items-center gap-3">
                      <Shield className="text-red-400 animate-pulse" size={20} />
                      <span className="font-medium text-red-100">KILL SWITCH</span>
                    </div>
                    <div className="w-12 h-6 bg-red-500/20 rounded-full flex items-center p-1">
                      <div className="w-4 h-4 bg-red-500 rounded-full" />
                    </div>
                  </button>
                </div>
              </div>

              {/* Order Book Visualization */}
              <div className="glass-panel rounded-3xl p-8 flex-1">
                <h3 className="text-lg font-semibold mb-6 flex items-center gap-2">
                  <LayoutDashboard size={18} className="text-blue-400" />
                  L2 Market Depth
                </h3>
                <div className="space-y-2">
                  <DepthBar side="sell" width="w-[30%]" price="96.1" />
                  <DepthBar side="sell" width="w-[45%]" price="95.8" />
                  <DepthBar side="sell" width="w-[20%]" price="95.5" />
                  <div className="py-2 text-center text-xs font-mono text-cyan-400/60 tracking-widest uppercase">SPREAD 0.1%</div>
                  <DepthBar side="buy" width="w-[80%]" price="95.3" />
                  <DepthBar side="buy" width="w-[60%]" price="95.0" />
                  <DepthBar side="buy" width="w-[90%]" price="94.7" />
                </div>
              </div>
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}

function NavItem({ icon, label, active = false }: { icon: React.ReactNode, label: string, active?: boolean }) {
  return (
    <div className={`
      flex items-center gap-3 px-4 py-3 rounded-xl transition-all cursor-pointer group
      ${active ? 'bg-cyan-500/10 text-cyan-400 border border-cyan-500/20' : 'text-white/40 hover:text-white/80 hover:bg-white/5'}
    `}>
      {icon}
      <span className="font-medium">{label}</span>
    </div>
  );
}

function StatCard({ label, value, subValue, icon, isNeon = false }: { label: string, value: string, subValue: string, icon: React.ReactNode, isNeon?: boolean }) {
  return (
    <div className={`glass-panel rounded-2xl p-6 border-l-4 ${isNeon ? 'neon-border-cyan border-l-cyan-400' : 'border-l-transparent'}`}>
      <div className="flex items-start justify-between mb-4">
        <span className="text-white/40 text-sm font-medium">{label}</span>
        <div className="p-2 bg-white/5 rounded-lg">{icon}</div>
      </div>
      <div className="space-y-1">
        <h4 className={`text-2xl font-bold ${isNeon ? 'neon-cyan' : ''}`}>{value}</h4>
        <p className="text-xs text-white/30">{subValue}</p>
      </div>
    </div>
  );
}

function DepthBar({ side, width, price }: { side: 'buy' | 'sell', width: string, price: string }) {
  return (
    <div className="flex items-center justify-between text-xs font-mono">
      <span className={side === 'buy' ? 'text-green-400' : 'text-red-400'}>{price}</span>
      <div className="flex-1 mx-4 h-1.5 rounded-full bg-white/5 overflow-hidden">
        <div className={`h-full rounded-full ${side === 'buy' ? 'bg-green-500/40' : 'bg-red-500/40'} ${width}`} />
      </div>
      <span className="text-white/40">100.5</span>
    </div>
  );
}
