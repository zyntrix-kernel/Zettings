/**
 * ZDL GlassPanel — multi-layered "liquid glass" material.
 *
 * Composes four visual layers on a G2/G3 squircle surface (Apple Liquid Glass):
 *   Layer 0: Refraction — backdrop-filter with SVG feDisplacementMap filter
 *   Layer 1: Translucent tint (`--glass-panel-bg`) + backdrop blur + saturate
 *   Layer 2: Specular edge highlight (`--glass-panel-border`)
 *   Layer 3: Elevation shadow (`--shadow-1`..`--shadow-4`) + content
 *
 * Tokens are sourced from the existing 3-tier cascade in `zdl.css` so theme
 * variants (light/dark/oled/hc) automatically re-tune the glass.
 *
 * Accessibility: respects `prefers-reduced-motion` via the global token
 * override in `zdl.css` (motion durations collapse to 0ms). The component
 * does no continuous/infinite animation — backdrop-filter re-paint is a
 * compositor-side effect only.
 */
import { type CSSProperties, type ReactNode, useEffect, useRef } from "react";
import { type SquircleOrder } from "../lib/zdl-motion.js";
import { Squircle } from "./squircle.js";
import { useSpring, ZDL_SPRINGS } from "../lib/zdl-motion-hooks.js";

/** Elevation index maps directly to `--shadow-1`..`--shadow-4` tokens. */
export type GlassElevation = 1 | 2 | 3 | 4;

export interface GlassPanelProps {
  /** Optional explicit width in CSS px. Defaults to intrinsic block size. */
  width?: number;
  /** Optional explicit height in CSS px. Defaults to intrinsic block size. */
  height?: number;
  /** Corner radius in CSS px. Defaults to 20. */
  radius?: number;
  /** Superellipse order. Defaults to 4 (G2). 6 = G3 for Apple-style liquid smoothness. */
  order?: SquircleOrder;
  /** Elevation index 1..4. Defaults to 2. */
  elevation?: GlassElevation;
  /** Optional className passed to the outer wrapper. */
  className?: string;
  /** Optional inline style passed to the outer wrapper. */
  style?: CSSProperties;
  /** Panel content. */
  children?: ReactNode;
  /** Enable Liquid Glass refraction effect (requires Chrome/Edge). Defaults to true. */
  liquidGlass?: boolean;
  /** Spring config for entrance animation. Defaults to ZDL_SPRINGS.modal. */
  springConfig?: import("../lib/zdl-motion.js").SpringConfig;
  /** When true, panel animates in on mount. */
  animateIn?: boolean;
}

const SHADOW_TOKEN: Record<GlassElevation, string> = {
  1: "var(--shadow-1)",
  2: "var(--shadow-2)",
  3: "var(--shadow-3)",
  4: "var(--shadow-4)",
};

// SVG filter ID for the liquid glass refraction effect
const LIQUID_GLASS_FILTER_ID = "zdl-liquid-glass";

// Generate the displacement map as a data URI
function generateDisplacementMapDataUri(width: number, height: number, _radius: number): string {
  // Create a radial gradient displacement map
  // R channel: horizontal displacement (center = neutral 128)
  // G channel: vertical displacement (center = neutral 128)
  const canvas = typeof document !== "undefined" ? document.createElement("canvas") : null;
  if (!canvas) return "";
  
  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext("2d");
  if (!ctx) return "";
  
  const cx = width / 2;
  const cy = height / 2;
  const maxDist = Math.min(cx, cy);
  
  const imageData = ctx.createImageData(width, height);
  const data = imageData.data;
  
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const dx = x - cx;
      const dy = y - cy;
      const dist = Math.sqrt(dx * dx + dy * dy);
      
      // Normalize distance to 0-1 at the edge
      const norm = Math.min(dist / maxDist, 1);
      
      // Create a smooth falloff - center is neutral (128), edges displace
      // Using a smoothstep for glass-like refraction
      const smooth = norm * norm * (3 - 2 * norm); // smoothstep
      const displacement = Math.round(smooth * 60); // max 60px displacement
      
      // R channel: horizontal displacement (red = right, blue = left)
      // G channel: vertical displacement (green = down, blue = up)
      // Center is 128 (neutral)
      const r = 128 + Math.round((dx / maxDist) * displacement);
      const g = 128 + Math.round((dy / maxDist) * displacement);
      const b = 128; // B channel unused for 2D displacement
      
      const idx = (y * width + x) * 4;
      data[idx] = r;
      data[idx + 1] = g;
      data[idx + 2] = b;
      data[idx + 3] = 255;
    }
  }
  
  ctx.putImageData(imageData, 0, 0);
  return canvas.toDataURL("image/png");
}

// Inject the SVG filter into the document
function injectLiquidGlassFilter(width: number, height: number, radius: number): void {
  if (typeof document === "undefined") return;
  
  // Check if filter already exists
  if (document.getElementById(LIQUID_GLASS_FILTER_ID)) return;
  
  const dataUri = generateDisplacementMapDataUri(width, height, radius);
  
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.style.cssText = "position: absolute; width: 0; height: 0; overflow: hidden; pointer-events: none;";
  svg.innerHTML = `
    <defs>
      <filter id="${LIQUID_GLASS_FILTER_ID}" color-interpolation-filters="sRGB" x="-50%" y="-50%" width="200%" height="200%">
        <feImage result="dispMap" x="0" y="0" width="${width}" height="${height}" preserveAspectRatio="none" href="${dataUri}" />
        <feDisplacementMap in="SourceGraphic" in2="dispMap" scale="-35" xChannelSelector="R" yChannelSelector="G" />
      </filter>
    </defs>
  `;
  document.body.appendChild(svg);
}

