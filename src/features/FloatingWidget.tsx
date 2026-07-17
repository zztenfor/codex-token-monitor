import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ExternalLink, GripHorizontal, Pause, Play, X } from "lucide-react";
import { useCallback, useEffect, useState, type CSSProperties, type MouseEvent } from "react";
import { useTranslation } from "react-i18next";
import { normalizeLanguage } from "../i18n";
import { formatDuration, formatTokens, shortSession } from "../lib/format";
import { api } from "../lib/tauri";
import type { DashboardData, FloatingWindowSettings, RealQuotaWindow } from "../lib/types";

const defaultFloatingSettings: FloatingWindowSettings = {
  floatingOpacity: 0.85,
  floatingAlwaysOnTop: true,
  floatingClickThrough: false,
  floatingMode: "compact",
};

export function FloatingWidget() {
  const { t, i18n } = useTranslation();
  const [data, setData] = useState<DashboardData | null>(null);
  const [settings, setSettings] = useState<FloatingWindowSettings>(defaultFloatingSettings);
  const [darkTheme, setDarkTheme] = useState(false);
  const [now, setNow] = useState(Date.now());
  const load = useCallback(() => api.dashboard(1).then(setData).catch(() => undefined), []);
  const session = data?.currentSession;
  const percent = Math.min(session?.usagePercent ?? 0, 100);
  const language = i18n.resolvedLanguage ?? i18n.language;
  const tokens = (value: number) => formatTokens(value, language);
  const syncAge = data?.lastSyncedAt ? Math.max(0, Math.floor((now - Date.parse(data.lastSyncedAt)) / 1000)) : null;

  useEffect(() => {
    document.body.classList.add("widget-body");
    document.documentElement.classList.add("widget-html");
    api.settings().then(async (saved) => {
      await i18n.changeLanguage(normalizeLanguage(saved.language));
      setSettings({
        floatingOpacity: saved.floatingOpacity,
        floatingAlwaysOnTop: saved.floatingAlwaysOnTop,
        floatingClickThrough: saved.floatingClickThrough,
        floatingMode: saved.floatingMode,
      });
      const dark = saved.theme === "dark" || (saved.theme === "system" && matchMedia("(prefers-color-scheme: dark)").matches);
      setDarkTheme(dark);
      document.documentElement.dataset.theme = dark ? "dark" : "light";
    }).catch(() => undefined);
    void load();
    const refreshTimer = window.setInterval(() => void load(), 5000);
    const clockTimer = window.setInterval(() => setNow(Date.now()), 1000);
    let stopUsage: (() => void) | undefined;
    let stopLanguage: (() => void) | undefined;
    let stopFloating: (() => void) | undefined;
    listen("usage-updated", () => void load()).then((unlisten) => (stopUsage = unlisten));
    listen<string>("language-changed", (event) => void i18n.changeLanguage(normalizeLanguage(event.payload))).then((unlisten) => (stopLanguage = unlisten));
    listen<FloatingWindowSettings>("floating-settings-changed", (event) => setSettings(event.payload)).then((unlisten) => (stopFloating = unlisten));
    return () => {
      document.body.classList.remove("widget-body");
      document.documentElement.classList.remove("widget-html");
      window.clearInterval(refreshTimer);
      window.clearInterval(clockTimer);
      stopUsage?.();
      stopLanguage?.();
      stopFloating?.();
    };
  }, [i18n, load]);

  const pause = async () => {
    const paused = await api.setPaused(!data?.syncPaused);
    setData((current) => (current ? { ...current, syncPaused: paused } : current));
  };

  const startDragging = (event: MouseEvent<HTMLElement>) => {
    const target = event.target as Element;
    if (event.button === 0 && !target.closest("button")) void getCurrentWindow().startDragging();
  };

  const openOnDoubleClick = (event: MouseEvent<HTMLElement>) => {
    if (!(event.target as Element).closest("button")) void api.openMain();
  };

  const quotaText = (window: RealQuotaWindow | null | undefined) => window ? `${window.usedPercent.toFixed(0)}%` : t("common.notAvailable");
  const quotaReset = (window: RealQuotaWindow | null | undefined) => window ? formatDuration(window.resetAfterSeconds, t) : t("common.notAvailable");
  const fiveHour = data?.realAccountQuota.fiveHour;
  const weekly = data?.realAccountQuota.weekly;
  const widgetOpacity = Math.min(Math.max(settings.floatingOpacity, 0.2), 1);
  const widgetStyles = {
    "--bg": "transparent",
    "--surface": darkTheme ? `rgba(29, 35, 42, ${widgetOpacity})` : `rgba(255, 255, 255, ${widgetOpacity})`,
    "--surface-soft": darkTheme ? `rgba(36, 43, 51, ${Math.max(widgetOpacity - 0.08, 0.16)})` : `rgba(248, 250, 252, ${Math.max(widgetOpacity - 0.08, 0.16)})`,
    "--border": darkTheme ? `rgba(52, 61, 71, ${Math.max(widgetOpacity - 0.26, 0.18)})` : `rgba(221, 227, 234, ${Math.max(widgetOpacity - 0.26, 0.18)})`,
  } as CSSProperties;

  return (
    <main className={`floating-widget ${settings.floatingMode}`} style={widgetStyles} onDoubleClick={openOnDoubleClick}>
      <header data-tauri-drag-region onMouseDown={startDragging}>
        <div className="widget-brand" data-tauri-drag-region><span className="widget-mark">C</span><strong>{t("app.name")}</strong></div>
        <GripHorizontal className="drag-handle" size={17} data-tauri-drag-region />
        <div className="widget-actions">
          <button onClick={() => void api.openMain()} title={t("widget.openDashboard")} aria-label={t("widget.openDashboard")}><ExternalLink size={15} /></button>
          <button onClick={() => void api.hideWidget()} title={t("widget.close")} aria-label={t("widget.close")}><X size={16} /></button>
        </div>
      </header>

      {settings.floatingMode === "compact" ? <section className="widget-compact">
        <div className="widget-compact-total"><span>{t("widget.todayTotal")}</span><strong>{tokens(data?.today.totalTokens ?? 0)}</strong></div>
        <div className="widget-compact-quota"><span><b>{t("widget.fiveHourShort")}</b>{quotaText(fiveHour)}</span><span><b>{t("widget.weeklyShort")}</b>{quotaText(weekly)}</span></div>
        <div className="widget-compact-model">{session?.model || t("widget.waiting")}</div>
      </section> : <section className="widget-detailed">
        <section className="widget-block widget-status-block">
          <div className="widget-block-title">{t("widget.status")}</div>
          <div className="widget-status-grid">
            <div><span>{t("widget.model")}</span><strong>{session?.model || t("session.unknown")}</strong></div>
            <div><span>{t("widget.session")}</span><strong title={session?.sessionId}>{session ? shortSession(session.sessionId) : t("common.notAvailable")}</strong></div>
          </div>
        </section>
        <section className="widget-block">
          <div className="widget-block-title">{t("widget.tokenUsage")}</div>
          <div className="widget-token-grid">
            <div><span>{t("widget.input")}</span><strong>{tokens(data?.today.inputTokens ?? 0)}</strong></div>
            <div><span>{t("widget.cachedInput")}</span><strong>{tokens(data?.today.cachedInputTokens ?? 0)}</strong></div>
            <div><span>{t("widget.output")}</span><strong>{tokens(data?.today.outputTokens ?? 0)}</strong></div>
            <div><span>{t("widget.reasoning")}</span><strong>{tokens(data?.today.reasoningOutputTokens ?? 0)}</strong></div>
            <div className="widget-token-total"><span>{t("widget.total")}</span><strong>{tokens(data?.today.totalTokens ?? 0)}</strong></div>
          </div>
        </section>
        <section className="widget-block">
          <div className="widget-block-title">{t("widget.contextCapacity")}</div>
          <div className="widget-context-grid">
            <div><span>{t("widget.contextUsage")}</span><strong>{session?.usagePercent == null ? t("common.notAvailable") : `${percent.toFixed(1)}%`}</strong></div>
            <div><span>{t("widget.contextCapacity")}</span><strong>{session?.contextWindow == null ? t("common.notAvailable") : tokens(session.contextWindow)}</strong></div>
            <div><span>{t("widget.contextRemaining")}</span><strong>{session?.contextWindow == null ? t("common.notAvailable") : tokens(Math.max(0, session.contextWindow - session.contextTokens))}</strong></div>
          </div>
          <div className="widget-context-progress"><i style={{ width: `${Math.min(percent, 100)}%` }} /></div>
        </section>
        <section className="widget-block">
          <div className="widget-block-title">{t("widget.quotaStatus")}</div>
          <div className="widget-quota-grid">{([[t("widget.fiveHourWindow"), fiveHour], [t("widget.weeklyWindow"), weekly]] as Array<[string, RealQuotaWindow | null | undefined]>).map(([title, window]) => <div className="widget-quota" key={title}><strong>{title}</strong><span><b>{t("widget.used")}</b>{quotaText(window)}</span><span><b>{t("widget.remaining")}</b>{window ? `${window.remainingPercent.toFixed(0)}%` : t("common.notAvailable")}</span><span><b>{t("widget.reset")}</b>{quotaReset(window)}</span></div>)}</div>
        </section>
        <section className="widget-today"><span>{t("widget.todayTotal")}</span><strong>{tokens(data?.today.totalTokens ?? 0)} {t("widget.tokens")}</strong></section>
      </section>}

      {settings.floatingMode === "compact" && <div className="widget-compact-context"><span>{session?.model || t("widget.waiting")}</span><strong>{session?.usagePercent == null ? t("common.notAvailable") : `${percent.toFixed(0)}%`}</strong></div>}
      <footer className="widget-footer"><span>{t("widget.lastSync")}</span><strong>{syncAge == null ? t("widget.notSynced") : t("widget.secondsAgo", { count: syncAge })}</strong><button className="widget-pause" onClick={() => void pause()} title={data?.syncPaused ? t("widget.resume") : t("widget.pause")} aria-label={data?.syncPaused ? t("widget.resume") : t("widget.pause")}>{data?.syncPaused ? <Play size={14} /> : <Pause size={14} />}</button></footer>
    </main>
  );
}
