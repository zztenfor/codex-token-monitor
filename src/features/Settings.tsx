import { open, save } from "@tauri-apps/plugin-dialog";
import { Database, FolderOpen, RefreshCw, RotateCcw, Save, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { translateError } from "../i18n/errors";
import { api } from "../lib/tauri";
import type { AppSettings, FloatingMode, Language } from "../lib/types";
import type { PricingSnapshot } from "../services/pricing";

interface Props {
  settings: AppSettings;
  pricing: PricingSnapshot;
  pricingLoading: boolean;
  onRefreshPricing: () => void;
  onSaved: (settings: AppSettings) => void;
}

export function Settings({ settings, pricing, pricingLoading, onRefreshPricing, onSaved }: Props) {
  const { t, i18n } = useTranslation();
  const [form, setForm] = useState(settings);
  const [status, setStatus] = useState("");

  useEffect(() => setForm(settings), [settings]);

  const update = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) =>
    setForm((current) => ({ ...current, [key]: value }));
  const updateNumber = (key: "dailyBudget" | "monthlyBudget" | "fiveHourLimit" | "weeklyLimit", value: string) =>
    update(key, Math.max(0, Number(value) || 0));

  const choosePath = async () => {
    const selected = await open({ directory: true, multiple: false, title: t("settings.choosePathTitle") });
    if (selected) update("codexPath", selected);
  };
  const detect = async () => {
    try {
      update("codexPath", await api.detectCodexPath());
    } catch (cause) {
      setStatus(translateError(cause, t, "errors.codexHome"));
    }
  };
  const persist = async (next = form, message = true) => {
    try {
      const saved = await api.saveSettings(next);
      onSaved(saved);
      if (message) setStatus(t("settings.saved"));
      return saved;
    } catch (cause) {
      setStatus(translateError(cause, t, "errors.invalidSetting"));
      return null;
    }
  };
  const changeLanguage = async (language: Language) => {
    const next = { ...form, language };
    setForm(next);
    await i18n.changeLanguage(language);
    await persist(next, false);
  };
  const updateFloating = async <K extends "floatingOpacity" | "floatingAlwaysOnTop" | "floatingClickThrough" | "floatingMode">(key: K, value: AppSettings[K]) => {
    const next = { ...form, [key]: value };
    setForm(next);
    await persist(next, false);
  };
  const exportData = async () => {
    const path = await save({ defaultPath: "codex-token-usage.csv", filters: [{ name: "CSV", extensions: ["csv"] }] });
    if (!path) return;
    try {
      setStatus(t("settings.exported", { count: await api.exportCsv(path) }));
    } catch (cause) {
      setStatus(translateError(cause, t));
    }
  };
  const clear = async () => {
    if (!confirm(t("settings.clearConfirm"))) return;
    try {
      await api.clearData();
      setStatus(t("settings.cleared"));
    } catch (cause) {
      setStatus(translateError(cause, t));
    }
  };

  return <main className="page settings-page">
    <header className="page-header"><div><h1>{t("settings.title")}</h1><p>{t("settings.subtitle")}</p></div></header>

    <section className="settings-section">
      <h2>{t("settings.dataSource")}</h2>
      <label>{t("settings.codexPath")}</label>
      <div className="input-action">
        <input value={form.codexPath} onChange={(event) => update("codexPath", event.target.value)} />
        <button className="icon-button" onClick={() => void choosePath()} title={t("settings.chooseDirectory")} aria-label={t("settings.chooseDirectory")}><FolderOpen size={18} /></button>
        <button className="icon-button" onClick={() => void detect()} title={t("settings.autoDetect")} aria-label={t("settings.autoDetect")}><RotateCcw size={18} /></button>
      </div>
      <small>{t("settings.pathDescription")}</small>
      <label>{t("settings.refreshInterval")}</label>
      <div className="segments settings-segments">{([1, 5, 10, 30] as const).map((seconds) =>
        <button key={seconds} className={form.refreshIntervalSeconds === seconds ? "active" : ""} onClick={() => update("refreshIntervalSeconds", seconds)}>{seconds}s</button>
      )}</div>
    </section>

    <section className="settings-section">
      <h2>{t("settings.pricing")}</h2>
      <div className="pricing-settings-row"><div><strong>{pricing.source === "online" ? t("pricing.online") : pricing.source === "local" ? t("pricing.localFallback") : t("pricing.unavailable")}</strong><small>{t("pricing.lastUpdated")}: {pricing.updatedAt ? new Date(pricing.updatedAt).toLocaleString(i18n.resolvedLanguage ?? i18n.language) : t("pricing.never")}</small></div><button className="icon-button" onClick={onRefreshPricing} disabled={pricingLoading} title={t("pricing.refreshNow")} aria-label={t("pricing.refreshNow")}><RefreshCw size={18} className={pricingLoading ? "spin" : ""} /></button></div>
      <small>{pricing.error ? t("pricing.refreshFailed") : t("settings.pricingDescription")}</small>
    </section>

    <section className="settings-section">
      <h2>{t("settings.quotaSettings")}</h2>
      <div className="two-fields">
        <div><label>{t("settings.fiveHourLimit")}</label><input type="number" min="0" step="1000" value={form.fiveHourLimit} onChange={(event) => updateNumber("fiveHourLimit", event.target.value)} /></div>
        <div><label>{t("settings.weeklyLimit")}</label><input type="number" min="0" step="1000" value={form.weeklyLimit} onChange={(event) => updateNumber("weeklyLimit", event.target.value)} /></div>
      </div>
      <small>{t("settings.quotaDescription")}</small>
    </section>

    <section className="settings-section">
      <h2>{t("settings.appearanceBudget")}</h2>
      <label>{t("settings.language")}</label>
      <select value={form.language} onChange={(event) => void changeLanguage(event.target.value as Language)}>
        <option value="zh-CN">{t("settings.languageChinese")}</option>
        <option value="en-US">{t("settings.languageEnglish")}</option>
      </select>
      <label>{t("settings.theme")}</label>
      <select value={form.theme} onChange={(event) => update("theme", event.target.value as AppSettings["theme"])}>
        <option value="system">{t("settings.themeSystem")}</option><option value="light">{t("settings.themeLight")}</option><option value="dark">{t("settings.themeDark")}</option>
      </select>
      <div className="two-fields">
        <div><label>{t("settings.dailyBudget")}</label><input type="number" min="0" value={form.dailyBudget} onChange={(event) => updateNumber("dailyBudget", event.target.value)} /></div>
        <div><label>{t("settings.monthlyBudget")}</label><input type="number" min="0" value={form.monthlyBudget} onChange={(event) => updateNumber("monthlyBudget", event.target.value)} /></div>
      </div>
      <label>{t("settings.alertThresholds")}</label>
      <div className="checks">{([80, 90, 95] as const).map((threshold) =>
        <label key={threshold}><input type="checkbox" checked={form[`alert${threshold}`]} onChange={(event) => update(`alert${threshold}`, event.target.checked)} />{threshold}%</label>
      )}</div>
    </section>

    <section className="settings-section">
      <h2>{t("settings.localDatabase")}</h2>
      <div className="data-actions">
        <button onClick={() => void exportData()}><Database size={17} />{t("settings.exportCsv")}</button>
        <button className="danger" onClick={() => void clear()}><Trash2 size={17} />{t("settings.clearStatistics")}</button>
      </div>
    </section>

    <section className="settings-section">
      <h2>{t("settings.floatingWindow")}</h2>
      <label>{t("settings.floatingMode")}</label>
      <select value={form.floatingMode} onChange={(event) => void updateFloating("floatingMode", event.target.value as FloatingMode)}>
        <option value="auto">{t("settings.autoCollapseMode")}</option>
        <option value="compact">{t("settings.compactMode")}</option>
        <option value="detailed">{t("settings.detailedMode")}</option>
        <option value="orb">{t("settings.orbMode")}</option>
      </select>
      <label htmlFor="floating-opacity">{t("settings.opacity")} {Math.round(form.floatingOpacity * 100)}%</label>
      <input id="floating-opacity" type="range" min="20" max="100" step="10" value={Math.round(form.floatingOpacity * 100)} onChange={(event) => void updateFloating("floatingOpacity", Number(event.target.value) / 100)} />
      <div className="checks floating-options">
        <label><input type="checkbox" checked={form.floatingAlwaysOnTop} onChange={(event) => void updateFloating("floatingAlwaysOnTop", event.target.checked)} />{t("settings.alwaysOnTop")}</label>
        <label><input type="checkbox" checked={form.floatingClickThrough} onChange={(event) => void updateFloating("floatingClickThrough", event.target.checked)} />{t("settings.clickThrough")}</label>
      </div>
      <small>{t("settings.clickThroughDescription")}</small>
    </section>

    <footer className="settings-footer"><span>{status}</span><button className="primary" onClick={() => void persist()}><Save size={17} />{t("settings.save")}</button></footer>
  </main>;
}
