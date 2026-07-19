import { CircleDollarSign, Database, RefreshCw, ShieldCheck } from "lucide-react";
import { useTranslation } from "react-i18next";
import { formatTokens } from "../lib/format";
import { canonicalModel, type ModelPricing } from "../services/pricing/pricing_provider";
import { calculateCost, type PricingSnapshot } from "../services/pricing/pricing_service";
import type { DashboardData, TokenTotals } from "../lib/types";

interface Props {
  data: DashboardData | null;
  pricing: PricingSnapshot;
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

export function Cost({ data, pricing, onRefresh, loading }: Props) {
  const { t, i18n } = useTranslation();
  const language = i18n.resolvedLanguage ?? i18n.language;
  const session = data?.currentSession;
  const totals: TokenTotals = session ?? { inputTokens: 0, cachedInputTokens: 0, outputTokens: 0, reasoningOutputTokens: 0, totalTokens: 0 };
  const selectedModel = session?.model ?? "";
  const selectedPricing = selectedModel ? modelFor(pricing, selectedModel) : null;
  const cost = calculateCost(selectedPricing, totals);
  const sourceLabel = pricing.source === "online" ? t("pricing.online") : pricing.source === "local" ? t("pricing.localFallback") : t("pricing.unavailable");

  return <main className="page cost-page">
    <header className="page-header"><div><h1>{t("cost.title")}</h1><p>{t("cost.subtitle")}</p></div><button className="icon-button sync-button" onClick={onRefresh} disabled={loading} title={t("pricing.refreshNow")} aria-label={t("pricing.refreshNow")}><RefreshCw size={18} className={loading ? "spin" : ""} /></button></header>

    <section className="cost-overview">
      <div className="panel cost-summary"><div className="panel-title"><div><h2>{t("cost.currentModel")}</h2><p>{selectedModel || t("cost.noSession")}</p></div><CircleDollarSign size={21} /></div><strong className="cost-amount">{cost ? usd(cost.totalCost) : t("pricing.unavailable")}</strong><span className="cost-caption">{t("cost.currentSessionEstimate")}</span><div className="cost-meta"><span>{t("pricing.source")}: {sourceLabel}</span><span>{t("pricing.lastUpdated")}: {pricing.updatedAt ? new Date(pricing.updatedAt).toLocaleString(language) : t("pricing.never")}</span></div></div>
      <div className="panel cost-breakdown"><div className="panel-title"><div><h2>{t("cost.breakdown")}</h2><p>{t("cost.reasoningIncluded")}</p></div><Database size={19} /></div>{cost ? <><div><span>{t("dashboard.inputTokens")}</span><strong>{usd(cost.inputCost)}</strong></div><div><span>{t("dashboard.cachedInputTokens")}</span><strong>{usd(cost.cachedInputCost)}</strong></div><div><span>{t("dashboard.outputTokens")}</span><strong>{usd(cost.outputCost)}</strong></div><div className="cost-total"><span>{t("dashboard.totalTokens")}</span><strong>{usd(cost.totalCost)}</strong></div></> : <div className="empty-state">{t("pricing.unavailableDescription")}</div>}</div>
    </section>

    <section className="panel pricing-panel"><div className="panel-title"><div><h2>{t("cost.modelPricing")}</h2><p>{t("cost.modelPricingSubtitle")}</p></div><ShieldCheck size={19} /></div><div className="pricing-table"><div className="pricing-row pricing-header"><span>{t("cost.model")}</span><span>{t("cost.inputPrice")}</span><span>{t("cost.cachedPrice")}</span><span>{t("cost.outputPrice")}</span></div>{pricing.models.length ? pricing.models.map((model) => <div className="pricing-row" key={model.model}><strong>{model.model}</strong><span>{usd(model.inputPerMillion)} / MTok</span><span>{model.cachedInputPerMillion == null ? t("pricing.unavailable") : `${usd(model.cachedInputPerMillion)} / MTok`}</span><span>{usd(model.outputPerMillion)} / MTok</span></div>) : <div className="empty-state">{t("pricing.unavailableDescription")}</div>}</div></section>

    <div className="local-badge cost-notice"><ShieldCheck size={17} /><div><strong>{t("cost.localNotice")}</strong><span>{t("cost.localNoticeDescription")}</span></div></div>
  </main>;
}
