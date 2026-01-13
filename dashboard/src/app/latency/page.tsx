"use client";

import React, { useState, useEffect, useMemo } from "react";
import {
    Clock,
    ArrowLeft,
    Activity,
    Zap,
    Shield,
    LayoutDashboard,
    BarChart3,
    Wallet,
    Database,
    Flame,
    Globe
} from "lucide-react";
import D3LineChart from "../../components/D3LineChart";
import { motion } from "framer-motion";
import Link from "next/link";
import { usePathname } from "next/navigation";

interface LatencyTick {
    timestamp: number;
    service_name: string;
    status: string;
    latency_ms: number;
}

export default function LatencyDashboard() {
    const [latencyData, setLatencyData] = useState<Record<string, LatencyTick[]>>({});
    const [loading, setLoading] = useState(true);
    const [mounted, setMounted] = useState(false);

    useEffect(() => {
        setMounted(true);
        const fetchData = async () => {
            try {
                // Try 127.0.0.1 explicitly if hostname is localhost to avoid IPv6 issues
                const hostname = typeof window !== 'undefined' ? window.location.hostname : 'localhost';
                const host = hostname === 'localhost' ? '127.0.0.1' : hostname;
                const res = await fetch(`http://${host}:8080/latency`);
                if (res.ok) {
                    const data = await res.json();
                    setLatencyData(data);
                }
            } catch (error) {
                console.error("Failed to fetch latency data:", error);
            } finally {
                setLoading(false);
            }
        };

        fetchData();
        const interval = setInterval(fetchData, 10000);
        return () => clearInterval(interval);
    }, []);

    const services = Object.keys(latencyData);

    const chartData = useMemo(() => {
        // Combine all service ticks into a single timeline for the graph
        // This is a bit complex since timestamps might not align perfectly
        const allTimestamps = new Set<number>();
        Object.values(latencyData).forEach(ticks => {
            ticks.forEach(t => allTimestamps.add(t.timestamp));
        });

        const sortedTimestamps = Array.from(allTimestamps).sort((a, b) => a - b);

        return sortedTimestamps.map(ts => {
            const entry: any = {
                time: new Date(ts * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false }),
            };
            services.forEach(service => {
                const tick = latencyData[service].find(t => t.timestamp === ts);
                if (tick) {
                    entry[service] = tick.latency_ms;
                }
            });
            return entry;
        });
    }, [latencyData, services]);

    return (
        <div className="flex h-screen overflow-hidden bg-[#020617] text-slate-100 font-sans selection:bg-cyan-500/30">
            {/* Background */}
            <div className="absolute inset-0 overflow-hidden pointer-events-none">
                <div className="absolute -top-[20%] -left-[10%] w-[60%] h-[60%] bg-purple-500/10 blur-[120px] rounded-full animate-pulse" />
                <div className="absolute -bottom-[20%] -right-[10%] w-[50%] h-[50%] bg-cyan-500/10 blur-[120px] rounded-full animate-pulse delay-1000" />
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
                        <p className="text-[10px] uppercase tracking-[0.2em] font-bold text-cyan-400">System v0.3.3</p>
                    </div>
                </div>

                <nav className="flex-1 space-y-1">
                    <NavItem icon={<LayoutDashboard size={18} />} label="Overview" href="/" />
                    <NavItem icon={<Clock size={18} />} label="Latency" href="/latency" />
                    <NavItem icon={<Activity size={18} />} label="Strategy Logs" />
                    <NavItem icon={<BarChart3 size={18} />} label="Performance" />
                    <NavItem icon={<Wallet size={18} />} label="Wallet Swarm" />
                    <NavItem icon={<Database size={18} />} label="API Docs" />
                </nav>
            </aside>

            {/* Main Content */}
            <main className="flex-1 overflow-y-auto p-8 relative z-10 scrollbar-hide">
                <div className="max-w-[1400px] mx-auto">
                    {/* Header */}
                    <div className="flex items-center justify-between mb-10">
                        <div>
                            <div className="flex items-center gap-2 text-cyan-400 text-xs font-bold uppercase tracking-widest mb-2">
                                <Shield size={14} />
                                Network Observability
                            </div>
                            <h2 className="text-3xl font-black tracking-tight mb-1">Infrastructure Latency</h2>
                            <p className="text-slate-400 text-sm flex items-center gap-2">
                                Real-time dependency performance monitoring
                            </p>
                        </div>
                        <Link href="/" className="px-6 py-2 bg-white/5 border border-white/10 text-slate-300 font-bold text-xs uppercase tracking-widest rounded-full hover:bg-white/10 transition-all flex items-center gap-2">
                            <ArrowLeft size={14} />
                            Return to Command Center
                        </Link>
                    </div>

                    {/* Latency Cards */}
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
                        {services.map(service => {
                            const latest = latencyData[service][latencyData[service].length - 1];
                            const status = latest?.status || "UNKNOWN";
                            const isHealthy = status === "HEALTHY";
                            const isDegraded = status === "DEGRADED";

                            return (
                                <div key={service} className="glass-panel rounded-3xl p-6 border border-white/5 hover:border-white/10 transition-all">
                                    <div className="flex items-start justify-between mb-6">
                                        <div className="p-2.5 bg-white/5 rounded-xl">
                                            <Zap className={isHealthy ? "text-cyan-400" : isDegraded ? "text-yellow-400" : "text-red-400"} />
                                        </div>
                                        <span className={`text-[10px] px-2 py-1 rounded-full font-black uppercase tracking-widest ${isHealthy ? 'bg-emerald-500/10 text-emerald-400' :
                                            isDegraded ? 'bg-yellow-500/10 text-yellow-400' : 'bg-red-500/10 text-red-400'
                                            }`}>
                                            {status}
                                        </span>
                                    </div>
                                    <div>
                                        <p className="text-slate-500 text-xs font-bold uppercase tracking-widest mb-1">{service}</p>
                                        <h4 className="text-2xl font-black tracking-tight text-slate-100">{latest?.latency_ms || 0} ms</h4>
                                    </div>
                                </div>
                            );
                        })}
                    </div>

                    {/* Latency History Chart */}
                    <div className="glass-panel rounded-[2rem] p-8 border border-white/5">
                        <h3 className="text-xl font-black mb-8 flex items-center gap-3">
                            <div className="w-1.5 h-6 bg-cyan-400 rounded-full" />
                            Temporal Latency Distribution
                        </h3>
                        <div className="h-[500px] w-full">
                            {mounted && (
                                <D3LineChart data={chartData} services={services} />
                            )}
                        </div>
                    </div>
                </div>
            </main>
        </div>
    );
}

function NavItem({ icon, label, href }: { icon: React.ReactNode, label: string, href?: string }) {
    const pathname = usePathname();
    const active = href ? pathname === href : false;

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
