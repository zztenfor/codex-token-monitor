import { CircleDollarSign, Database, RefreshCw, ShieldCheck, TriangleAlert } from "lucide-react";
import { useTranslation } from "react-i18next";
import { formatTokens } from "../lib/format";
import type { UsageRange } from "../lib/preferences";
import { canonicalModel, type ModelPricing } from "../services/pricing/pricing_provider";
import { calculateCost, type PricingSnapshot } from "../services/pricing/pricing_service";
import type { DashboardData, ModelUsage, TokenTotals } from "../lib/types";

interface Props {
  data: DashboardData | null;
  pricing: PricingSnapshot;
  range: UsageRange;
  onRange: (range: UsageRange) => void;
  onRefresh: () => void;
  loading: boolean;
}

function usd(value: number): string {
  return new Intl.NumberFormat(undefined, { style: "currency", currency: "USD", minimumFractionDigits: 4, maximumFractionDigits: 4 }).format(value);
}

function modelFor(pricing: PricingSnapshot, model: string): ModelPricing | null {
  const wanted = canonicalModel(model);
  return pricing.models.find((item) => item.model === wanted) ?? null;
}

function priceStatus(pricing: PricingSnapshot, model: string): "matched" | "pending" | "unknown" {
  if (!model || canonicalModel(model) === "unknown") return "unknown";
  return modelFor(pricing, model) ? "matched" : "pending";
}

export interface AggregateCost {
  totalTokens: number;
  totalCost: number;
  pricedModels: number;
  unavailableModels: number;
  rows: Array<{ usage: ModelUsage; cost: ReturnType<typeof calculateCost> }>;
}

export function aggregateCost(models: ModelUsage[], pricing: PricingSnapshot): AggregateCost {
  let totalTokens = 0;
  let totalCost = 0;
  let pricedModels = 0;
  let unavailableModels = 0;
  const rows = models.map((usage) => {
    totalTokens += usage.totalTokens;
    const cost = calculateCost(modelFor(pricing, usage.model), usage);
    if (cost) {
      totalCost += cost.totalCost;
      pricedModels += 1;
    } else {
      unavailableModels += 1;
    }
    return { usage, cost };
  });
  return { totalTokens, totalCost, pricedModels, unavailableModels, rows };
}

