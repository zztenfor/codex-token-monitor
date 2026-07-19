import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { BarChart3, CircleDollarSign, Pause, PictureInPicture2, Play, Settings as SettingsIcon, ShieldCheck } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Dashboard } from "./features/Dashboard";
import { Cost } from "./features/Cost";
import { FloatingWidget } from "./features/FloatingWidget";
import { Settings } from "./features/Settings";
import i18n, { detectSystemLanguage, normalizeLanguage } from "./i18n";
import { translateError } from "./i18n/errors";
import { readUsageRange, saveUsageRange, type UsageRange } from "./lib/preferences";
import { api } from "./lib/tauri";
import type { AppSettings, DashboardData } from "./lib/types";
import { PricingService, type PricingSnapshot } from "./services/pricing";

const pricingService = new PricingService();

const defaults: AppSettings = {
  codexPath: "",
  refreshIntervalSeconds: 5,
  theme: "system",
  language: detectSystemLanguage(),
  floatingOpacity: 0.85,
  floatingAlwaysOnTop: true,
  floatingClickThrough: false,
  floatingMode: "compact",
  dailyBudget: 0,
  monthlyBudget: 0,
  fiveHourLimit: 0,
  weeklyLimit: 0,
  alert80: true,
  alert90: true,
  alert95: true,
};

function MainApp() {
  const { t } = useTranslation();
  const [view, setView] = useState<"dashboard" | "cost" | "settings">("dashboard");
  const [range, setRange] = useState<UsageRange>(readUsageRange);
  const rangeRef = useRef<UsageRange>(range);
  const loadRequestRef = useRef(0);
  const [data, setData] = useState<DashboardData | null>(null);
  const [settings, setSettings] = useState(defaults);
  const [loading, setLoading] = useState(false);
  const [quotaLoading, setQuotaLoading] = useState(false);
  const [error, setError] = useState("");
  const [pricing, setPricing] = useState<PricingSnapshot>(() => pricingService.getSnapshot());
  const [pricingLoading, setPricingLoading] = useState(false);

  const load = useCallback(async () => {
    const requestedRange = rangeRef.current;
    const requestId = ++loadRequestRef.current;
    try {
      const next = await api.dashboard(requestedRange);
      if (requestId !== loadRequestRef.current || requestedRange !== rangeRef.current) return;
      setData(next);
      setError("");
    } catch (cause) {
      if (requestId !== loadRequestRef.current || requestedRange !== rangeRef.current) return;
      setError(translateError(cause, t));
    }
  }, [t]);

  const selectRange = useCallback((nextRange: UsageRange) => {
    rangeRef.current = nextRange;
    saveUsageRange(nextRange);
    setRange(nextRange);
  }, []);

  const sync = useCallback(async () => {
    setLoading(true);
    try {
      await api.syncNow();
      await load();
    } catch (cause) {
      setError(translateError(cause, t, "errors.sync"));
    } finally {
      setLoading(false);
    }
  }, [load, t]);

  const syncQuota = useCallback(async () => {
    setQuotaLoading(true);
    try {
      await api.syncAccountQuota();
      await load();
      setError("");
    } catch (cause) {
      setError(translateError(cause, t, "errors.quota"));
    } finally {
      setQuotaLoading(false);
    }
  }, [load, t]);

  const refreshPricing = useCallback(async () => {
    setPricingLoading(true);
    const next = await pricingService.syncPricing();
    setPricing(next);
    setPricingLoading(false);
  }, []);

  useEffect(() => {
    api.settings().then(async (saved) => {
      const language = normalizeLanguage(saved.language || detectSystemLanguage());
      await i18n.changeLanguage(language);
      setSettings({ ...saved, language });
      if (!saved.language) await api.saveSettings({ ...saved, language });
    }).catch((cause) => setError(translateError(cause, t)));
    void sync();
    void pricingService.initialize().then((snapshot) => {
      setPricing(snapshot);
      if (pricingService.needsSync()) void refreshPricing();
    });
  }, [refreshPricing]);

  useEffect(() => {
    void load();
  }, [range]);

  useEffect(() => {
    const dark = settings.theme === "dark" || (settings.theme === "system" && matchMedia("(prefers-color-scheme: dark)").matches);
    document.documentElement.dataset.theme = dark ? "dark" : "light";
    document.documentElement.lang = settings.language;
  }, [settings]);

  useEffect(() => {
    let stopUsage: (() => void) | undefined;
    let stopQuota: (() => void) | undefined;
    let stopSettings: (() => void) | undefined;
    let stopLanguage: (() => void) | undefined;
    listen("usage-updated", () => void load()).then((unlisten) => (stopUsage = unlisten));
    listen("account-quota-updated", () => void load()).then((unlisten) => (stopQuota = unlisten));
    listen("open-settings", () => setView("settings")).then((unlisten) => (stopSettings = unlisten));
    listen<string>("language-changed", (event) => {
      const language = normalizeLanguage(event.payload);
      void i18n.changeLanguage(language);
      setSettings((current) => ({ ...current, language }));
    }).then((unlisten) => (stopLanguage = unlisten));
    return () => {
      stopUsage?.();
      stopQuota?.();
      stopSettings?.();
      stopLanguage?.();
    };
  }, [load]);

  const pause = async () => {
    try {
      const paused = await api.setPaused(!data?.syncPaused);
      setData((current) => (current ? { ...current, syncPaused: paused } : current));
    } catch (cause) {
      setError(translateError(cause, t));
    }
  };

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand"><span className="brand-mark">C</span><div><strong>{t("app.brand")}</strong><span>{t("app.subtitle")}</span></div></div>
        <nav>
          <button className={view === "dashboard" ? "active" : ""} onClick={() => setView("dashboard")}><BarChart3 size={19} />{t("nav.dashboard")}</button>
          <button className={view === "cost" ? "active" : ""} onClick={() => setView("cost")}><CircleDollarSign size={19} />{t("nav.cost")}</button>
          <button className={view === "settings" ? "active" : ""} onClick={() => setView("settings")}><SettingsIcon size={19} />{t("nav.settings")}</button>
        </nav>
        <div className="sidebar-bottom">
          <button aria-label={t("common.openFloatingWindow")} onClick={() => void api.toggleWidget()}><PictureInPicture2 size={18} />{t("common.openFloatingWindow")}</button>
          <button onClick={() => void pause()}>{data?.syncPaused ? <Play size={18} /> : <Pause size={18} />}{data?.syncPaused ? t("common.resumeSync") : t("common.pauseSync")}</button>
          <div className="local-badge"><ShieldCheck size={17} /><div><strong>{t("common.localOnly")}</strong><span>{t("common.localOnlyDescription")}</span></div></div>
        </div>
      </aside>
      <div className="content">
        {error && <div className="error-banner">{error}</div>}
        {view === "dashboard" ? <Dashboard data={data} range={range} loading={loading} onRange={selectRange} onSync={sync} quotaLoading={quotaLoading} onQuotaSync={syncQuota} /> : view === "cost" ? <Cost data={data} pricing={pricing} range={range} onRange={selectRange} onRefresh={() => void refreshPricing()} loading={pricingLoading} /> : <Settings settings={settings} pricing={pricing} pricingLoading={pricingLoading} onRefreshPricing={() => void refreshPricing()} onSaved={(next) => { setSettings(next); void load(); }} />}
      </div>
    </div>
  );
}

export default function App() {
  return getCurrentWindow().label === "widget" ? <FloatingWidget /> : <MainApp />;
}
