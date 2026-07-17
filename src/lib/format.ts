import type { TFunction } from "i18next";

export function formatTokens(value: number, language = "en-US"): string {
  return new Intl.NumberFormat(language, {
    notation: value >= 100_000 ? "compact" : "standard",
    maximumFractionDigits: 1,
  }).format(value);
}

export function shortSession(id: string): string {
  return id.length > 18 ? `${id.slice(0, 8)}...${id.slice(-6)}` : id;
}

export function formatDuration(seconds: number | null, t: TFunction): string {
  if (seconds == null) return t("format.noUsage");
  const safeSeconds = Math.max(0, Math.floor(seconds));
  const days = Math.floor(safeSeconds / 86_400);
  const hours = Math.floor((safeSeconds % 86_400) / 3_600);
  const minutes = Math.ceil((safeSeconds % 3_600) / 60);
  if (days > 0) return t("format.daysHours", { days, hours });
  return t("format.hoursMinutes", { hours, minutes });
}
