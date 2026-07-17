import { Clock3, RefreshCw, ShieldCheck } from "lucide-react";
import { useTranslation } from "react-i18next";
import { formatDuration } from "../lib/format";
import type { RealAccountQuota, RealQuotaWindow } from "../lib/types";

interface Props {
  quota: RealAccountQuota | null;
  loading: boolean;
  onRefresh: () => void;
}

function WindowUsage({ title, window }: { title: string; window: RealQuotaWindow | null }) {
  const { t } = useTranslation();
  if (!window) {
    return <div className="real-quota-window unavailable">
      <div><strong>{title}</strong><span>{t("quota.notReturned")}</span></div>
      <span className="quota-unavailable">{t("quota.unavailable")}</span>
    </div>;
  }
  return <div className="real-quota-window">
    <div className="real-quota-heading"><strong>{title}</strong><span><Clock3 size={13} />{t("quota.resetIn", { duration: formatDuration(window.resetAfterSeconds, t) })}</span></div>
    <div className="real-quota-numbers">
      <div><span>{t("quota.usedLabel")}</span><strong>{window.usedPercent.toFixed(1)}%</strong></div>
      <div><span>{t("quota.remaining")}</span><strong>{window.remainingPercent.toFixed(1)}%</strong></div>
    </div>
    <div className="real-quota-progress"><i style={{ width: `${Math.min(window.usedPercent, 100)}%` }} /></div>
  </div>;
}

export function RealAccountQuotaCard({ quota, loading, onRefresh }: Props) {
  const { t, i18n } = useTranslation();
  const language = i18n.resolvedLanguage ?? i18n.language;
  const available = quota?.status === "ok";
  const messageKey = quota?.status && ["AUTH_NOT_FOUND", "AUTH_INVALID", "AUTH_EXPIRED", "CONNECTION_FAILED", "QUOTA_UNAVAILABLE", "NOT_SYNCED"].includes(quota.status)
    ? quota.status
    : "unknown";
  return <section className="panel real-account-panel">
    <div className="panel-title">
      <div><h2>{t("quota.realTitle")}</h2><p>{t("quota.realSubtitle")}</p></div>
      <button className="icon-button" onClick={onRefresh} disabled={loading} title={t("quota.refresh")} aria-label={t("quota.refresh")}><RefreshCw size={17} className={loading ? "spin" : ""} /></button>
    </div>
    {!available && <div className="real-quota-status"><ShieldCheck size={16} /><span>{t(`quota.messages.${messageKey}`)}</span></div>}
    {available && <div className="real-quota-grid">
      <WindowUsage title={t("settings.fiveHourLimit")} window={quota.fiveHour} />
      <WindowUsage title={t("settings.weeklyLimit")} window={quota.weekly} />
    </div>}
    <div className="real-quota-footer">
      <span>{available ? t("quota.officialData") : t("quota.unavailable")}</span>
      <span>{quota?.createdAt ? t("quota.updated", { time: new Date(quota.createdAt).toLocaleString(language) }) : t("quota.notSyncedYet")}</span>
    </div>
  </section>;
}
