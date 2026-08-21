import { describe, expect, it } from "vitest";
import { parseHash } from "./router";

describe("hash router", () => {
  it("maps empty and home hashes to home", () => {
    expect(parseHash("")).toEqual({ kind: "home" });
    expect(parseHash("#/home")).toEqual({ kind: "home" });
    expect(parseHash("#/")).toEqual({ kind: "home" });
  });

  it("parses category routes", () => {
    expect(parseHash("#/system")).toEqual({ kind: "category", category: "system" });
    expect(parseHash("#/time-language")).toEqual({
      kind: "category",
      category: "time-language",
    });
  });

  it("preserves deep segments in the category route", () => {
    // L2 pages land later; the hub still resolves.
    expect(parseHash("#/system/display")).toEqual({
      kind: "category",
      category: "system",
    });
  });

  it("falls back to home for malformed input", () => {
    expect(parseHash("#/System")).toEqual({ kind: "home" });
    expect(parseHash("#/sys tem")).toEqual({ kind: "home" });
    expect(parseHash("#//double//slash")).toEqual({ kind: "home" });
  });
});
