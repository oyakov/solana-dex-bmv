"use client";

import React, { useState, useEffect, useMemo } from "react";
import {
    Users,
    Zap,
    Globe,
    Clock,
    TrendingUp,
    AlertTriangle,
    ExternalLink,
    PieChart,
    Crown,
    Database,
    ArrowLeft
} from "lucide-react";
import Link from "next/link";
import { motion, AnimatePresence } from "framer-motion";
import Sidebar from "../../components/Sidebar";
import { getAuthHeaders } from "../../utils/auth";

interface TokenHolder {
    rank: number;
    address: string;
    ata_address: string;
    balance: number;
    balance_formatted: number;
    percent_of_supply: number;
    is_whale: boolean;
}

interface TokenHoldersResponse {
    holders: TokenHolder[];
    total_supply: number;
    total_supply_formatted: number;
    top_10_concentration: number;
    top_20_concentration: number;
    largest_holder_percent: number;
}

export default function HoldersPage() {
    const { t } = useLanguage();
    const [data, setData] = useState<TokenHoldersResponse | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
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

    useEffect(() => {
        const fetchHolders = async () => {
            try {
                const res = await fetch(`/api/holders`, {
                    headers: getAuthHeaders(),
                });

                if (res.status === 401) {
                    window.location.href = "/login";
                    return;
                }

                if (res.ok) {
                    const responseData = await res.json();
                    setData(responseData);
                } else {
                    setError("Failed to fetch holder data");
                }
            } catch (err) {
                console.error("Failed to fetch holders:", err);
                setError("Network error");
            } finally {
                setLoading(false);
            }
        };

        fetchHolders();
        // Refresh every 30 seconds
        const interval = setInterval(fetchHolders, 30000);
        return () => clearInterval(interval);
    }, []);

    const formatAddress = (addr: string) => {
        if (!addr || addr.length < 12) return addr;
        return `${addr.slice(0, 6)}...${addr.slice(-6)}`;
    };

    const formatBalance = (balance: number) => {
        if (balance >= 1_000_000) {
            return `${(balance / 1_000_000).toFixed(2)}M`;
        }
        if (balance >= 1_000) {
            return `${(balance / 1_000).toFixed(2)}K`;
        }
        return balance.toFixed(2);
    };

    // Calculate distribution data for pie chart visualization
    const distributionData = useMemo(() => {
        if (!data?.holders) return [];
        const top5 = data.holders.slice(0, 5);
        const rest = data.holders.slice(5);
        const restPercent = rest.reduce((sum, h) => sum + h.percent_of_supply, 0);
        const otherPercent = 100 - data.top_20_concentration;

        return [
            ...top5.map((h) => ({
                label: `#${h.rank}`,
                percent: h.percent_of_supply,
                color: h.is_whale ? "bg-rose-500" : "bg-cyan-500"
            })),
            { label: "#6-20", percent: restPercent, color: "bg-purple-500" },
            { label: "Others", percent: otherPercent, color: "bg-slate-700" }
        ];
    }, [data]);

    return (
        <div className="flex h-screen overflow-hidden bg-[#020617] text-slate-100 font-sans selection:bg-cyan-500/30">
            {/* Dynamic Background */}
            <div className="absolute inset-0 overflow-hidden pointer-events-none">
                <div className="absolute -top-[20%] -left-[10%] w-[60%] h-[60%] bg-purple-500/10 blur-[120px] rounded-full animate-pulse" />
                <div className="absolute -bottom-[20%] -right-[10%] w-[50%] h-[50%] bg-cyan-500/10 blur-[120px] rounded-full animate-pulse delay-1000" />
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
                                    {t("back")}
                                </Link>
                            </div>
                            <h2 className="text-3xl font-black tracking-tight mb-1">{t("tokenHolders")}</h2>
                            <p className="text-slate-400 text-sm flex items-center gap-2">
                                <Globe size={14} className="text-purple-400" />
                                {t("distributionSolana")}
                            </p>
                        </div>
                        <div className="flex items-center gap-3">
                            <div className="flex items-center gap-2 px-4 py-2 bg-white/5 rounded-full border border-white/5">
                                <Clock size={14} className="text-slate-500" />
                                <span className="text-xs font-mono text-slate-300">
                                    {mounted ? time : "00:00:00"}
                                </span>
                            </div>
                            <div className="px-5 py-2 bg-purple-500/10 border border-purple-500/20 rounded-full">
                                <span className="text-[10px] font-black uppercase tracking-widest text-purple-400 flex items-center gap-2">
                                    <PieChart size={10} />
                                    {t("liveConcentration")}
                                </span>
                            </div>
                        </div>
                    </div>

                    {loading ? (
                        <div className="glass-panel rounded-[2rem] p-12 border border-white/5 flex items-center justify-center">
                            <div className="animate-spin w-8 h-8 border-2 border-cyan-400 border-t-transparent rounded-full" />
                        </div>
                    ) : error ? (
                        <div className="glass-panel rounded-[2rem] p-12 border border-red-500/20 flex items-center justify-center gap-3 text-red-400">
                            <AlertTriangle size={24} />
                            <span className="font-bold">{error}</span>
                        </div>
                    ) : (
                        <>
                            {/* Metrics Cards */}
                            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
                                <MetricCard
                                    label={t("totalSupply")}
                                    value={formatBalance(data?.total_supply_formatted || 0)}
                                    subValue={t("bmvTokens")}
                                    icon={<Database className="text-cyan-400" />}
                                />
                                <MetricCard
                                    label={t("top10Concentration")}
                                    value={`${(data?.top_10_concentration || 0).toFixed(1)}%`}
                                    subValue={data && data.top_10_concentration > 50 ? t("centralized") : t("healthyDistribution")}
                                    icon={<Users className="text-purple-400" />}
                                    status={data && data.top_10_concentration > 50 ? "warning" : "healthy"}
                                />
                                <MetricCard
                                    label={t("top20Concentration")}
                                    value={`${(data?.top_20_concentration || 0).toFixed(1)}%`}
                                    subValue={t("combinedHoldings")}
                                    icon={<PieChart className="text-blue-400" />}
                                />
                                <MetricCard
                                    label={t("largestHolder")}
                                    value={`${(data?.largest_holder_percent || 0).toFixed(2)}%`}
                                    subValue={t("whaleAlert")}
                                    icon={<Crown className="text-yellow-400" />}
                                    status={data && data.largest_holder_percent > 10 ? "warning" : "healthy"}
                                />
                            </div>

                            <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
                                {/* Holders Table */}
                                <div className="lg:col-span-2 glass-panel rounded-[2rem] p-8 border border-white/5">
                                    <h3 className="text-xl font-black mb-6 flex items-center gap-3">
                                        <div className="w-1.5 h-6 bg-purple-400 rounded-full" />
                                        {t("top20Holders")}
                                    </h3>
                                    <div className="overflow-x-auto">
                                        <table className="w-full">
                                            <thead>
                                                <tr className="border-b border-white/5">
                                                    <th className="text-[10px] font-black uppercase tracking-widest text-slate-500 py-3 text-left">{t("rank")}</th>
                                                    <th className="text-[10px] font-black uppercase tracking-widest text-slate-500 py-3 text-left">{t("address")}</th>
                                                    <th className="text-[10px] font-black uppercase tracking-widest text-slate-500 py-3 text-right">{t("balance")}</th>
                                                    <th className="text-[10px] font-black uppercase tracking-widest text-slate-500 py-3 text-right">{t("share")}</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {data?.holders.map((holder, idx) => (
                                                    <motion.tr
                                                        key={holder.address}
                                                        initial={{ opacity: 0, y: 10 }}
                                                        animate={{ opacity: 1, y: 0 }}
                                                        transition={{ delay: idx * 0.03 }}
                                                        className={`border-b border-white/5 hover:bg-white/5 transition-colors ${holder.is_whale ? 'bg-rose-500/5' : ''}`}
                                                    >
                                                        <td className="py-4">
                                                            <div className="flex items-center gap-2">
                                                                {holder.rank <= 3 ? (
                                                                    <div className={`w-6 h-6 rounded-full flex items-center justify-center text-[10px] font-black ${holder.rank === 1 ? 'bg-yellow-500/20 text-yellow-400' : holder.rank === 2 ? 'bg-slate-400/20 text-slate-300' : 'bg-orange-500/20 text-orange-400'}`}>
                                                                        {holder.rank}
                                                                    </div>
                                                                ) : (
                                                                    <span className="text-slate-400 font-mono text-sm w-6 text-center">{holder.rank}</span>
                                                                )}
                                                                {holder.is_whale && (
                                                                    <span className="px-1.5 py-0.5 bg-rose-500/20 text-rose-400 text-[8px] font-black uppercase rounded">🐋</span>
                                                                )}
                                                            </div>
                                                        </td>
                                                        <td className="py-4">
                                                            <div className="flex items-center gap-2">
                                                                <span className="font-mono text-sm text-slate-300">{formatAddress(holder.address)}</span>
                                                                <a
                                                                    href={`https://solscan.io/account/${holder.address}`}
                                                                    target="_blank"
                                                                    rel="noopener noreferrer"
                                                                    className="text-slate-500 hover:text-cyan-400 transition-colors"
                                                                >
                                                                    <ExternalLink size={12} />
                                                                </a>
                                                            </div>
                                                        </td>
                                                        <td className="py-4 text-right">
                                                            <span className="font-mono text-sm text-slate-200">{formatBalance(holder.balance_formatted)}</span>
                                                        </td>
                                                        <td className="py-4 text-right">
                                                            <div className="flex items-center justify-end gap-2">
                                                                <div className="w-20 h-1.5 bg-white/5 rounded-full overflow-hidden">
                                                                    <motion.div
                                                                        initial={{ width: 0 }}
                                                                        animate={{ width: `${Math.min(100, holder.percent_of_supply * 2)}%` }}
                                                                        className={`h-full rounded-full ${holder.is_whale ? 'bg-rose-500' : 'bg-cyan-500'}`}
                                                                    />
                                                                </div>
                                                                <span className={`font-mono text-sm ${holder.is_whale ? 'text-rose-400' : 'text-cyan-400'}`}>
                                                                    {holder.percent_of_supply.toFixed(2)}%
                                                                </span>
                                                            </div>
                                                        </td>
                                                    </motion.tr>
                                                ))}
                                            </tbody>
                                        </table>
                                    </div>
                                </div>

                                {/* Distribution Visualization */}
                                <div className="glass-panel rounded-[2rem] p-8 border border-white/5">
                                    <h3 className="text-lg font-black mb-6 flex items-center gap-2 text-slate-300">
                                        <PieChart size={18} className="text-purple-400" />
                                        {t("distributionOverview")}
                                    </h3>

                                    {/* Stacked Bar Chart */}
                                    <div className="mb-8">
                                        <div className="h-8 w-full rounded-full overflow-hidden flex">
                                            {distributionData.map((seg, i) => (
                                                <motion.div
                                                    key={i}
                                                    initial={{ width: 0 }}
                                                    animate={{ width: `${seg.percent}%` }}
                                                    transition={{ delay: i * 0.1, duration: 0.5 }}
                                                    className={`${seg.color} h-full`}
                                                    title={`${seg.label}: ${seg.percent.toFixed(1)}%`}
                                                />
                                            ))}
                                        </div>
                                    </div>

                                    {/* Legend */}
                                    <div className="space-y-3">
                                        {distributionData.map((seg, i) => (
                                            <div key={i} className="flex items-center justify-between text-sm">
                                                <div className="flex items-center gap-2">
                                                    <div className={`w-3 h-3 rounded-full ${seg.color}`} />
                                                    <span className="text-slate-400 font-medium">{seg.label === "Others" ? t("others") : seg.label}</span>
                                                </div>
                                                <span className="font-mono text-slate-300">{seg.percent.toFixed(1)}%</span>
                                            </div>
                                        ))}
                                    </div>

                                    {/* Alert for high concentration */}
                                    {data && data.top_10_concentration > 50 && (
                                        <div className="mt-8 p-4 bg-yellow-500/10 border border-yellow-500/20 rounded-2xl">
                                            <div className="flex items-start gap-3">
                                                <AlertTriangle size={18} className="text-yellow-400 mt-0.5" />
                                                <div>
                                                    <p className="text-yellow-400 font-bold text-sm">{t("highConcentration")}</p>
                                                    <p className="text-slate-400 text-xs mt-1">
                                                        {t("highConcentrationDesc")}
                                                    </p>
                                                </div>
                                            </div>
                                        </div>
                                    )}

                                    {/* Health Indicator */}
                                    <div className="mt-8 p-4 glass-panel rounded-2xl border border-white/5">
                                        <div className="flex items-center justify-between mb-2">
                                            <span className="text-[10px] font-black uppercase tracking-widest text-slate-500">{t("distributionHealth")}</span>
                                            <span className={`text-[10px] font-black uppercase ${data && data.top_10_concentration <= 50 ? 'text-emerald-400' : 'text-yellow-400'}`}>
                                                {data && data.top_10_concentration <= 30 ? t("excellent") : data && data.top_10_concentration <= 50 ? t("good") : t("fair")}
                                            </span>
                                        </div>
                                        <div className="h-2 bg-white/5 rounded-full overflow-hidden">
                                            <motion.div
                                                initial={{ width: 0 }}
                                                animate={{ width: `${Math.min(100, 100 - (data?.top_10_concentration || 50))}%` }}
                                                className={`h-full rounded-full ${data && data.top_10_concentration <= 30 ? 'bg-emerald-500' : data && data.top_10_concentration <= 50 ? 'bg-cyan-500' : 'bg-yellow-500'}`}
                                            />
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </>
                    )}
                </div>
            </main>
        </div>
    );
}

function MetricCard({ label, value, subValue, icon, status }: {
    label: string;
    value: string;
    subValue: string;
    icon: React.ReactNode;
    status?: 'healthy' | 'warning';
}) {
    return (
        <div className="glass-panel rounded-3xl p-6 border border-white/5 hover:border-white/10 transition-all flex flex-col justify-between group">
            <div className="flex items-start justify-between mb-6">
                <div className="p-2.5 bg-white/5 rounded-xl group-hover:bg-white/10 transition-colors">{icon}</div>
                {status && (
                    <span className={`text-[10px] px-2 py-1 rounded-full font-black uppercase tracking-widest ${status === 'healthy' ? 'bg-emerald-500/10 text-emerald-400' : 'bg-yellow-500/10 text-yellow-400'}`}>
                        {status === 'healthy' ? '✓ Healthy' : '⚠ Alert'}
                    </span>
                )}
            </div>
            <div>
                <p className="text-slate-500 text-xs font-bold uppercase tracking-widest mb-1">{label}</p>
                <h4 className="text-2xl font-black tracking-tight text-slate-100">{value}</h4>
                <p className="text-[10px] text-slate-500 mt-1 font-medium">{subValue}</p>
            </div>
        </div>
    );
}
