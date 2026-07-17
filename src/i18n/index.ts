import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import enUS from "./locales/en-US.json";
import zhCN from "./locales/zh-CN.json";

export const supportedLanguages = ["zh-CN", "en-US"] as const;
export type Language = (typeof supportedLanguages)[number];

export function normalizeLanguage(language?: string | null): Language {
  return language?.toLowerCase().startsWith("zh") ? "zh-CN" : "en-US";
}

export function detectSystemLanguage(): Language {
  return normalizeLanguage(globalThis.navigator?.language);
}

void i18n.use(initReactI18next).init({
  resources: {
    "zh-CN": { translation: zhCN },
    "en-US": { translation: enUS },
  },
  lng: detectSystemLanguage(),
  fallbackLng: "en-US",
  supportedLngs: supportedLanguages,
  interpolation: { escapeValue: false },
  returnNull: false,
});

export default i18n;
