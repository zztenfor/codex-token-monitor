import type { TFunction } from "i18next";

export function translateError(cause: unknown, t: TFunction, fallbackKey = "errors.generic"): string {
  const message = String(cause);
  if (message.includes("Codex data directory was not found")) return t("errors.codexHome");
  if (message.includes("Database") || message.includes("database")) return t("errors.database");
  if (message.includes("Invalid setting")) return t("errors.invalidSetting");
  if (message.includes("Quota") || message.includes("quota")) return t("errors.quota");
  return t(fallbackKey);
}
