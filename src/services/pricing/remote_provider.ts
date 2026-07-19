import { api } from "../../lib/tauri";
import { canonicalModel, createPricingCache, type ModelPricing, type PricingProvider } from "./pricing_provider";

export const OFFICIAL_PRICING_URL = "https://developers.openai.com/api/docs/pricing";
const MODEL_ID_SOURCE = "(?:gpt-\\d+(?:\\.\\d+)?(?:-[a-z0-9]+)*|o\\d+(?:-[a-z0-9]+)*)";

function documentText(html: string): string {
  if (typeof DOMParser !== "undefined") {
    const document = new DOMParser().parseFromString(html, "text/html");
    return document.body.textContent?.replace(/\s+/g, " ").trim() ?? "";
  }
  return html.replace(/<script[\s\S]*?<\/script>/gi, " ").replace(/<style[\s\S]*?<\/style>/gi, " ").replace(/<[^>]+>/g, " ").replace(/\s+/g, " ").trim();
}

export function parseOfficialPricingHtml(html: string, now = new Date()): ModelPricing[] {
  const text = documentText(html);
  const rowPattern = new RegExp(`\\b(${MODEL_ID_SOURCE})\\b\\s*\\$\\s*([0-9]+(?:\\.[0-9]+)?)\\s*(?:\\$\\s*([0-9]+(?:\\.[0-9]+)?)|-)\\s*(?:\\$\\s*[0-9]+(?:\\.[0-9]+)?|-)\\s*\\$\\s*([0-9]+(?:\\.[0-9]+)?)`, "gi");
  const models = new Map<string, ModelPricing>();
  for (const match of text.matchAll(rowPattern)) {
    const modelId = match[1]?.toLowerCase();
    const input = Number(match[2]);
    const cached = match[3] == null ? null : Number(match[3]);
    const output = Number(match[4]);
    if (!modelId || !Number.isFinite(input) || !Number.isFinite(output)) continue;
    const model = canonicalModel(modelId);
    if (models.has(model)) continue;
    models.set(model, {
      model,
      modelAlias: modelId,
      inputPerMillion: input,
      cachedInputPerMillion: cached,
      outputPerMillion: output,
      source: "online",
      updatedAt: now.toISOString(),
    });
  }
  return [...models.values()];
}

export class RemotePricingProvider implements PricingProvider {
  readonly name = "online";

  async getModels(): Promise<ModelPricing[]> {
    const html = await api.fetchPricingSource();
    const models = parseOfficialPricingHtml(html);
    if (!models.length) throw new Error("Pricing source did not contain compatible model prices");
    return models;
  }

  async getPricing(model: string): Promise<ModelPricing | null> {
    return (await this.getModels()).find((item) => item.model === canonicalModel(model)) ?? null;
  }

  async syncPricing() {
    return createPricingCache(await this.getModels(), "online");
  }
}
