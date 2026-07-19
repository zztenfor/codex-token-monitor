import { canonicalModel, createPricingCache, type ModelPricing, type PricingProvider } from "./pricing_provider";

const FALLBACK_UPDATED_AT = "2026-07-17T00:00:00.000Z";

const FALLBACK_MODELS: ModelPricing[] = [
  { model: "gpt-5.6", modelAlias: "gpt-5.6-sol", inputPerMillion: 5, cachedInputPerMillion: 0.5, outputPerMillion: 30, source: "local", updatedAt: FALLBACK_UPDATED_AT },
  { model: "gpt-5.6-terra", modelAlias: "gpt-5.6-terra", inputPerMillion: 2.5, cachedInputPerMillion: 0.25, outputPerMillion: 15, source: "local", updatedAt: FALLBACK_UPDATED_AT },
  { model: "gpt-5.6-luna", modelAlias: "gpt-5.6-luna", inputPerMillion: 1, cachedInputPerMillion: 0.1, outputPerMillion: 6, source: "local", updatedAt: FALLBACK_UPDATED_AT },
  { model: "gpt-5.4", modelAlias: "gpt-5.4", inputPerMillion: 2.5, cachedInputPerMillion: 0.25, outputPerMillion: 15, source: "local", updatedAt: FALLBACK_UPDATED_AT },
  { model: "gpt-5.2", modelAlias: "gpt-5.2", inputPerMillion: 1.75, cachedInputPerMillion: 0.175, outputPerMillion: 14, source: "local", updatedAt: FALLBACK_UPDATED_AT },
  { model: "gpt-5.1", modelAlias: "gpt-5.1", inputPerMillion: 1.25, cachedInputPerMillion: 0.125, outputPerMillion: 10, source: "local", updatedAt: FALLBACK_UPDATED_AT },
  { model: "gpt-5", modelAlias: "gpt-5", inputPerMillion: 1.25, cachedInputPerMillion: 0.125, outputPerMillion: 10, source: "local", updatedAt: FALLBACK_UPDATED_AT },
  { model: "gpt-4.1", modelAlias: "gpt-4.1", inputPerMillion: 2, cachedInputPerMillion: 0.5, outputPerMillion: 8, source: "local", updatedAt: FALLBACK_UPDATED_AT },
];

export class LocalPricingProvider implements PricingProvider {
  readonly name = "local";

  async getModels(): Promise<ModelPricing[]> {
    return FALLBACK_MODELS.map((model) => ({ ...model }));
  }

  async getPricing(model: string): Promise<ModelPricing | null> {
    const canonical = canonicalModel(model);
    return (await this.getModels()).find((item) => item.model === canonical) ?? null;
  }

  async syncPricing() {
    return createPricingCache(await this.getModels(), "local", new Date(FALLBACK_UPDATED_AT));
  }
}