export function Cost({ data, pricing, range, onRange, onRefresh, loading }: Props) {
  const { t, i18n } = useTranslation();
  const language = i18n.resolvedLanguage ?? i18n.language;
  const session = data?.currentSession;
  const totals: TokenTotals = session ?? { inputTokens: 0, cachedInputTokens: 0, outputTokens: 0, reasoningOutputTokens: 0, totalTokens: 0 };
  const selectedModel = session?.model ?? "";
  const selectedPricing = selectedModel ? modelFor(pricing, selectedModel) : null;
  const cost = calculateCost(selectedPricing, totals);
  const cumulative = aggregateCost(data?.modelUsage ?? [], pricing);
  const sourceLabel = pricing.source === "online" ? t("pricing.online") : pricing.source === "local" ? t("pricing.localFallback") : t("pricing.unavailable");
  const rangeLabel = range === 1 ? t("dashboard.today") : t("dashboard.days", { count: range });

  return <main className="page cost-page">
    <header className="page-header"><div><h1>{t("cost.title")}</h1><p>{t("cost.subtitle")}</p></div><button className="icon-button sync-button" onClick={onRefresh} disabled={loading} title={t("pricing.refreshNow")} aria-label={t("pricing.refreshNow")}><RefreshCw size={18} className={loading ? "spin" : ""} /></button></header>

    <section className="cost-overview">
      <div className="panel cost-summary"><div className="panel-title"><div><h2>{t("cost.cumulativeTitle")}</h2><p>{t("cost.cumulativeSubtitle")}</p></div><CircleDollarSign size={21} /></div><div className="segments cost-range-segments">{([1, 7, 30] as const).map((days) => <button key={days} className={range === days ? "active" : ""} onClick={() => onRange(days)}>{days === 1 ? t("dashboard.today") : t("dashboard.days", { count: days })}</button>)}</div><strong className="cost-amount">{cumulative.pricedModels > 0 || cumulative.rows.length === 0 ? usd(cumulative.totalCost) : t("pricing.unavailable")}</strong><span className="cost-caption">{t("cost.cumulativeEstimate", { range: rangeLabel })}</span>{cumulative.unavailableModels > 0 && <div className="cost-partial-warning"><TriangleAlert size={14} /><span>{t("cost.partialPricingWarning", { count: cumulative.unavailableModels })}</span></div>}<div className="cost-cumulative-meta"><span>{formatTokens(cumulative.totalTokens, language)} {t("cost.cumulativeTokens")}</span><span>{t("cost.modelsCount", { count: cumulative.rows.length })}</span></div><div className="cost-meta"><span>{t("pricing.source")}: {sourceLabel}</span><span>{t("pricing.lastUpdated")}: {pricing.updatedAt ? new Date(pricing.updatedAt).toLocaleString(language) : t("pricing.never")}</span></div></div>
      <div className="panel cost-session-summary"><div className="panel-title"><div><h2>{t("cost.currentModel")}</h2><p>{selectedModel || t("cost.noSession")}</p></div><Database size={19} /></div><strong className="cost-amount">{cost ? usd(cost.totalCost) : priceStatus(pricing, selectedModel) === "pending" ? "价格待确认" : priceStatus(pricing, selectedModel) === "unknown" ? "未知模型" : t("pricing.unavailable")}</strong><span className="cost-caption">{t("cost.currentSessionEstimate")}</span><div className="cost-meta"><span>{formatTokens(totals.totalTokens, language)} {t("dashboard.totalTokens")}</span><span>{selectedModel || t("cost.noSession")}</span></div></div>
    </section>

    <section className="cost-breakdown-grid">
      <div className="panel cost-breakdown"><div className="panel-title"><div><h2>{t("cost.breakdown")}</h2><p>{t("cost.reasoningIncluded")}</p></div><Database size={19} /></div>{cost ? <><div><span>{t("dashboard.inputTokens")}</span><strong>{usd(cost.inputCost)}</strong></div><div><span>{t("dashboard.cachedInputTokens")}</span><strong>{usd(cost.cachedInputCost)}</strong></div><div><span>{t("dashboard.outputTokens")}</span><strong>{usd(cost.outputCost)}</strong></div><div className="cost-total"><span>{t("dashboard.totalTokens")}</span><strong>{usd(cost.totalCost)}</strong></div></> : <div className="empty-state">{priceStatus(pricing, selectedModel) === "pending" ? "价格待确认" : priceStatus(pricing, selectedModel) === "unknown" ? "未知模型" : t("pricing.unavailableDescription")}</div>}</div>
      <div className="panel cost-breakdown"><div className="panel-title"><div><h2>{t("cost.cumulativeBreakdown")}</h2><p>{t("cost.cumulativeBreakdownSubtitle")}</p></div><Database size={19} /></div>{cumulative.rows.length ? cumulative.rows.map(({ usage, cost: modelCost }) => <div className="cost-model-row" key={usage.model}><span><strong>{usage.model || t("cost.unknownModel")}</strong><small>{formatTokens(usage.totalTokens, language)} {t("dashboard.totalTokens")}</small></span><strong>{modelCost ? usd(modelCost.totalCost) : t("pricing.unavailable")}</strong></div>) : <div className="empty-state">{t("cost.noCumulativeUsage")}</div>}</div>
    </section>

    <section className="panel pricing-panel"><div className="panel-title"><div><h2>{t("cost.modelPricing")}</h2><p>{t("cost.modelPricingSubtitle")}</p></div><ShieldCheck size={19} /></div><div className="pricing-table"><div className="pricing-row pricing-header"><span>{t("cost.model")}</span><span>{t("cost.inputPrice")}</span><span>{t("cost.cachedPrice")}</span><span>{t("cost.outputPrice")}</span></div>{pricing.models.length ? pricing.models.map((model) => <div className="pricing-row" key={model.model}><strong>{model.model}</strong><span>{usd(model.inputPerMillion)} / MTok</span><span>{model.cachedInputPerMillion == null ? t("pricing.unavailable") : `${usd(model.cachedInputPerMillion)} / MTok`}</span><span>{usd(model.outputPerMillion)} / MTok</span></div>) : <div className="empty-state">{t("pricing.unavailableDescription")}</div>}</div></section>

    <div className="local-badge cost-notice"><ShieldCheck size={17} /><div><strong>{t("cost.localNotice")}</strong><span>{t("cost.localNoticeDescription")}</span></div></div>
  </main>;
}
