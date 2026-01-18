"use client";

import React from "react";
import {
    Zap,
    Activity,
    Wallet,
    LayoutDashboard,
    Settings,
    BarChart3,
    LogOut,
    FlaskConical,
    Users
} from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { motion } from "framer-motion";
import { logout } from "../utils/auth";
import { useLanguage } from "./LanguageProvider";

export default function Sidebar() {
    const pathname = usePathname();
    const { language, setLanguage, t } = useLanguage();

    const buttonClasses = (active: boolean) =>
        `px-3 py-1 rounded-full text-[10px] font-black uppercase tracking-[0.2em] border transition-colors ${active ? "bg-cyan-500 text-black border-cyan-400" : "text-slate-400 border-white/10 hover:text-white"
        }`;

    return (
        <aside className="w-80 h-screen border-r border-white/5 bg-[#020617]/80 backdrop-blur-2xl flex flex-col relative z-20 shrink-0">
            <div className="p-10">
                <div className="flex items-center gap-4 mb-12 group cursor-pointer">
                    <div className="w-12 h-12 bg-cyan-500 rounded-2xl flex items-center justify-center shadow-lg shadow-cyan-500/20 group-hover:rotate-12 transition-transform duration-500">
                        <Zap className="text-black" size={24} fill="black" />
                    </div>
                    <div>
                        <h1 className="text-2xl font-black tracking-tighter leading-none">BMV.BOT</h1>
                        <p className="text-cyan-400/60 text-[10px] font-bold uppercase tracking-[0.3em] mt-1">System v0.4.8</p>
                    </div>
                </div>

                <nav className="space-y-2">
                    <NavItem icon={<LayoutDashboard size={20} />} label={t("commandCenter")} href="/" active={pathname === "/"} />
                    <NavItem icon={<Activity size={20} />} label={t("latencyReport")} href="/latency" active={pathname === "/latency"} />
                    <NavItem icon={<FlaskConical size={20} />} label={t("simulationLab")} href="/simulation" active={pathname === "/simulation"} />
                    <NavItem icon={<Wallet size={20} />} label={t("walletSwarm")} href="/wallets" active={pathname === "/wallets"} />
                    <NavItem icon={<Users size={20} />} label={t("tokenHolders")} href="/holders" active={pathname === "/holders"} />
                    <NavItem icon={<BarChart3 size={20} />} label={t("pnlEngine")} />
                    <NavItem icon={<Settings size={20} />} label={t("protocolConfig")} />
                    <div onClick={logout} className="flex items-center gap-3 px-5 py-4 rounded-2xl transition-all cursor-pointer group relative text-slate-500 hover:text-red-400 hover:bg-red-500/5 mt-4">
                        <LogOut size={20} />
                        <span className="font-bold text-sm tracking-tight relative z-10">{t("logout")}</span>
                    </div>
                </nav>

                <div className="mt-8">
                    <p className="text-[10px] font-black uppercase tracking-[0.3em] text-slate-500 mb-3">{t("language")}</p>
                    <div className="flex items-center gap-2">
                        <button
                            type="button"
                            className={buttonClasses(language === "ru")}
                            onClick={() => setLanguage("ru")}
                        >
                            RU
                        </button>
                        <button
                            type="button"
                            className={buttonClasses(language === "en")}
                            onClick={() => setLanguage("en")}
                        >
                            EN
                        </button>
                    </div>
                </div>
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
                            <span className="text-xs font-black text-slate-200 tracking-tight">{t("systemStatus")}</span>
                            <div className="flex items-center gap-1.5 mt-0.5">
                                <div className="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse" />
                                <p className="text-white/40 text-[10px] font-bold uppercase tracking-wider">{t("connected")}</p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </aside>
    );
}

function NavItem({ icon, label, active = false, href }: { icon: React.ReactNode, label: string, active?: boolean, href?: string }) {
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
