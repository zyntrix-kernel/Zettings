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

  it("parses L2 sub-routes into the category + sub shape", () => {
    expect(parseHash("#/system/display")).toEqual({
      kind: "category",
      category: "system",
      sub: "display",
    });
    expect(parseHash("#/devices/bluetooth")).toEqual({
      kind: "category",
      category: "devices",
      sub: "bluetooth",
    });
  });

  it("falls back to home for malformed input", () => {
    expect(parseHash("#/System")).toEqual({ kind: "home" });
    expect(parseHash("#/sys tem")).toEqual({ kind: "home" });
    expect(parseHash("#//double//slash")).toEqual({ kind: "home" });
  });
});
