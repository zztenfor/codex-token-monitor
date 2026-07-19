import { Activity, ArrowDownToLine, ArrowUpFromLine, BrainCircuit, Clock3, Database, FolderKanban, Gauge, RefreshCw } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Area, AreaChart, CartesianGrid, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { Metric } from "../components/Metric";
import { RealAccountQuotaCard } from "../components/RealAccountQuotaCard";
import { formatDuration, formatTokens, shortSession } from "../lib/format";
import type { UsageRange } from "../lib/preferences";
import type { DashboardData, QuotaWindow } from "../lib/types";

interface Props {
  data: DashboardData | null;
  range: UsageRange;
  loading: boolean;
  onRange: (days: UsageRange) => void;
  onSync: () => void;
  quotaLoading: boolean;
  onQuotaSync: () => void;
}

function QuotaRow({ kind, quota }: { kind: "fiveHour" | "weekly"; quota: QuotaWindow }) {
  const { t, i18n } = useTranslation();
  const language = i18n.resolvedLanguage ?? i18n.language;
  const percent = Math.min(quota.usagePercent ?? 0, 100);
  const configured = quota.limitTokens > 0;
  const releaseLabel = kind === "weekly" && quota.resetAt
    ? t("quota.firstRelease", { time: new Date(quota.resetAt).toLocaleString(language) })
    : t("quota.releasesIn", { duration: formatDuration(quota.resetInSeconds, t) });
  return <div className="quota-window">
    <div className="quota-heading"><div><strong>{t(`quota.${kind}`)}</strong><span>{configured ? t("quota.used", { value: percent.toFixed(1) }) : t("quota.setLimit")}</span></div><span className={`forecast ${quota.forecastStatus}`}>{t(`status.${quota.forecastStatus}`)}</span></div>
    <div className="quota-values"><div><span>{t("quota.consumed")}</span><strong>{formatTokens(quota.consumedTokens, language)}</strong></div><div><span>{t("quota.remaining")}</span><strong>{configured ? formatTokens(quota.remainingTokens, language) : "--"}</strong></div></div>
    <div className="quota-progress"><i style={{ width: `${percent}%` }} /></div>
    <div className="quota-meta"><span><Clock3 size={13} />{releaseLabel}</span><span>{t("quota.forecast", { value: formatTokens(quota.projectedTokens, language) })}</span></div>
  </div>;
}

