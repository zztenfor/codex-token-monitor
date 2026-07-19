import { describe, expect, it } from "vitest";
import { LocalPricingProvider } from "./local_provider";
import { parseOfficialPricingHtml } from "./remote_provider";
import { PricingService } from "./pricing_service";
import type { ModelPricing, PricingCache, PricingProvider } from "./pricing_provider";
import type { PricingCacheStore } from "./pricing_cache";

const onlineModel: ModelPricing = {
  model: "gpt-5.6",
  modelAlias: "gpt-5.6-sol",
  inputPerMillion: 5,
  cachedInputPerMillion: 0.5,
  outputPerMillion: 30,
  source: "online",
  updatedAt: "2026-07-17T00:00:00.000Z",
};

function provider(models: ModelPricing[]): PricingProvider {
  const cache: PricingCache = { updated_at: "2026-07-17T00:00:00.000Z", version: "2026-07-17", source: "online", models };
  return { name: "test", getModels: async () => models, getPricing: async (model) => models.find((item) => item.model === model) ?? null, syncPricing: async () => cache };
}

function store(cache: PricingCache | null): PricingCacheStore {
  return { load: async () => cache, save: async (next) => { cache = next; } };
}

describe("online pricing provider", () => {
  it("parses official model IDs, aliases, and token prices", () => {
    const html = `<table><tr><th>Model</th><th>Input</th><th>Cached input</th><th>Cache writes</th><th>Output</th></tr><tr><td>gpt-5.6-sol</td><td>$5.00</td><td>$0.50</td><td>$6.25</td><td>$30.00</td></tr><tr><td>gpt-5.7-mini</td><td>$1.00</td><td>$0.10</td><td>-</td><td>$6.00</td></tr></table>`;
    const models = parseOfficialPricingHtml(html, new Date("2026-07-17T00:00:00.000Z"));
    expect(models[0]).toEqual(onlineModel);
    expect(models[1]?.model).toBe("gpt-5.7-mini");
  });

  it("skips models when a required price is missing", () => {
    const html = `<p>gpt-5.6-sol $5.00 $0.50</p>`;
    expect(parseOfficialPricingHtml(html)).toEqual([]);
  });
});

describe("local pricing provider", () => {
  it("includes the official GPT-5.5 price and Codex alias", async () => {
    const pricing = await new LocalPricingProvider().getPricing("gpt-5.5-codex");
    expect(pricing?.model).toBe("gpt-5.5");
    expect(pricing?.inputPerMillion).toBe(5);
    expect(pricing?.cachedInputPerMillion).toBe(0.5);
    expect(pricing?.outputPerMillion).toBe(30);
  });
});

describe("pricing service", () => {
  it("keeps local pricing when the online provider fails", async () => {
    const failing: PricingProvider = { name: "offline", getModels: async () => { throw new Error("offline"); }, getPricing: async () => { throw new Error("offline"); }, syncPricing: async () => { throw new Error("offline"); } };
    const service = new PricingService(failing, provider([{ ...onlineModel, source: "local" }]), store(null));
    await service.initialize();
    const snapshot = await service.syncPricing();
    expect(snapshot.models).toHaveLength(1);
    expect(snapshot.source).toBe("local");
    expect(snapshot.error).toContain("offline");
  });

  it("uses a fresh cache without a network sync", async () => {
    const cached: PricingCache = { updated_at: new Date().toISOString(), version: "today", source: "online", models: [onlineModel] };
    const service = new PricingService(provider([]), provider([]), store(cached));
    const snapshot = await service.initialize();
    expect(snapshot.source).toBe("online");
    expect(service.needsSync()).toBe(false);
  });

  it("adds local fallback models that are absent from the online cache", async () => {
    const fallback = { ...onlineModel, model: "gpt-5.4", modelAlias: "gpt-5.4", source: "local" as const };
    const cached: PricingCache = { updated_at: new Date().toISOString(), version: "today", source: "online", models: [onlineModel] };
    const service = new PricingService(provider([]), provider([fallback]), store(cached));
    const snapshot = await service.initialize();
    expect(snapshot.models.map((model) => model.model)).toEqual(["gpt-5.4", "gpt-5.6"]);
    expect(service.getPricing("gpt-5.4")?.source).toBe("local");
  });

  it("calculates reasoning output at the output rate and aliases Codex models", async () => {
    const service = new PricingService(provider([onlineModel]), provider([]), store(null));
    await service.syncPricing();
    const cost = service.calculateCost("gpt-5.6-codex", { inputTokens: 1_000_000, cachedInputTokens: 100_000, outputTokens: 200_000, reasoningOutputTokens: 300_000, totalTokens: 1_600_000 });
    expect(cost?.inputCost).toBe(5);
    expect(cost?.cachedInputCost).toBe(0.05);
    expect(cost?.outputCost).toBe(15);
    expect(cost?.totalCost).toBe(20.05);
  });

  it("returns unavailable instead of estimating an unknown cached price", async () => {
    const model = { ...onlineModel, cachedInputPerMillion: null };
    const service = new PricingService(provider([model]), provider([]), store(null));
    await service.syncPricing();
    expect(service.calculateCost("gpt-5.6", { inputTokens: 1, cachedInputTokens: 1, outputTokens: 1, reasoningOutputTokens: 1, totalTokens: 4 })).toBeNull();
    expect(service.calculateCost("gpt-unknown", { inputTokens: 1, cachedInputTokens: 0, outputTokens: 1, reasoningOutputTokens: 0, totalTokens: 2 })).toBeNull();
  });
});
