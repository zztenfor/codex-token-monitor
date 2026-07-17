export const USAGE_RANGE_STORAGE_KEY = "codex-token-monitor:usage-range";

export type UsageRange = 1 | 7 | 30;

interface PreferenceStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

export function parseUsageRange(value: unknown): UsageRange {
  const numeric = typeof value === "number" ? value : Number(value);
  return numeric === 1 || numeric === 7 || numeric === 30 ? numeric : 7;
}

export function readUsageRange(storage: PreferenceStorage = window.localStorage): UsageRange {
  try {
    return parseUsageRange(storage.getItem(USAGE_RANGE_STORAGE_KEY));
  } catch {
    return 7;
  }
}

export function saveUsageRange(range: UsageRange, storage: PreferenceStorage = window.localStorage): void {
  try {
    storage.setItem(USAGE_RANGE_STORAGE_KEY, String(range));
  } catch {
    // A disabled WebView storage backend should not prevent changing the range.
  }
}
