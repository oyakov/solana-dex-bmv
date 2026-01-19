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
  | "healthy"
  | "terminalAccess"
  | "authCredentials"
  | "enterPassword"
  | "establishConnection"
  | "authorizing"
  | "invalidPassword"
  | "connectivityFailure"
  | "encryptedSession"
  | "back"
  | "activeNodes"
  | "injectWallet"
  | "abort"
  | "initialize"
  | "hardwareManagement"
  | "multiSigLayer"
  | "synchronizingSwarm"
  | "secretKey"
  | "enterSecretKey"
  | "walletCaution"
  | "publicAddress"
  | "masterNode"
  | "infrastructureLatency"
  | "networkObservability"
  | "temporalLatency"
  | "latencyMonitoring"
  | "returnToCommandCenter"
  | "unknown"
  | "distributionSolana"
  | "liveConcentration"
  | "totalSupply"
  | "bmvTokens"
  | "top10Concentration"
  | "healthyDistribution"
  | "combinedHoldings"
  | "largestHolder"
  | "whaleAlert"
  | "top20Holders"
  | "rank"
  | "address"
  | "balance"
  | "share"
  | "distributionOverview"
  | "others"
  | "highConcentration"
  | "highConcentrationDesc"
  | "distributionHealth"
  | "excellent"
  | "good"
  | "fair"
  | "marketEvolution"
  | "configuration"
  | "marketScenario"
  | "basePrice"
  | "steps"
  | "volatility"
  | "running"
  | "runSimulation"
  | "simulationResults"
  | "buyOrders"
  | "sellOrders"
  | "avgSpread"
  | "priceRange"
  | "projectedOrders"
  | "size"
  | "upwardSaw"
  | "downwardSaw"
  | "sideways"
  | "flashCrash"
  | "pumpAndDump"
  | "gradualRise"
  | "upwardSawDesc"
  | "downwardSawDesc"
  | "sidewaysDesc"
  | "flashCrashDesc"
  | "pumpAndDumpDesc"
  | "gradualRiseDesc"
  | "backtestingMarketScenarios"
  | "simulationEngineActive"
  | "configureAndRun"
  | "moreOrders"
  | "side"
  | "price"
  | "step"
  | "buyOrder"
  | "sellOrder";

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
    terminalAccess: "ДОСТУП К ТЕРМИНАЛУ",
    authCredentials: "Учетные данные для аутентификации",
    enterPassword: "Введите пароль системы...",
    establishConnection: "Установить соединение",
    authorizing: "Авторизация...",
    invalidPassword: "Неверный пароль. Система заблокирована.",
    connectivityFailure: "Ошибка связи. Терминал недоступен.",
    encryptedSession: "Зашифрованная сессия",
    back: "Назад",
    activeNodes: "{count} УЗЛОВ",
    injectWallet: "Добавить кошелек",
    abort: "Отмена",
    initialize: "Инициализировать",
    hardwareManagement: "Управление на аппаратном уровне и ротация",
    multiSigLayer: "Уровень Multi-Sig активен",
    synchronizingSwarm: "Синхронизация роя...",
    secretKey: "Секретный ключ Base58",
    enterSecretKey: "Введите секретный ключ кошелька...",
    walletCaution: "Внимание: Добавленные кошельки становятся активными немедленно.",
    publicAddress: "Публичный адрес",
    masterNode: "Мастер-узел",
    infrastructureLatency: "Задержка инфраструктуры",
    networkObservability: "Сетевая наблюдаемость",
    temporalLatency: "Временное распределение задержек",
    latencyMonitoring: "Мониторинг производительности в реальном времени",
    returnToCommandCenter: "Вернуться в командный центр",
    unknown: "НЕИЗВЕСТНО",
    distributionSolana: "Распределение BMV через Solana RPC",
    liveConcentration: "Данные о концентрации",
    totalSupply: "Общее предложение",
    bmvTokens: "Токенов BMV",
    healthyDistribution: "Здоровое распределение",
    combinedHoldings: "Совокупные владения",
    largestHolder: "Крупнейший держатель",
    whaleAlert: "Внимание: Кит",
    top20Holders: "Топ-20 держателей",
    rank: "Ранг",
    address: "Адрес",
    balance: "Баланс",
    share: "Доля",
    distributionOverview: "Обзор распределения",
    others: "Прочие",
    highConcentration: "Высокая концентрация",
    highConcentrationDesc: "Топ-10 держателей контролируют более 50% предложения.",
    distributionHealth: "Здоровье распределения",
    excellent: "Отлично",
    good: "Хорошо",
    fair: "Средне",
    marketEvolution: "Эволюция рынка и плотность сетки",
    configuration: "Конфигурация",
    marketScenario: "Рыночный сценарий",
    basePrice: "Базовая цена (SOL)",
    steps: "Шаги",
    volatility: "Волатильность",
    running: "Запуск...",
    runSimulation: "Запустить симуляцию",
    simulationResults: "Результаты симуляции",
    buyOrders: "Ордера на покупку",
    sellOrders: "Ордера на продажу",
    avgSpread: "Средний спред",
    priceRange: "Диапазон цен",
    projectedOrders: "Прогнозные ордера",
    size: "Размер",
    upwardSaw: "Восходящая пила",
    downwardSaw: "Нисходящая пила",
    sideways: "Боковик",
    flashCrash: "Флэш-крэш",
    pumpAndDump: "Памп и дамп",
    gradualRise: "Постепенный рост",
    upwardSawDesc: "Постепенный аптренд с откатами",
    downwardSawDesc: "Постепенный даунтренд с отскоками",
    sidewaysDesc: "Движение в диапазоне",
    flashCrashDesc: "Внезапный обвал и восстановление",
    pumpAndDumpDesc: "Резкий рост, затем распродажа",
    gradualRiseDesc: "Устойчивое движение вверх",
    backtestingMarketScenarios: "Тестирование рыночных сценариев",
    simulationEngineActive: "Движок симуляции активен",
    configureAndRun: "Настройте и запустите симуляцию для просмотра результатов",
    moreOrders: "+ {count} еще ордеров",
    side: "Сторона",
    price: "Цена",
    step: "Шаг",
    buyOrder: "Ордер на покупку",
    sellOrder: "Ордер на продажу",
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
    terminalAccess: "TERMINAL ACCESS",
    authCredentials: "Authentication Credentials",
    enterPassword: "Enter system password...",
    establishConnection: "Establish Connection",
    authorizing: "Authorizing...",
    invalidPassword: "Invalid access password. System locked.",
    connectivityFailure: "Connectivity failure. Cannot reach terminal.",
    encryptedSession: "Encrypted Session",
    back: "Back",
    activeNodes: "{count} NODES",
    injectWallet: "Inject Wallet",
    abort: "Abort",
    initialize: "Initialize",
    hardwareManagement: "Hardware-Level Management & Rotation",
    multiSigLayer: "Multi-Sig Layer Active",
    synchronizingSwarm: "Synchronizing Swarm...",
    secretKey: "Base58 Secret Key",
    enterSecretKey: "Enter wallet secret key...",
    walletCaution: "Caution: Injected wallets are active immediately and participate in automated cycles.",
    publicAddress: "Public Address",
    masterNode: "Master Node",
    infrastructureLatency: "Infrastructure Latency",
    networkObservability: "Network Observability",
    temporalLatency: "Temporal Latency Distribution",
    realTimeMonitoring: "Real-time dependency performance monitoring",
    returnToCommandCenter: "Return to Command Center",
    unknown: "UNKNOWN",
    distributionSolana: "BMV Distribution via Solana RPC",
    liveConcentration: "Live Concentration Data",
    totalSupply: "Total Supply",
    bmvTokens: "BMV Tokens",
    healthyDistribution: "Healthy Distribution",
    combinedHoldings: "Combined Holdings",
    largestHolder: "Largest Holder",
    whaleAlert: "Whale Alert",
    top20Holders: "Top 20 Token Holders",
    rank: "Rank",
    address: "Address",
    balance: "Balance",
    share: "Share",
    distributionOverview: "Distribution Overview",
    others: "Others",
    highConcentration: "High Concentration",
    highConcentrationDesc: "Top 10 holders control over 50% of supply. Distribution may be centralized.",
    distributionHealth: "Distribution Health",
    excellent: "Excellent",
    good: "Good",
    fair: "Fair",
    marketEvolution: "Market Evolution & Grid Density",
    configuration: "Configuration",
    marketScenario: "Market Scenario",
    basePrice: "Base Price (SOL)",
    steps: "Steps",
    volatility: "Volatility",
    running: "Running...",
    runSimulation: "Run Simulation",
    simulationResults: "Simulation Results",
    buyOrders: "Buy Orders",
    sellOrders: "Sell Orders",
    avgSpread: "Avg Spread",
    priceRange: "Price Range",
    projectedOrders: "Projected Orders",
    size: "Size",
    upwardSaw: "Upward Saw",
    downwardSaw: "Downward Saw",
    sideways: "Sideways",
    flashCrash: "Flash Crash",
    pumpAndDump: "Pump & Dump",
    gradualRise: "Gradual Rise",
    upwardSawDesc: "Gradual uptrend with pullbacks",
    downwardSawDesc: "Gradual downtrend with bounces",
    sidewaysDesc: "Range-bound movement",
    flashCrashDesc: "Sudden price collapse & recovery",
    pumpAndDumpDesc: "Sharp rise then selloff",
    gradualRiseDesc: "Steady upward movement",
    backtestingMarketScenarios: "Backtest Market Scenarios",
    simulationEngineActive: "Simulation Engine Active",
    configureAndRun: "Configure and run a simulation to see results",
    moreOrders: "+ {count} more orders",
    side: "Side",
    price: "Price",
    step: "Step",
    buyOrder: "Buy Order",
    sellOrder: "Sell Order",
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
