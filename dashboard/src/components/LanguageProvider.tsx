"use client";

import React, { createContext, useCallback, useContext, useEffect, useMemo, useState } from "react";

type Language = "ru" | "en";

type TranslationKey =
  | "language"
  | "commandCenter"
  | "latencyReport"
  | "simulationLab"
  | "walletSwarm"
  | "tokenHolders"
  | "pnlEngine"
  | "protocolConfig"
  | "logout"
  | "systemStatus"
  | "connected"
  | "tradingCommandCenter"
  | "liveFromSolana"
  | "deployGrid"
  | "assetPivot"
  | "seededVwapMidPoint"
  | "solBalance"
  | "totalSwarmReserve"
  | "usdcBalance"
  | "stablecoinLiquidity"
  | "solUsdc"
  | "marketBaseline"
  | "whaleIndex"
  | "concentrationTop10"
  | "orderImbalance"
  | "bidDominance"
  | "askDominance"
  | "safeHavenIndex"
  | "betaVsSolana"
  | "marketSpread"
  | "realTimeLiquidityGap"
  | "tight"
  | "wide"
  | "nodeStatus"
  | "online"
  | "active"
  | "multiWalletRotation"
  | "channelWidth"
  | "dynamicVolatilityBound"
  | "protected"
  | "dryRunMode"
  | "assetPriceHistory"
  | "solUsdcCorrelation"
  | "tacticalControl"
  | "forceRebalance"
  | "killSwitch"
  | "orderBookV1"
  | "live"
  | "liquidityConcentration"
  | "targetLevel"
  | "resist90"
  | "resist50"
  | "support50"
  | "support90"
  | "bmvBasePrice"
  | "solCorrelation"
  | "actionTriggered"
  | "liquid"
  | "ready"
  | "centralized"
  | "healthy";

