import { beforeEach, describe, expect, it } from "vitest";
import i18n, { normalizeLanguage } from ".";

describe("i18n", () => {
  beforeEach(() => {
    void i18n.changeLanguage("en-US");
  });

  it("switches all shared translations without a reload", async () => {
    expect(i18n.t("dashboard.title")).toBe("Dashboard");
    await i18n.changeLanguage("zh-CN");
    expect(i18n.t("dashboard.title")).toBe("仪表盘");
    expect(i18n.t("settings.language")).toBe("语言");
  });

  it("normalizes unsupported future locales to the safe fallback", () => {
    expect(normalizeLanguage("ja-JP")).toBe("en-US");
    expect(normalizeLanguage("zh-TW")).toBe("zh-CN");
  });
});
