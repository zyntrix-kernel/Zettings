/**
 * Liquid Glass surface — ZDL material implementation (DESIGN.md §5).
 *
 * Four-layer composition per the Liquid Glass technique:
 *   L0 refraction  backdrop-filter: blur() url(#displacement) saturate/brighten
 *   L1 tint        translucent wash
 *   L2 specular    inset rim highlights on all four edges
 *   L3 content     this component's children
 *
 * Browser support: full refraction in Chromium; other engines keep the frosted
 * blur+tint (the SVG url() term is ignored). High-contrast theme and
 * `prefers-reduced-transparency` collapse everything to an opaque surface.
 *
 * Performance contract: each refracting instance owns one compositor layer.
 * Views MUST mount at most a handful of these (rail, topbar/search overlay);
 * bulk content uses plain frost classes instead.
 */
import {
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
  type CSSProperties,
  type ReactNode,
} from "react";

/** Displacement intensity. Negative = Apple-style magnifying edge lensing. */
export type DisplacementScale = number;

export interface LiquidGlassSurfaceProps {
  /** Frosted blur radius in px (skill guideline: smallest that reads frosted). */
  blur?: number;
  /**
   * Displacement scale in px. Try −20 (subtle) to −60 (dramatic); default −30.
   * Ignored on engines without SVG backdrop filters.
   */
  scale?: DisplacementScale;
  /**
   * Tint wash color. Defaults to the theme token `--glass-tint` so themes
   * control appearance centrally.
   */
  tint?: string;
  /** Adds three-pass chromatic aberration (more GPU work; use sparingly). */
  chromatic?: boolean;
  className?: string;
  style?: CSSProperties;
  children?: ReactNode;
}

/**
 * Builds an inline SVG displacement map: red gradient drives X offset, green
 * drives Y, and a blurred neutral mask confines warping to the edges so the
 * center stays optically clean (skill-blessed approach; no image assets).
 */
export function buildDisplacementMapUri(width: number, height: number, radius: number): string {
  const blurStdDev = Math.max(5, Math.round(radius * 0.35));
  const svg =
    `<svg xmlns='http://www.w3.org/2000/svg' width='${width}' height='${height}'>` +
    `<defs>` +
    `<linearGradient id='gx' x1='0%' y1='0%' x2='100%' y2='0%'>` +
    `<stop offset='0%' stop-color='#000'/><stop offset='100%' stop-color='#f00'/>` +
    `</linearGradient>` +
    `<linearGradient id='gy' x1='0%' y1='0%' x2='0%' y2='100%'>` +
    `<stop offset='0%' stop-color='#000'/><stop offset='100%' stop-color='#0f0'/>` +
    `</linearGradient>` +
    `<filter id='b'><feGaussianBlur stdDeviation='${blurStdDev}'/></filter>` +
    `</defs>` +
    `<rect width='${width}' height='${height}' rx='${radius}' fill='url(#gx)' style='mix-blend-mode:screen'/>` +
    `<rect width='${width}' height='${height}' rx='${radius}' fill='url(#gy)' style='mix-blend-mode:screen'/>` +
    `<rect width='${width}' height='${height}' rx='${radius}' fill='#808080' filter='url(#b)'/>` +
    `</svg>`;
  return `data:image/svg+xml,${encodeURIComponent(svg)}`;
}

/** Chromium-only capability (module-level, computed once, SSR-safe). */
const supportsSvgBackdrop = (): boolean => {
  if (typeof window === "undefined" || typeof navigator === "undefined") return false;
  const ua = navigator.userAgent;
  return /Chrome\/|Edg\//.test(ua) && !/Firefox\//.test(ua) && !/iPhone|iPad/.test(ua);
};

/** Opaque fallback when the user or theme forbids translucency. */
const transparencyForbidden = (): boolean => {
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") return false;
  return (
    window.matchMedia("(prefers-reduced-transparency: reduce)").matches ||
    document.documentElement.dataset.theme === "hc"
  );
};

interface Size {
  w: number;
  h: number;
}

