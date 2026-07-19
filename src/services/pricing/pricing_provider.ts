export type PricingSource = "online" | "local";

export interface ModelPricing {
  model: string;
  modelAlias: string;
  inputPerMillion: number;
  cachedInputPerMillion: number | null;
  outputPerMillion: number;
  source: PricingSource;
  updatedAt: string;
}

export interface PricingCache {
  updated_at: string;
  version: string;
  source: PricingSource;
  models: ModelPricing[];
}

export interface PricingProvider {
  readonly name: string;
  getModels(): Promise<ModelPricing[]>;
  getPricing(model: string): Promise<ModelPricing | null>;
  syncPricing(): Promise<PricingCache>;
}

export const MODEL_ALIASES: Record<string, string> = {
  "gpt-5.6-codex": "gpt-5.6",
  "gpt-5.6-sol": "gpt-5.6",
  "gpt-5.5-codex": "gpt-5.5",
  "gpt-5-codex": "gpt-5",
  "gpt-5.1-codex": "gpt-5.1",
};

export function canonicalModel(model: string): string {
  const normalized = model.trim().toLowerCase();
  return MODEL_ALIASES[normalized] ?? normalized;
}

export function createPricingCache(models: ModelPricing[], source: PricingSource, now = new Date()): PricingCache {
  return {
    updated_at: now.toISOString(),
    version: now.toISOString().slice(0, 10),
    source,
    models,
  };
}
