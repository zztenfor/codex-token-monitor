import { renderToStaticMarkup } from "react-dom/server";
import { beforeEach, describe, expect, it } from "vitest";
import i18n from "../i18n";
import { RealAccountQuotaCard } from "./RealAccountQuotaCard";

describe("RealAccountQuotaCard", () => {
  beforeEach(() => {
    void i18n.changeLanguage("en-US");
  });

  it("renders official quota values without substituting a missing window", () => {
    const markup = renderToStaticMarkup(<RealAccountQuotaCard loading={false} onRefresh={() => undefined} quota={{
      createdAt: "2026-07-16T10:00:00Z",
      fiveHour: null,
      weekly: { usedPercent: 42, remainingPercent: 58, resetAfterSeconds: 580000 },
      status: "ok",
      message: "",
    }} />);
    expect(markup).toContain("5 Hour Limit");
    expect(markup).toContain("Unavailable");
    expect(markup).toContain("42.0%");
    expect(markup).toContain("58.0%");
  });

  it("renders the authentication error message", () => {
    const markup = renderToStaticMarkup(<RealAccountQuotaCard loading={false} onRefresh={() => undefined} quota={{
      createdAt: null,
      fiveHour: null,
      weekly: null,
      status: "AUTH_NOT_FOUND",
      message: "Please login Codex first",
    }} />);
    expect(markup).toContain("Please login Codex first");
  });

  it("renders the same official state in Chinese", async () => {
    await i18n.changeLanguage("zh-CN");
    const markup = renderToStaticMarkup(<RealAccountQuotaCard loading={false} onRefresh={() => undefined} quota={{
      createdAt: null, fiveHour: null, weekly: null, status: "AUTH_NOT_FOUND", message: "",
    }} />);
    expect(markup).toContain("请先登录 Codex");
  });
});
