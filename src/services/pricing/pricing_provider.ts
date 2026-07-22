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

// Keep user-visible model matching data in a small editable file. The built-in
// aliases remain as a safe fallback for older installations.
import priceAliases from "../../../model_price_alias.json";

export function canonicalModel(model: string): string {
  const normalized = model.trim().toLowerCase();
  const configured = (priceAliases as Record<string, { canonical: string | null }>)[normalized];
  if (configured) return configured.canonical ?? "unknown";
  if (MODEL_ALIASES[normalized]) return MODEL_ALIASES[normalized];
  if (/^gpt-\d+(?:\.\d+)?-(?:sol|codex|pro|nano)$/.test(normalized)) return normalized.replace(/-(?:sol|codex|pro|nano)$/, "");
  if (normalized === "codex-auto-review" || normalized === "codex-next") return "codex";
  return normalized || "unknown";
}

export function createPricingCache(models: ModelPricing[], source: PricingSource, now = new Date()): PricingCache {
  return {
    updated_at: now.toISOString(),
    version: now.toISOString().slice(0, 10),
    source,
    models,
  };
}
