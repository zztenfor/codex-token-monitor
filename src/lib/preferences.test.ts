import { describe, expect, it } from "vitest";
import { parseUsageRange, readUsageRange, saveUsageRange, USAGE_RANGE_STORAGE_KEY } from "./preferences";

describe("usage history range preference", () => {
  it.each([
    ["1", 1],
    ["7", 7],
    ["30", 30],
    [30, 30],
  ])("accepts supported range %s", (value, expected) => {
    expect(parseUsageRange(value)).toBe(expected);
  });

  it.each([null, "", "14", 0, 31, undefined])("falls back to seven days for %s", (value) => {
    expect(parseUsageRange(value)).toBe(7);
  });

  it("reads and saves the selected range", () => {
    const values = new Map<string, string>([[USAGE_RANGE_STORAGE_KEY, "30"]]);
    const storage = {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => values.set(key, value),
    };

    expect(readUsageRange(storage)).toBe(30);
    saveUsageRange(1, storage);
    expect(readUsageRange(storage)).toBe(1);
  });

  it("survives an unavailable storage backend", () => {
    const storage = {
      getItem: () => { throw new Error("unavailable"); },
      setItem: () => { throw new Error("unavailable"); },
    };

    expect(readUsageRange(storage)).toBe(7);
    expect(() => saveUsageRange(30, storage)).not.toThrow();
  });
});