/** Measures the surface so feImage dimensions match rendered px exactly. */
function useMeasuredSize(): { ref: React.RefObject<HTMLDivElement | null>; size: Size } {
  const ref = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState<Size>({ w: 0, h: 0 });

  useEffect(() => {
    const el = ref.current;
    if (el === null) return;
    const observer = new ResizeObserver((entries) => {
      const box = entries[0]?.contentRect;
      if (box !== undefined) {
        setSize({ w: Math.round(box.width), h: Math.round(box.height) });
      }
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  return { ref, size };
}

export function LiquidGlassSurface({
  blur = 14,
  scale = -30,
  tint,
  chromatic = false,
  className,
  style,
  children,
}: LiquidGlassSurfaceProps) {
  const uid = useId();
  const filterId = `zl-glass-${uid.replace(/[^a-zA-Z0-9]/g, "")}`;
  const { ref, size } = useMeasuredSize();
  const [forbidden, setForbidden] = useState<boolean>(() => transparencyForbidden());

  // Track theme/transparency flips that arrive after mount.
  useEffect(() => {
    if (typeof window.matchMedia !== "function") return;
    const media = window.matchMedia("(prefers-reduced-transparency: reduce)");
    const onChange = (): void => setForbidden(transparencyForbidden());
    media.addEventListener("change", onChange);
    return () => media.removeEventListener("change", onChange);
  }, []);

  const canRefract = useMemo(
    () => !forbidden && supportsSvgBackdrop() && size.w > 0 && size.h > 0,
    [forbidden, size.w, size.h],
  );

  const mapHref = useMemo(() => {
    // Radius is inherited from CSS; approximate from the minor axis for the
    // mask falloff (visual effect only — exactness not required for the map).
    const radius = Math.round(Math.min(size.w, size.h) / 4);
    return buildDisplacementMapUri(size.w, size.h, radius);
  }, [size.w, size.h]);

  const backdrop = canRefract
    ? `blur(${blur}px) url(#${filterId}) brightness(1.05) saturate(1.35)`
    : `blur(${blur}px) brightness(1.04) saturate(1.2)`;

  return (
    <>
      {canRefract && (
        <svg
          aria-hidden="true"
          focusable="false"
          width={0}
          height={0}
          style={{ position: "absolute", overflow: "hidden" }}
        >
          <defs>
            <filter
              id={filterId}
              colorInterpolationFilters="sRGB"
              x="0%"
              y="0%"
              width="100%"
              height="100%"
            >
              <feImage
                result="dispMap"
                x={0}
                y={0}
                width={size.w}
                height={size.h}
                preserveAspectRatio="none"
                href={mapHref}
              />
              {chromatic ? (
                <>
                  <feDisplacementMap
                    in="SourceGraphic"
                    in2="dispMap"
                    scale={scale}
                    xChannelSelector="R"
                    yChannelSelector="G"
                    result="dispR"
                  />
                  <feColorMatrix
                    in="dispR"
                    type="matrix"
                    values="1 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 1 0"
                    result="Ronly"
                  />
                  <feDisplacementMap
                    in="SourceGraphic"
                    in2="dispMap"
                    scale={Math.round(scale * 1.07)}
                    xChannelSelector="R"
                    yChannelSelector="G"
                    result="dispG"
                  />
                  <feColorMatrix
                    in="dispG"
                    type="matrix"
                    values="0 0 0 0 0  0 1 0 0 0  0 0 0 0 0  0 0 0 1 0"
                    result="Gonly"
                  />
                  <feDisplacementMap
                    in="SourceGraphic"
                    in2="dispMap"
                    scale={Math.round(scale * 1.035)}
                    xChannelSelector="R"
                    yChannelSelector="G"
                    result="dispB"
                  />
                  <feColorMatrix
                    in="dispB"
                    type="matrix"
                    values="0 0 0 0 0  0 0 0 0 0  0 0 1 0 0  0 0 0 1 0"
                    result="Bonly"
                  />
                  <feBlend in="Ronly" in2="Gonly" mode="screen" result="RG" />
                  <feBlend in="RG" in2="Bonly" mode="screen" />
                </>
              ) : (
                <feDisplacementMap
                  in="SourceGraphic"
                  in2="dispMap"
                  scale={scale}
                  xChannelSelector="R"
                  yChannelSelector="G"
                />
              )}
            </filter>
          </defs>
        </svg>
      )}

      <div
        ref={ref}
        className={`zlg ${forbidden ? "zlg--opaque" : ""} ${className ?? ""}`}
        style={{ ["--zlg-blur" as string]: `${blur}px`, ...style }}
      >
        {/* L0+L1+L2 are decorative; content stays in the accessibility tree. */}
        <div aria-hidden="true" className="zlg__refract" data-refract={canRefract || undefined} style={{ backdropFilter: backdrop, WebkitBackdropFilter: backdrop }} />
        <div aria-hidden="true" className="zlg__tint" style={tint !== undefined ? { background: tint } : undefined} />
        <div aria-hidden="true" className="zlg__specular" />
        <div className="zlg__content">{children}</div>
      </div>
    </>
  );
}
