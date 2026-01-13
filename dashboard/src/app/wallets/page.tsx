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
    Flame,
    Globe,
    Database,
    BarChart3,
    Clock,
    ArrowLeft,
    LogOut
} from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { motion, AnimatePresence } from "framer-motion";
import WalletList from "@/components/WalletList";
import { logout } from "../../utils/auth";


export default function WalletsPage() {
    const [mounted, setMounted] = useState(false);
    const [time, setTime] = useState("");

    useEffect(() => {
        setMounted(true);
        const updateTime = () => {
            setTime(new Date().toLocaleTimeString([], { hour12: false }));
        };
        updateTime();
        const interval = setInterval(updateTime, 1000);
        return () => clearInterval(interval);
    }, []);

    return (
        <div className="min-h-screen bg-[#020617] text-slate-100 font-sans selection:bg-cyan-500/30 overflow-hidden flex">
            {/* Dynamic Background */}
            <div className="fixed inset-0 pointer-events-none">
                <div className="absolute top-[-10%] left-[-10%] w-[40%] h-[40%] bg-cyan-500/10 blur-[120px] rounded-full animate-pulse" />
                <div className="absolute bottom-[-10%] right-[-10%] w-[40%] h-[40%] bg-purple-500/10 blur-[120px] rounded-full" />
                <div className="absolute inset-0 bg-[url('https://grainy-gradients.vercel.app/noise.svg')] opacity-20 contrast-150 brightness-100" />
            </div>

            {/* Sidebar Navigation */}
            <aside className="w-80 h-screen border-r border-white/5 bg-[#020617]/80 backdrop-blur-2xl flex flex-col relative z-20 shrink-0">
                <div className="p-10">
                    <div className="flex items-center gap-4 mb-12 group cursor-pointer">
                        <div className="w-12 h-12 bg-cyan-500 rounded-2xl flex items-center justify-center shadow-lg shadow-cyan-500/20 group-hover:rotate-12 transition-transform duration-500">
                            <Zap className="text-black" size={24} fill="black" />
                        </div>
                        <div>
                            <h1 className="text-2xl font-black tracking-tighter leading-none">BMV.BOT</h1>
                            <p className="text-cyan-400/60 text-[10px] font-bold uppercase tracking-[0.3em] mt-1">Swarmlord Prototype</p>
                        </div>
                    </div>

                    <nav className="space-y-2">
                        <NavItem icon={<LayoutDashboard size={20} />} label="Command Center" href="/" />
                        <NavItem icon={<Activity size={20} />} label="Latency Report" href="/latency" />
                        <NavItem icon={<Wallet size={20} />} label="Wallet Swarm" active href="/wallets" />
                        <NavItem icon={<BarChart3 size={20} />} label="PnL Engine" />
                        <NavItem icon={<Settings size={20} />} label="Protocol Config" />
                        <div onClick={logout} className="flex items-center gap-3 px-5 py-4 rounded-2xl transition-all cursor-pointer group relative text-slate-500 hover:text-red-400 hover:bg-red-500/5 mt-4">
                            <LogOut size={20} />
                            <span className="font-bold text-sm tracking-tight relative z-10">Logout</span>
                        </div>
                    </nav>

                </div>

                <div className="mt-auto p-10">
                    <div className="p-6 bg-white/5 rounded-[2rem] border border-white/5 relative group overflow-hidden">
                        <div className="absolute inset-0 bg-gradient-to-br from-cyan-500/5 to-purple-500/5 opacity-0 group-hover:opacity-100 transition-opacity" />
                        <div className="flex items-center gap-4 relative z-10">
                            <div className="relative">
                                <div className="w-10 h-10 rounded-full bg-slate-800 border border-white/10 ring-2 ring-emerald-500/20 ring-offset-2 ring-offset-[#020617]" />
                                <div className="absolute bottom-0 right-0 w-3 h-3 bg-emerald-500 rounded-full border-2 border-[#020617]" />
                            </div>
                            <div className="flex flex-col">
                                <span className="text-xs font-black text-slate-200 tracking-tight">System Status</span>
                                <div className="flex items-center gap-1.5 mt-0.5">
                                    <div className="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse" />
                                    <p className="text-white/40 text-[10px] font-bold uppercase tracking-wider">Connected</p>
                                </div>
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
                            <div className="flex items-center gap-2 mb-2">
                                <Link href="/" className="p-1 px-2 bg-white/5 rounded-lg border border-white/5 hover:bg-white/10 text-slate-400 hover:text-white transition-all text-[10px] font-black uppercase tracking-widest flex items-center gap-1">
                                    <ArrowLeft size={10} />
                                    Back
                                </Link>
                            </div>
                            <h2 className="text-4xl font-black tracking-tight mb-1">Wallet Swarm</h2>
                            <p className="text-slate-400 text-sm flex items-center gap-2 font-medium">
                                <Database size={14} className="text-blue-400" />
                                Hardware-Level Management & Rotation
                            </p>
                        </div>
                        <div className="flex items-center gap-3">
                            <div className="flex items-center gap-2 px-4 py-2 bg-white/5 rounded-full border border-white/5">
                                <Clock size={14} className="text-slate-500" />
                                <span className="text-xs font-mono text-slate-300">
                                    {mounted ? time : "00:00:00"}
                                </span>
                            </div>
                            <div className="px-5 py-2 bg-blue-500/10 border border-blue-500/20 rounded-full">
                                <span className="text-[10px] font-black uppercase tracking-widest text-blue-400 flex items-center gap-2">
                                    <Shield size={10} />
                                    Multi-Sig Layer Active
                                </span>
                            </div>
                        </div>
                    </div>

                    {/* Wallet List Component */}
                    <div className="glass-panel rounded-[2.5rem] p-10 border border-white/5 relative overflow-hidden min-h-[600px]">
                        {mounted && <WalletList />}
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
      flex items-center gap-3 px-5 py-4 rounded-2xl transition-all cursor-pointer group relative
      ${active ? 'bg-cyan-500/10 text-cyan-400 shadow-inner' : 'text-slate-500 hover:text-slate-200 hover:bg-white/5'}
    `}>
            {active && <motion.div layoutId="nav-active" className="absolute inset-0 bg-cyan-500/5 rounded-2xl border border-cyan-500/20 shadow-[0_0_20px_rgba(34,211,238,0.05)]" />}
            <span className="relative z-10 group-hover:scale-110 transition-transform">{icon}</span>
            <span className="font-bold text-sm tracking-tight relative z-10">{label}</span>
            {active && <div className="ml-auto w-1.5 h-1.5 rounded-full bg-cyan-400 shadow-[0_0_8px_rgba(34,211,238,0.8)]" />}
        </div>
    );

    if (href) {
        return <Link href={href}>{content}</Link>;
    }

    return content;
}
