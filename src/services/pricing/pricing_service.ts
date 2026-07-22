import { LocalPricingProvider } from "./local_provider";
import { isCacheFresh, tauriPricingCache, type PricingCacheStore } from "./pricing_cache";
import { RemotePricingProvider } from "./remote_provider";
import { canonicalModel, type ModelPricing, type PricingCache, type PricingProvider } from "./pricing_provider";
import type { TokenTotals } from "../../lib/types";

export interface CalculatedCost {
  model: string;
  inputCost: number;
  cachedInputCost: number;
  outputCost: number;
  totalCost: number;
  currency: "USD";
}

export interface PricingSnapshot {
  models: ModelPricing[];
  updatedAt: string | null;
  source: "online" | "local" | "unavailable";
  error: string;
}

export const emptyPricingSnapshot: PricingSnapshot = {
  models: [],
  updatedAt: null,
  source: "unavailable",
  error: "",
};

export function calculateCost(pricing: ModelPricing | null, totals: TokenTotals): CalculatedCost | null {
  if (!pricing || pricing.cachedInputPerMillion == null) return null;
  const inputCost = totals.inputTokens * pricing.inputPerMillion / 1_000_000;
  const cachedInputCost = totals.cachedInputTokens * pricing.cachedInputPerMillion / 1_000_000;
  const outputCost = (totals.outputTokens + totals.reasoningOutputTokens) * pricing.outputPerMillion / 1_000_000;
  return { model: pricing.model, inputCost, cachedInputCost, outputCost, totalCost: inputCost + cachedInputCost + outputCost, currency: "USD" };
}

export class PricingService {
  private cache: PricingCache | null = null;
  private snapshot: PricingSnapshot = emptyPricingSnapshot;
  private shouldSync = true;

  constructor(
    private readonly remote: PricingProvider = new RemotePricingProvider(),
    private readonly local: PricingProvider = new LocalPricingProvider(),
    private readonly store: PricingCacheStore = tauriPricingCache,
  ) {}

  getSnapshot(): PricingSnapshot {
    return { ...this.snapshot, models: [...this.snapshot.models] };
  }

  async initialize(): Promise<PricingSnapshot> {
    const cached = await this.store.load();
    if (cached) {
      this.cache = cached;
      await this.applyCache(cached);
    } else {
      await this.applyLocalFallback();
    }
    this.shouldSync = !cached || !isCacheFresh(cached);
    return this.getSnapshot();
  }

  needsSync(): boolean {
    return this.shouldSync;
  }

  async syncPricing(): Promise<PricingSnapshot> {
    try {
      const cache = await this.remote.syncPricing();
      await this.store.save(cache);
      this.cache = cache;
      await this.applyCache(cache);
      this.shouldSync = false;
    } catch (cause) {
      if (!this.snapshot.models.length || this.snapshot.source === "unavailable") await this.applyLocalFallback();
      this.snapshot = { ...this.snapshot, error: cause instanceof Error ? cause.message : "Pricing unavailable" };
      this.shouldSync = false;
    }
    return this.getSnapshot();
  }

  getPricing(model: string): ModelPricing | null {
    const wanted = canonicalModel(model);
    return this.snapshot.models.find((item) => item.model === wanted) ?? null;
  }

  calculateCost(model: string, totals: TokenTotals): CalculatedCost | null {
    return calculateCost(this.getPricing(model), totals);
  }

  private async applyCache(cache: PricingCache) {
    // Keep the online result authoritative, but retain bundled entries that
    // the public page failed to expose during a transient/partial update.
    // This prevents newly shipped models (for example GPT-5.6) disappearing
    // from the calculator while still preferring online prices whenever they
    // exist.
    const models = new Map((await this.local.getModels()).map((model) => [model.model, model]));
    for (const model of cache.models) models.set(model.model, { ...model });
    this.snapshot = {
      models: [...models.values()],
      updatedAt: cache.updated_at,
      source: cache.source,
      error: "",
    };
  }

  private async applyLocalFallback() {
    const models = await this.local.getModels();
    this.snapshot = { models, updatedAt: models[0]?.updatedAt ?? null, source: "local", error: "" };
  }
}
