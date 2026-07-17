import { beforeEach, describe, expect, it } from "vitest";
import i18n from "../i18n";
import { formatDuration, formatTokens, shortSession } from "./format";

describe("format helpers", () => {
  beforeEach(() => {
    void i18n.changeLanguage("en-US");
  });
  it("keeps small token counts readable", () => expect(formatTokens(1234)).toContain("1,234"));
  it("compacts large token counts", () => expect(formatTokens(320000)).toMatch(/320|32万/));
  it("shortens long session identifiers", () => expect(shortSession("019f695b-aefc-7d12-8923-43e20fd862e2")).toContain("..."));
  it("formats rolling-window reset durations", () => expect(formatDuration(19_860, i18n.t.bind(i18n))).toBe("5h 31m"));
});
