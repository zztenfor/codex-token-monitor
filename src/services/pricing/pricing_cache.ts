import { api } from "../../lib/tauri";
import type { PricingCache } from "./pricing_provider";

export interface PricingCacheStore {
  load(): Promise<PricingCache | null>;
  save(cache: PricingCache): Promise<void>;
}

function isPricingCache(value: unknown): value is PricingCache {
  if (!value || typeof value !== "object") return false;
  const candidate = value as Partial<PricingCache>;
  return typeof candidate.updated_at === "string"
    && typeof candidate.version === "string"
    && (candidate.source === "online" || candidate.source === "local")
    && Array.isArray(candidate.models);
}

export const tauriPricingCache: PricingCacheStore = {
  async load() {
    const raw = await api.readPricingCache();
    if (!raw) return null;
    try {
      const parsed: unknown = JSON.parse(raw);
      return isPricingCache(parsed) ? parsed : null;
    } catch {
      return null;
    }
  },
  save(cache) {
    return api.savePricingCache(JSON.stringify(cache));
  },
};

export function isCacheFresh(cache: PricingCache, now = Date.now()): boolean {
  const updated = Date.parse(cache.updated_at);
  return Number.isFinite(updated) && now - updated < 24 * 60 * 60 * 1000;
}
