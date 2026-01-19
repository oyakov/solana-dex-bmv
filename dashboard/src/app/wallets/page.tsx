"use client";

import React, { useState, useEffect } from "react";
import {
    Shield,
    Database,
    Clock,
    ArrowLeft
} from "lucide-react";
import Link from "next/link";
import WalletList from "@/components/WalletList";
import Sidebar from "../../components/Sidebar";
import { useLanguage } from "../../components/LanguageProvider";


export default function WalletsPage() {
    const { t } = useLanguage();
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
                        <div className="flex items-center gap-2 mb-2">
                            <Link href="/" className="p-1 px-2 bg-white/5 rounded-lg border border-white/5 hover:bg-white/10 text-slate-400 hover:text-white transition-all text-[10px] font-black uppercase tracking-widest flex items-center gap-1">
                                <ArrowLeft size={10} />
                                {t("back")}
                            </Link>
                        </div>
                        <h2 className="text-4xl font-black tracking-tight mb-1">{t("walletSwarm")}</h2>
                        <p className="text-slate-400 text-sm flex items-center gap-2 font-medium">
                            <Database size={14} className="text-blue-400" />
                            {t("hardwareManagement")}
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
                                {t("multiSigLayer")}
                            </span>
                        </div>
                    </div>
                </div>

                {/* Wallet List Component */}
                <div className="glass-panel rounded-[2.5rem] p-10 border border-white/5 relative overflow-hidden min-h-[600px]">
                    {mounted && <WalletList />}
                </div>
            </main>
        </div>
    );
}
