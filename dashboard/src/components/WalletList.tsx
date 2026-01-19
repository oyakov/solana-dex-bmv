"use client";

import React, { useState, useEffect } from "react";
import { Wallet, Plus, ArrowUpRight, ArrowDownRight, RefreshCcw, Loader2, Shield, Zap } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { getAuthHeaders } from "../utils/auth";
import { useLanguage } from "./LanguageProvider";


interface WalletInfo {
    pubkey: string;
    sol_balance: number;
    usdc_balance: number;
}

export default function WalletList() {
    const { t } = useLanguage();
    const [wallets, setWallets] = useState<WalletInfo[]>([]);
    const [loading, setLoading] = useState(true);
    const [refreshing, setRefreshing] = useState(false);
    const [showAddModal, setShowAddModal] = useState(false);
    const [secret, setSecret] = useState("");
    const [isAdding, setIsAdding] = useState(false);
    const [error, setError] = useState("");

    const fetchWallets = async () => {
        setRefreshing(true);
        try {
            const response = await fetch(`/api/wallets`, {
                headers: getAuthHeaders(),
            });

            if (response.ok) {
                const data = await response.json();
                setWallets(data);
            }
        } catch (err) {
            console.error("Failed to fetch wallets:", err);
        } finally {
            setLoading(false);
            setRefreshing(false);
        }
    };

    useEffect(() => {
        fetchWallets();
        const interval = setInterval(fetchWallets, 30000);
        return () => clearInterval(interval);
    }, []);

    const handleAddWallet = async (e: React.FormEvent) => {
        e.preventDefault();
        setIsAdding(true);
        setError("");
        try {
            const res = await fetch(`/api/wallets/add`, {
                method: "POST",
                headers: getAuthHeaders(),
                body: JSON.stringify({ secret }),
            });

            const data = await res.json();
            if (data.status === "ok") {
                setShowAddModal(false);
                setSecret("");
                fetchWallets();
            } else {
                setError(data.message || "Failed to add wallet");
            }
        } catch (err) {
            setError("Network error occurred");
        } finally {
            setIsAdding(false);
        }
    };

    if (loading) {
        return (
            <div className="flex flex-col items-center justify-center p-20 space-y-4">
                <Loader2 className="w-10 h-10 text-cyan-500 animate-spin" />
                <p className="text-slate-400 font-bold tracking-widest uppercase text-xs">{t("synchronizingSwarm")}</p>
            </div>
        );
    }

    return (
        <div className="space-y-8">
            {/* Action Header */}
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                    <div className="w-1.5 h-6 bg-cyan-400 rounded-full" />
                    <h3 className="text-xl font-black">{t("activeSwarm")}</h3>
                    <span className="px-2 py-0.5 bg-cyan-500/10 text-cyan-400 text-[10px] font-black rounded-full border border-cyan-500/20">
                        {t("activeNodes", { count: wallets.length.toString() })}
                    </span>
                </div>
                <div className="flex gap-2">
                    <button
                        onClick={fetchWallets}
                        className="p-2.5 bg-white/5 rounded-xl border border-white/5 hover:bg-white/10 transition-all active:scale-95 group"
                    >
                        <RefreshCcw size={18} className={`text-slate-400 group-hover:text-white transition-colors ${refreshing ? 'animate-spin' : ''}`} />
                    </button>
                    <button
                        onClick={() => setShowAddModal(true)}
                        className="flex items-center gap-2 px-5 py-2.5 bg-cyan-500 text-black font-black text-xs uppercase tracking-widest rounded-xl hover:bg-cyan-400 transition-all active:scale-95 shadow-lg shadow-cyan-500/20"
                    >
                        <Plus size={16} strokeWidth={3} />
                        {t("injectWallet")}
                    </button>
                </div>
            </div>

            {/* Wallet Grid */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <AnimatePresence mode="popLayout">
                    {wallets.map((wallet, idx) => (
                        <motion.div
                            key={wallet.pubkey}
                            initial={{ opacity: 0, scale: 0.9, y: 20 }}
                            animate={{ opacity: 1, scale: 1, y: 0 }}
                            transition={{ delay: idx * 0.05 }}
                            className="glass-panel group relative overflow-hidden rounded-[2rem] p-6 border border-white/5 hover:border-cyan-500/30 transition-all"
                        >
                            {/* Card Background Glow */}
                            <div className="absolute -top-12 -right-12 w-24 h-24 bg-cyan-500/5 blur-3xl rounded-full group-hover:bg-cyan-500/10 transition-all" />

                            <div className="flex items-start justify-between mb-6">
                                <div className="p-3 bg-white/5 rounded-2xl border border-white/5 group-hover:bg-cyan-500/10 group-hover:border-cyan-500/20 transition-all">
                                    <Wallet className="text-slate-400 group-hover:text-cyan-400 transition-colors" size={24} />
                                </div>
                                <div className="text-right">
                                    <p className="text-[10px] text-slate-500 font-black uppercase tracking-[0.2em] mb-1">Index</p>
                                    <p className="text-xl font-mono font-black text-white/20 group-hover:text-cyan-400/20 transition-colors">#{idx.toString().padStart(2, '0')}</p>
                                </div>
                            </div>

                            <div className="space-y-4">
                                <div>
                                    <p className="text-[10px] text-slate-500 font-bold uppercase tracking-widest mb-1.5 opacity-60">{t("publicAddress")}</p>
                                    <div className="flex items-center gap-2">
                                        <p className="text-xs font-mono text-slate-300 break-all bg-black/20 p-2 rounded-lg border border-white/5 w-full">
                                            {wallet.pubkey}
                                        </p>
                                    </div>
                                </div>

                                <div className="grid grid-cols-2 gap-4">
                                    <div className="bg-white/5 rounded-2xl p-4 border border-white/5 group-hover:border-cyan-500/10 transition-all">
                                        <div className="flex items-center gap-2 mb-1">
                                            <div className="w-1 h-1 rounded-full bg-cyan-400" />
                                            <p className="text-[10px] text-slate-500 font-black uppercase tracking-widest">SOL</p>
                                        </div>
                                        <p className="text-lg font-black text-slate-200">{wallet.sol_balance.toFixed(4)}</p>
                                    </div>
                                    <div className="bg-white/5 rounded-2xl p-4 border border-white/5 group-hover:border-cyan-500/10 transition-all">
                                        <div className="flex items-center gap-2 mb-1">
                                            <div className="w-1 h-1 rounded-full bg-purple-400" />
                                            <p className="text-[10px] text-slate-500 font-black uppercase tracking-widest">USDC</p>
                                        </div>
                                        <p className="text-lg font-black text-slate-200">{wallet.usdc_balance.toFixed(2)}</p>
                                    </div>
                                </div>
                            </div>

                            {/* Status Indicator */}
                            <div className="mt-6 pt-6 border-t border-white/5 flex items-center justify-between">
                                <div className="flex items-center gap-2">
                                    <div className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse shadow-[0_0_8px_rgba(16,185,129,0.5)]" />
                                    <span className="text-[10px] font-black text-emerald-500 uppercase tracking-widest">{t("ready")}</span>
                                </div>
                                {idx === 0 && (
                                    <span className="text-[10px] font-black text-yellow-500 uppercase tracking-widest px-2 py-0.5 bg-yellow-500/10 rounded-full border border-yellow-500/20">
                                        {t("masterNode")}
                                    </span>
                                )}
                            </div>
                        </motion.div>
                    ))}
                </AnimatePresence>
            </div>

            {/* Add Wallet Modal */}
            <AnimatePresence>
                {showAddModal && (
                    <div className="fixed inset-0 z-50 flex items-center justify-center p-6">
                        <motion.div
                            initial={{ opacity: 0 }}
                            animate={{ opacity: 1 }}
                            exit={{ opacity: 0 }}
                            onClick={() => setShowAddModal(false)}
                            className="absolute inset-0 bg-black/80 backdrop-blur-md"
                        />
                        <motion.div
                            initial={{ scale: 0.9, opacity: 0, y: 20 }}
                            animate={{ scale: 1, opacity: 1, y: 0 }}
                            exit={{ scale: 0.9, opacity: 0, y: 20 }}
                            className="relative w-full max-w-md glass-panel rounded-[2.5rem] p-10 border border-white/10 shadow-2xl"
                        >
                            <div className="mb-8">
                                <h3 className="text-2xl font-black mb-2 flex items-center gap-3">
                                    <div className="p-2 bg-cyan-500/20 rounded-xl">
                                        <Plus className="text-cyan-400" size={24} />
                                    </div>
                                    {t("injectWallet")}
                                </h3>
                                <p className="text-slate-400 text-sm">{t("walletCaution")}</p>
                            </div>

                            <form onSubmit={handleAddWallet} className="space-y-6">
                                <div>
                                    <label className="text-[10px] font-black uppercase tracking-widest text-slate-500 ml-1 mb-2 block">
                                        {t("secretKey")}
                                    </label>
                                    <textarea
                                        required
                                        value={secret}
                                        onChange={(e) => setSecret(e.target.value)}
                                        placeholder={t("enterSecretKey")}
                                        className="w-full bg-black/40 border border-white/10 rounded-2xl p-4 text-xs font-mono text-slate-200 focus:outline-none focus:border-cyan-500/50 transition-all resize-none h-32"
                                    />
                                    {error && (
                                        <p className="text-red-400 text-[10px] font-bold mt-2 ml-1 uppercase tracking-wider flex items-center gap-1">
                                            <Shield size={10} />
                                            {error}
                                        </p>
                                    )}
                                </div>

                                <div className="flex gap-3">
                                    <button
                                        type="button"
                                        onClick={() => setShowAddModal(false)}
                                        className="flex-1 px-6 py-3.5 bg-white/5 text-slate-400 font-black text-xs uppercase tracking-widest rounded-2xl hover:bg-white/10 transition-all active:scale-95 border border-white/5"
                                    >
                                        {t("abort")}
                                    </button>
                                    <button
                                        disabled={isAdding}
                                        type="submit"
                                        className="flex-2 px-10 py-3.5 bg-cyan-500 text-black font-black text-xs uppercase tracking-widest rounded-2xl hover:bg-cyan-400 transition-all active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                                    >
                                        {isAdding ? (
                                            <Loader2 className="animate-spin" size={18} />
                                        ) : (
                                            <>
                                                <Zap size={18} fill="currentColor" />
                                                {t("initialize")}
                                            </>
                                        )}
                                    </button>
                                </div>
                            </form>

                            <div className="mt-8 p-4 bg-yellow-500/5 border border-yellow-500/10 rounded-2xl">
                                <p className="text-[9px] text-yellow-500/60 font-medium leading-relaxed uppercase tracking-wider">
                                    {t("walletCaution")}
                                </p>
                            </div>
                        </motion.div>
                    </div>
                )}
            </AnimatePresence>
        </div>
    );
}