// Cleanup function
function removeLiquidGlassFilter(): void {
  if (typeof document === "undefined") return;
  const existing = document.getElementById(LIQUID_GLASS_FILTER_ID);
  if (existing) {
    existing.remove();
  }
}

export function GlassPanel({
  width,
  height,
  radius = 20,
  order = 4,
  elevation = 2,
  className,
  style,
  children,
  liquidGlass = true,
  springConfig = ZDL_SPRINGS.modal,
  animateIn = true,
}: GlassPanelProps): React.ReactElement {
  const containerRef = useRef<HTMLDivElement>(null);
  const enterSpring = useSpring(animateIn ? 1 : 0, springConfig);
  
  // Inject/remove liquid glass filter on mount/unmount
  useEffect(() => {
    if (!liquidGlass || !width || !height) return;
    injectLiquidGlassFilter(width, height, radius);
    return () => removeLiquidGlassFilter();
  }, [liquidGlass, width, height, radius]);
  
  // The outer wrapper carries backdrop-filter + tint + specular border + shadow.
  // The Squircle inside clips everything to the continuous-curvature shape.
  const surfaceStyle: CSSProperties = {
    position: "relative",
    background: "var(--glass-panel-bg)",
    // Layer 0: Refraction - Liquid Glass feDisplacementMap (Chrome/Edge only)
    // Falls back to blur only in Firefox/Safari
    backdropFilter: liquidGlass && width && height
      ? `blur(var(--glass-panel-blur)) saturate(var(--glass-panel-saturate)) url(#${LIQUID_GLASS_FILTER_ID}) brightness(1.05)`
      : "blur(var(--glass-panel-blur)) saturate(var(--glass-panel-saturate))",
    WebkitBackdropFilter: liquidGlass && width && height
      ? `blur(var(--glass-panel-blur)) saturate(var(--glass-panel-saturate)) url(#${LIQUID_GLASS_FILTER_ID}) brightness(1.05)`
      : "blur(var(--glass-panel-blur)) saturate(var(--glass-panel-saturate))",
    // Layer 2: Specular border
    border: "1px solid var(--glass-panel-border)",
    // Layer 3: Elevation shadow
    boxShadow: SHADOW_TOKEN[elevation],
    // Outer box carries the size; Squircle inherits via 100%.
    width: width !== undefined ? `${width}px` : "100%",
    height: height !== undefined ? `${height}px` : "auto",
    // Entrance animation
    opacity: enterSpring.position,
    transform: `scale(${0.95 + 0.05 * enterSpring.position})`,
    transition: animateIn ? `opacity var(--motion-duration-base) var(--motion-ease-out), transform var(--motion-duration-base) var(--motion-ease-out)` : "none",
    isolation: "isolate", // Required for Liquid Glass refraction
  };

  // Layer 1: Glass tint overlay
  const tintStyle: CSSProperties = {
    position: "absolute",
    inset: 0,
    borderRadius: radius !== undefined ? `${radius}px` : "inherit",
    background: "var(--glass-panel-tint, rgba(255, 255, 255, 0.08))",
    pointerEvents: "none",
  };

  // Layer 2: Specular rim highlight (top edge bright, bottom edge subtle)
  const specularStyle: CSSProperties = {
    position: "absolute",
    inset: 0,
    borderRadius: radius !== undefined ? `${radius}px` : "inherit",
    boxShadow: "inset 0 1px 0 rgba(255,255,255,0.5), inset 0 -1px 0 rgba(255,255,255,0.08)",
    pointerEvents: "none",
  };

  // Inner content wrapper separates layout concerns from the squircle clip.
  const innerStyle: CSSProperties = {
    position: "relative",
    // Lift content above the absolute-positioned SVG defs in <Squircle>.
    zIndex: 10,
    padding: "var(--space-6)",
    ...style,
  };

  // When dimensions are explicit, Squircle can compute its own clip-path.
  // When they are auto/intrinsic, we use a fallback pseudo-blank so the
  // clip-path tracks the content — squircle behaves via 100%-driven sizing.
  const sqWidth = width ?? 0;
  const sqHeight = height ?? 0;

  return (
    <div ref={containerRef} className={className} style={surfaceStyle}>
      {/* Layer 1: Glass tint */}
      <div style={tintStyle} aria-hidden="true" />
      {/* Layer 2: Specular rim */}
      <div style={specularStyle} aria-hidden="true" />
      {sqWidth > 0 && sqHeight > 0 ? (
        <Squircle width={sqWidth} height={sqHeight} radius={radius} order={order}>
          <div style={innerStyle}>{children}</div>
        </Squircle>
      ) : (
        // Intrinsic-size path: leave content clipped to a fallback squircle
        // only after measurement — for now, render flat with the glass
        // material intact. Phase 3 hooks the ResizeObserver-driven path.
        <div style={innerStyle}>{children}</div>
      )}
    </div>
  );
}