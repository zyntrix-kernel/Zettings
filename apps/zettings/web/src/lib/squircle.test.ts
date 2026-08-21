import { describe, expect, it } from "vitest";
import { squircleClipPath, squirclePath, G2_ORDER, G3_ORDER } from "./squircle";

describe("squircle geometry", () => {
  it("degrades to a rectangle at zero radius", () => {
    expect(squirclePath({ width: 100, height: 50, radius: 0 })).toBe(
      "M0 0 H100 V50 H0 Z",
    );
  });

  it("clamps radius to half the minor axis", () => {
    const a = squirclePath({ width: 40, height: 40, radius: 500 });
    const b = squirclePath({ width: 40, height: 40, radius: 20 });
    expect(a).toBe(b);
  });

  it("produces a closed path with sampled points", () => {
    const d = squirclePath({ width: 200, height: 80, radius: 16, order: G2_ORDER });
    expect(d.startsWith("M")).toBe(true);
    expect(d.endsWith("Z")).toBe(true);
    // 128 samples -> 127 line segments after the initial M.
    expect(d.match(/L/g)?.length).toBe(127);
  });

  it("G3 order differs from G2 order for the same box", () => {
    const g2 = squirclePath({ width: 120, height: 120, radius: 24, order: G2_ORDER });
    const g3 = squirclePath({ width: 120, height: 120, radius: 24, order: G3_ORDER });
    expect(g2).not.toBe(g3);
  });

  it("clip-path wrapper quotes the path", () => {
    const clip = squircleClipPath({ width: 10, height: 10, radius: 4 });
    expect(clip.startsWith('path("')).toBe(true);
    expect(clip.endsWith('")')).toBe(true);
  });
});
