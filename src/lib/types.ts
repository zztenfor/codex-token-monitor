export type Theme = "system" | "light" | "dark";
export type Language = "zh-CN" | "en-US";
export type FloatingMode = "compact" | "detailed";

export interface TokenTotals {
  inputTokens: number;
  cachedInputTokens: number;
  outputTokens: number;
  reasoningOutputTokens: number;
  totalTokens: number;
}

export type ForecastStatus = "safe" | "warning" | "danger" | "unconfigured";

export interface QuotaWindow {
  consumedTokens: number;
  limitTokens: number;
  remainingTokens: number;
  usagePercent: number | null;
  resetAt: string | null;
  resetInSeconds: number | null;
  projectedTokens: number;
  forecastStatus: ForecastStatus;
}

export interface RealQuotaWindow {
  usedPercent: number;
  remainingPercent: number;
  resetAfterSeconds: number;
}

export interface RealAccountQuota {
  createdAt: string | null;
  fiveHour: RealQuotaWindow | null;
  weekly: RealQuotaWindow | null;
  status: string;
  message: string;
}

export interface CurrentSession extends TokenTotals {
  sessionId: string;
  model: string;
  contextWindow: number | null;
  usagePercent: number | null;
  contextTokens: number;
  updatedAt: string;
}

export interface UsagePoint extends TokenTotals {
  bucket: string;
  label: string;
}

export interface ProjectUsage {
  projectKey: string;
  displayName: string;
  totalTokens: number;
}

export interface DashboardData {
  today: TokenTotals;
  currentSession: CurrentSession | null;
  chart: UsagePoint[];
  projects: ProjectUsage[];
  monthlyTotalTokens: number;
  projectedMonthlyTokens: number;
  dailyAverageTokens: number;
  riskLevel: "low" | "medium" | "high" | "insufficient";
  fiveHourQuota: QuotaWindow;
  weeklyQuota: QuotaWindow;
  realAccountQuota: RealAccountQuota;
  syncPaused: boolean;
  lastSyncedAt: string | null;
}

export interface AppSettings {
  codexPath: string;
  refreshIntervalSeconds: 1 | 5 | 10 | 30;
  theme: Theme;
  language: Language;
  floatingOpacity: number;
  floatingAlwaysOnTop: boolean;
  floatingClickThrough: boolean;
  floatingMode: FloatingMode;
  dailyBudget: number;
  monthlyBudget: number;
  fiveHourLimit: number;
  weeklyLimit: number;
  alert80: boolean;
  alert90: boolean;
  alert95: boolean;
}

export interface FloatingWindowSettings {
  floatingOpacity: number;
  floatingAlwaysOnTop: boolean;
  floatingClickThrough: boolean;
  floatingMode: FloatingMode;
}

export interface SyncReport {
  filesScanned: number;
  eventsInserted: number;
  eventsSkipped: number;
  parseErrors: number;
  syncedAt: string;
}
