"use client";

import React, { useState } from "react";
import { Flame, Lock, ArrowRight, ShieldAlert } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { useRouter } from "next/navigation";

export default function LoginPage() {
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const router = useRouter();

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError("");

    try {
      const response = await fetch(`/api/login`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ password }),
      });

      if (response.ok) {
        const { token } = await response.json();
        localStorage.setItem("bmv_auth_token", token);
        // Also set a cookie for middleware (simplified for this task)
        document.cookie = `bmv_auth_token=${token}; path=/; max-age=86400; SameSite=Strict`;
        router.push("/");
      } else {
        setError("Invalid access password. System locked.");
      }
    } catch (err) {
      setError("Connectivity failure. Cannot reach terminal.");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-[#020617] text-slate-100 font-sans selection:bg-cyan-500/30 overflow-hidden relative">
      {/* Background Decor */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[800px] h-[800px] bg-cyan-500/10 blur-[160px] rounded-full animate-pulse" />
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-purple-500/5 blur-[120px] rounded-full animate-pulse delay-1000" />
      </div>

      <motion.div 
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="w-full max-w-md z-10 p-8"
      >
        <div className="glass-panel p-10 rounded-[2.5rem] border border-white/5 shadow-2xl relative overflow-hidden backdrop-blur-2xl">
          <div className="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-transparent via-cyan-500/50 to-transparent" />
          
          <div className="flex flex-col items-center mb-10">
            <div className="p-4 bg-gradient-to-br from-cyan-400 to-blue-600 rounded-2xl shadow-xl shadow-cyan-500/20 mb-6 group cursor-pointer active:scale-95 transition-transform">
              <Flame className="w-10 h-10 text-white" />
            </div>
            <h1 className="text-3xl font-black tracking-tighter mb-2 bg-clip-text text-transparent bg-gradient-to-r from-white to-white/60">
              TERMINAL ACCESS
            </h1>
            <p className="text-[10px] uppercase tracking-[0.3em] font-bold text-cyan-400 opacity-80">BMV ECO SYSTEM COMMAND</p>
          </div>

          <form onSubmit={handleLogin} className="space-y-6">
            <div className="space-y-2">
              <label className="text-[10px] font-black uppercase tracking-widest text-slate-500 ml-1">Authentication Credentials</label>
              <div className="relative group">
                <div className="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
                  <Lock size={18} className="text-slate-500 group-focus-within:text-cyan-400 transition-colors" />
                </div>
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="Enter system password..."
                  required
                  className="w-full bg-white/5 border border-white/5 rounded-2xl py-4 pl-12 pr-4 text-sm font-medium focus:outline-none focus:ring-2 focus:ring-cyan-500/20 focus:border-cyan-500/30 transition-all placeholder:text-slate-600"
                />
              </div>
            </div>

            <AnimatePresence mode="wait">
              {error && (
                <motion.div 
                  initial={{ opacity: 0, height: 0 }}
                  animate={{ opacity: 1, height: 'auto' }}
                  exit={{ opacity: 0, height: 0 }}
                  className="flex items-center gap-2 p-4 bg-red-500/10 border border-red-500/20 rounded-xl text-red-400 text-xs font-bold"
                >
                  <ShieldAlert size={16} />
                  <span>{error}</span>
                </motion.div>
              )}
            </AnimatePresence>

            <button
              type="submit"
              disabled={loading}
              className="w-full py-4 bg-white text-[#020617] font-black text-xs uppercase tracking-[0.2em] rounded-2xl hover:bg-cyan-400 transition-colors disabled:opacity-50 flex items-center justify-center gap-2 group"
            >
              {loading ? (
                <span className="animate-pulse">Authorizing...</span>
              ) : (
                <>
                  Establish Connection
                  <ArrowRight size={16} className="group-hover:translate-x-1 transition-transform" />
                </>
              )}
            </button>
          </form>

          <div className="mt-8 pt-6 border-t border-white/5 flex flex-col items-center gap-2">
            <p className="text-[9px] text-slate-600 font-bold uppercase tracking-widest">Encrypted Session v4.0.0</p>
            <div className="flex gap-1">
              <div className="w-1 h-1 rounded-full bg-cyan-500/40" />
              <div className="w-1 h-1 rounded-full bg-cyan-500/20" />
              <div className="w-1 h-1 rounded-full bg-cyan-500/10" />
            </div>
          </div>
        </div>
      </motion.div>
    </div>
  );
}