const translations: Record<Language, Record<TranslationKey, string>> = {
  ru: {
    language: "Язык",
    commandCenter: "Командный центр",
    latencyReport: "Отчет задержек",
    simulationLab: "Лаборатория симуляций",
    walletSwarm: "Рой кошельков",
    tokenHolders: "Держатели токенов",
    pnlEngine: "Движок PnL",
    protocolConfig: "Настройки протокола",
    logout: "Выйти",
    systemStatus: "Статус системы",
    connected: "Подключено",
    tradingCommandCenter: "Торговый командный центр",
    liveFromSolana: "Онлайн с Solana Mainnet Beta",
    deployGrid: "Запуск сетки",
    assetPivot: "Опорная цена",
    seededVwapMidPoint: "Опорная точка VWAP",
    solBalance: "Баланс SOL",
    totalSwarmReserve: "Общий резерв роя",
    usdcBalance: "Баланс USDC",
    stablecoinLiquidity: "Ликвидность стейблкоина",
    solUsdc: "SOL/USDC",
    marketBaseline: "Базовый рынок",
    whaleIndex: "Индекс китов",
    concentrationTop10: "Концентрация в топ-10",
    orderImbalance: "Дисбаланс ордеров",
    bidDominance: "Преимущество bid",
    askDominance: "Преимущество ask",
    safeHavenIndex: "Индекс защиты",
    betaVsSolana: "Бета к индексу Solana",
    marketSpread: "Рыночный спред",
    realTimeLiquidityGap: "Разрыв ликвидности в реальном времени",
    tight: "Узкий",
    wide: "Широкий",
    nodeStatus: "Статус узлов",
    online: "Онлайн",
    active: "Активных",
    multiWalletRotation: "Ротация кошельков",
    channelWidth: "Ширина канала",
    dynamicVolatilityBound: "Динамический предел волатильности",
    protected: "Защищено",
    dryRunMode: "Тестовый режим",
    assetPriceHistory: "История цены актива",
    solUsdcCorrelation: "Корреляция SOL/USDC",
    tacticalControl: "Тактическое управление",
    forceRebalance: "Принудительный ребаланс",
    killSwitch: "Аварийный стоп",
    orderBookV1: "Стакан V1",
    live: "Онлайн",
    liquidityConcentration: "Концентрация ликвидности",
    targetLevel: "Целевой уровень",
    resist90: "СОПР (90%)",
    resist50: "СОПР (50%)",
    support50: "ПОДДЕРЖ (50%)",
    support90: "ПОДДЕРЖ (90%)",
    bmvBasePrice: "Базовая цена BMV",
    solCorrelation: "Корреляция SOL",
    actionTriggered: "Действие {action} выполнено",
    liquid: "Ликвидно",
    ready: "Готово",
    centralized: "Централизовано",
    healthy: "Здорово",
  },
  en: {
    language: "Language",
    commandCenter: "Command Center",
    latencyReport: "Latency Report",
    simulationLab: "Simulation Lab",
    walletSwarm: "Wallet Swarm",
    tokenHolders: "Token Holders",
    pnlEngine: "PnL Engine",
    protocolConfig: "Protocol Config",
    logout: "Logout",
    systemStatus: "System Status",
    connected: "Connected",
    tradingCommandCenter: "Trading Command Center",
    liveFromSolana: "Live from Solana Mainnet Beta",
    deployGrid: "Deploy Grid",
    assetPivot: "Asset Pivot",
    seededVwapMidPoint: "Seeded VWAP Mid-Point",
    solBalance: "SOL Balance",
    totalSwarmReserve: "Total Swarm Reserve",
    usdcBalance: "USDC Balance",
    stablecoinLiquidity: "Stablecoin Liquidity",
    solUsdc: "SOL/USDC",
    marketBaseline: "Market Baseline",
    whaleIndex: "Whale Index",
    concentrationTop10: "Concentration in Top 10",
    orderImbalance: "Order Imbalance",
    bidDominance: "Bid Dominance",
    askDominance: "Ask Dominance",
    safeHavenIndex: "Safe Haven Index",
    betaVsSolana: "Beta vs Solana Index",
    marketSpread: "Market Spread",
    realTimeLiquidityGap: "Real-time Liquidity Gap",
    tight: "Tight",
    wide: "Wide",
    nodeStatus: "Node Status",
    online: "Online",
    active: "Active",
    multiWalletRotation: "Multi-Wallet Rotation",
    channelWidth: "Channel Width",
    dynamicVolatilityBound: "Dynamic Volatility Bound",
    protected: "Protected",
    dryRunMode: "Dry Run Mode",
    assetPriceHistory: "Asset Price History",
    solUsdcCorrelation: "SOL/USDC Correlation",
    tacticalControl: "Tactical Control",
    forceRebalance: "Force Rebalance",
    killSwitch: "Kill Switch",
    orderBookV1: "Order Book V1",
    live: "Live",
    liquidityConcentration: "Liquidity Concentr.",
    targetLevel: "Target Level",
    resist90: "RESIST (90%)",
    resist50: "RESIST (50%)",
    support50: "SUPPORT (50%)",
    support90: "SUPPORT (90%)",
    bmvBasePrice: "BMV Base Price",
    solCorrelation: "SOL Correlation",
    actionTriggered: "Action {action} triggered successfully",
    liquid: "Liquid",
    ready: "Ready",
    centralized: "Centralized",
    healthy: "Healthy",
  },
};

type LanguageContextValue = {
  language: Language;
  setLanguage: (language: Language) => void;
  t: (key: TranslationKey, vars?: Record<string, string>) => string;
};

const LanguageContext = createContext<LanguageContextValue | undefined>(undefined);

const STORAGE_KEY = "bmv-language";

export function LanguageProvider({ children }: { children: React.ReactNode }) {
  const [language, setLanguageState] = useState<Language>("ru");

  useEffect(() => {
    const stored = window.localStorage.getItem(STORAGE_KEY) as Language | null;
    if (stored === "ru" || stored === "en") {
      setLanguageState(stored);
    }
  }, []);

  useEffect(() => {
    document.documentElement.lang = language;
    window.localStorage.setItem(STORAGE_KEY, language);
  }, [language]);

  const setLanguage = useCallback((next: Language) => {
    setLanguageState(next);
  }, []);

  const t = useCallback(
    (key: TranslationKey, vars?: Record<string, string>) => {
      const template = translations[language][key] ?? key;
      if (!vars) {
        return template;
      }
      return Object.entries(vars).reduce(
        (acc, [varKey, value]) => acc.replace(`{${varKey}}`, value),
        template
      );
    },
    [language]
  );

  const value = useMemo(() => ({ language, setLanguage, t }), [language, setLanguage, t]);

  return <LanguageContext.Provider value={value}>{children}</LanguageContext.Provider>;
}

export function useLanguage() {
  const context = useContext(LanguageContext);
  if (!context) {
    throw new Error("useLanguage must be used within LanguageProvider");
  }
  return context;
}
