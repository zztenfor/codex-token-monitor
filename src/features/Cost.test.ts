import { describe, expect, it } from "vitest";
import type { ModelUsage } from "../lib/types";
import type { PricingSnapshot } from "../services/pricing/pricing_service";
import { aggregateCost } from "./Cost";

describe("cumulative cost", () => {
  it("keeps known model costs and flags unknown models", () => {
    const pricing: PricingSnapshot = {
      source: "online",
      updatedAt: "2026-07-19T00:00:00.000Z",
      error: "",
      models: [{
        model: "gpt-5.5",
        modelAlias: "gpt-5.5",
        inputPerMillion: 5,
        cachedInputPerMillion: 0.5,
        outputPerMillion: 30,
        source: "online",
        updatedAt: "2026-07-19T00:00:00.000Z",
      }],
    };
    const usage: ModelUsage[] = [
      { model: "gpt-5.5", inputTokens: 1_000_000, cachedInputTokens: 0, outputTokens: 0, reasoningOutputTokens: 0, totalTokens: 1_000_000 },
      { model: "unknown-model", inputTokens: 2_000_000, cachedInputTokens: 0, outputTokens: 0, reasoningOutputTokens: 0, totalTokens: 2_000_000 },
    ];

    const result = aggregateCost(usage, pricing);
    expect(result.totalTokens).toBe(3_000_000);
    expect(result.totalCost).toBe(5);
    expect(result.pricedModels).toBe(1);
    expect(result.unavailableModels).toBe(1);
  });
});