export function Dashboard({ data, range, loading, onRange, onSync, quotaLoading, onQuotaSync }: Props) {
  const { t, i18n } = useTranslation();
  const language = i18n.resolvedLanguage ?? i18n.language;
  const tokens = (value: number) => formatTokens(value, language);
  const today = data?.today ?? { inputTokens: 0, cachedInputTokens: 0, outputTokens: 0, reasoningOutputTokens: 0, totalTokens: 0 };
  const session = data?.currentSession;
  const percent = session?.usagePercent ?? 0;
  const breakdown = [
    { label: t("dashboard.categoryInput"), value: today.inputTokens, tone: "input" },
    { label: t("dashboard.categoryCached"), value: today.cachedInputTokens, tone: "cached" },
    { label: t("dashboard.categoryOutput"), value: today.outputTokens, tone: "output" },
    { label: t("dashboard.categoryReasoning"), value: today.reasoningOutputTokens, tone: "reasoning" },
  ];
  return (
    <main className="page dashboard">
      <header className="page-header">
        <div><h1>{t("dashboard.title")}</h1><p>{t("dashboard.subtitle")}</p></div>
        <button className="icon-button sync-button" onClick={onSync} disabled={loading} title={t("dashboard.syncNow")} aria-label={t("dashboard.syncNow")}><RefreshCw size={18} className={loading ? "spin" : ""} /></button>
      </header>

      <section className="metrics-grid" aria-label={t("dashboard.todayUsage")}>
        <Metric label={t("dashboard.inputTokens")} value={today.inputTokens} icon={ArrowDownToLine} tone="blue" />
        <Metric label={t("dashboard.cachedInputTokens")} value={today.cachedInputTokens} icon={Database} tone="rose" />
        <Metric label={t("dashboard.outputTokens")} value={today.outputTokens} icon={ArrowUpFromLine} tone="green" />
        <Metric label={t("dashboard.reasoningTokens")} value={today.reasoningOutputTokens} icon={BrainCircuit} tone="amber" />
        <Metric label={t("dashboard.totalTokens")} value={today.totalTokens} icon={Activity} tone="rose" />
      </section>

      <RealAccountQuotaCard quota={data?.realAccountQuota ?? null} loading={quotaLoading} onRefresh={onQuotaSync} />

      <section className="breakdown-quota-grid">
        <div className="panel breakdown-panel">
          <div className="panel-title"><div><h2>{t("dashboard.breakdownTitle")}</h2><p>{t("dashboard.breakdownSubtitle")}</p></div><Gauge size={19} /></div>
          <div className="breakdown-total"><span>{t("dashboard.chartTotal")}</span><strong>{tokens(today.totalTokens)}</strong></div>
          <div className="breakdown-bar" aria-label={t("dashboard.categoryProportions")}>{breakdown.map((item) => <i key={item.tone} className={item.tone} style={{ width: `${today.totalTokens > 0 ? item.value * 100 / today.totalTokens : 0}%` }} />)}</div>
          <div className="breakdown-list">{breakdown.map((item) => <div key={item.tone}><span><i className={item.tone} />{item.label}</span><strong>{tokens(item.value)}</strong><em>{today.totalTokens > 0 ? `${(item.value * 100 / today.totalTokens).toFixed(1)}%` : "0.0%"}</em></div>)}</div>
        </div>
        <div className="panel quota-panel">
          <div className="panel-title"><div><h2>{t("quota.monitorTitle")}</h2><p>{t("quota.monitorSubtitle")}</p></div><Clock3 size={19} /></div>
          {data ? <><QuotaRow kind="fiveHour" quota={data.fiveHourQuota} /><QuotaRow kind="weekly" quota={data.weeklyQuota} /></> : <div className="empty-state">{t("quota.loading")}</div>}
        </div>
      </section>

      <section className="dashboard-grid">
        <div className="panel session-panel">
          <div className="panel-title"><div><h2>{t("session.title")}</h2><p>{t("session.subtitle")}</p></div><span className="live-dot">{t("session.live")}</span></div>
          {session ? <>
            <div className="session-main"><div><span>{t("session.model")}</span><strong>{session.model || t("session.unknown")}</strong></div><div><span>{t("session.session")}</span><strong title={session.sessionId}>{shortSession(session.sessionId)}</strong></div></div>
            <div className="usage-row"><span>{t("session.contextUsage")}</span><strong>{session.usagePercent == null ? t("common.notAvailable") : `${percent.toFixed(1)}%`}</strong></div>
            <div className="progress"><i style={{ width: `${Math.min(percent, 100)}%` }} /></div>
            <div className="usage-caption"><span>{t("session.inContext", { value: tokens(session.contextTokens) })}</span><span>{session.contextWindow ? t("session.remaining", { value: tokens(Math.max(0, session.contextWindow - session.contextTokens)) }) : t("session.limitUnavailable")}</span></div>
          </> : <div className="empty-state">{t("session.notFound")}</div>}
        </div>

        <div className="panel prediction-panel">
          <div className="panel-title"><div><h2>{t("dashboard.usagePrediction")}</h2><p>{t("dashboard.predictionSubtitle")}</p></div><Database size={19} /></div>
          <div className="prediction-stats">
            <div className="prediction-stat">
              <strong>{tokens(data?.monthlyTotalTokens ?? 0)}</strong>
              <span>{t("dashboard.monthlyActual")}</span>
            </div>
            <div className="prediction-stat prediction-stat-forecast">
              <strong>{tokens(data?.projectedMonthlyTokens ?? 0)}</strong>
              <span>{t("dashboard.projectedMonthly")}</span>
            </div>
          </div>
          <div className="prediction-footer"><span>{t("dashboard.dailyAverage", { value: tokens(data?.dailyAverageTokens ?? 0) })}</span><span className={`risk ${data?.riskLevel ?? "insufficient"}`}>{t(`status.${data?.riskLevel ?? "insufficient"}`)}</span></div>
        </div>
      </section>

      <section className="panel chart-panel">
        <div className="panel-title"><div><h2>{t("dashboard.usageHistory")}</h2><p>{t("dashboard.historySubtitle")}</p></div><div className="segments">{([1, 7, 30] as const).map((days) => <button key={days} className={range === days ? "active" : ""} onClick={() => onRange(days)}>{days === 1 ? t("dashboard.today") : t("dashboard.days", { count: days })}</button>)}</div></div>
        <div className="chart-wrap"><ResponsiveContainer width="100%" height="100%"><AreaChart data={data?.chart ?? []} margin={{ top: 10, right: 8, bottom: 0, left: -12 }}><defs><linearGradient id="usageFill" x1="0" y1="0" x2="0" y2="1"><stop offset="5%" stopColor="#1473e6" stopOpacity={0.28}/><stop offset="95%" stopColor="#1473e6" stopOpacity={0}/></linearGradient></defs><CartesianGrid strokeDasharray="3 3" vertical={false} /><XAxis dataKey="label" tickLine={false} axisLine={false} /><YAxis tickFormatter={tokens} tickLine={false} axisLine={false}/><Tooltip formatter={(value) => [tokens(Number(value)), t("dashboard.chartTotal")]}/><Area type="monotone" dataKey="totalTokens" stroke="#1473e6" strokeWidth={2} fill="url(#usageFill)" /></AreaChart></ResponsiveContainer></div>
      </section>

      <section className="panel projects-panel">
        <div className="panel-title"><div><h2>{t("dashboard.projects")}</h2><p>{t("dashboard.projectsSubtitle")}</p></div><FolderKanban size={19} /></div>
        <div className="project-list">{data?.projects.length ? data.projects.map((project, index) => <div className="project-row" key={project.projectKey}><span className="rank">{index + 1}</span><div><strong>{project.displayName}</strong><span>{t("dashboard.projectTokens", { value: tokens(project.totalTokens) })}</span></div><div className="mini-bar"><i style={{ width: `${(project.totalTokens / data.projects[0]!.totalTokens) * 100}%` }} /></div></div>) : <div className="empty-state">{t("dashboard.noProjects")}</div>}</div>
      </section>
    </main>
  );
}
