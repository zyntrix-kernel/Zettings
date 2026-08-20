/**
 * ZDL GlassButton — token-driven squircle action button.
 *
 * A real `<button>` (keyboard operable, focusable, disabled-aware) whose
 * surface is a G2 squircle liquid-glass material composed from the token
 * cascade. Replaces the hand-rolled `.liquid-glass-button` div stacks and
 * `border-radius` inline styles used across panels.
 *
 * Variants map to the ZDL glass densities:
 *   - `regular`    — standard action button (default)
 *   - `prominent`  — primary / key action (accent-tinted glass)
 *   - `clear`      — subtle overlay button
 *   - `destructive`— destructive action (uses `--danger` token)
 *
 * Interactive states (hover/pressed/focus-visible) live in `zdl.css` under
 * `.zdl-glass-button` so they are styled declaratively, run on the compositor
 * (transform/box-shadow only), and respect `prefers-reduced-motion`.
 *
 * Accessibility: exposes `aria-label` (passed through), a visible
 * `:focus-visible` ring via `--ring`, and reduced-motion-safe transitions.
 */
import { type CSSProperties, type ReactNode } from "react";
import { type SquircleOrder } from "../lib/zdl-motion.js";
import { useElementSize } from "../lib/use-element-size.js";
import { Squircle } from "./squircle.js";

export type GlassButtonVariant = "regular" | "prominent" | "clear" | "destructive";

export interface GlassButtonProps {
  /** Button variant. Defaults to `regular`. */
  variant?: GlassButtonVariant;
  /** Optional explicit width in CSS px. When omitted, the button self-measures. */
  width?: number;
  /** Optional explicit height in CSS px. When omitted, the button self-measures. */
  height?: number;
  /** Corner radius in CSS px. Defaults to 14 (G2, compact). */
  radius?: number;
  /** Superellipse order. Defaults to 4 (G2). */
  order?: SquircleOrder;
  /** Accessible label for icon-only buttons; overrides visible text for AT. */
  "aria-label"?: string;
  /** `aria-pressed` for toggle / selectable buttons (e.g. profile cards). */
  "aria-pressed"?: boolean;
  /** Disabled state. */
  disabled?: boolean;
  /** Optional className appended to the button (not the clipped wrapper). */
  className?: string;
  /** Optional inline style applied to the button. */
  style?: CSSProperties;
  /** Click handler. */
  onClick?: () => void;
  /** Optional type attribute. Defaults to `button`. */
  type?: "button" | "submit" | "reset";
  /** Optional data-testid for e2e targeting. */
  dataTestId?: string;
  /** Button content. */
  children?: ReactNode;
}

export function GlassButton({
  variant = "regular",
  width,
  height,
  radius = 14,
  order = 4,
  "aria-label": ariaLabel,
  "aria-pressed": ariaPressed,
  disabled = false,
  className,
  style,
  onClick,
  type = "button",
  dataTestId,
  children,
}: GlassButtonProps): React.ReactElement {
  const [ref, measured] = useElementSize<HTMLButtonElement>();
  const w = width ?? measured?.width ?? 0;
  const h = height ?? measured?.height ?? 0;

  const btnStyle: CSSProperties = {
    ...style,
    position: "relative",
    display: "inline-flex",
    alignItems: "center",
    justifyContent: "center",
    gap: "var(--space-2)",
    padding: "var(--space-2) var(--space-4)",
    font: "inherit",
    fontSize: "var(--text-sm)",
    lineHeight: "var(--text-sm-lh)",
    fontWeight: 500,
    cursor: disabled ? "not-allowed" : "pointer",
    width: width !== undefined ? `${width}px` : undefined,
    height: height !== undefined ? `${height}px` : undefined,
  };

  return (
    <button
      ref={ref}
      type={type}
      aria-label={ariaLabel}
      aria-pressed={ariaPressed}
      disabled={disabled}
      onClick={onClick}
      data-testid={dataTestId}
      className={`zdl-glass-button zdl-glass-button--${variant}${className ? ` ${className}` : ""}`}
      style={btnStyle}
    >
      {w > 0 && h > 0 ? (
        <Squircle width={w} height={h} radius={radius} order={order}>
          <span className="zdl-glass-button__content">{children}</span>
        </Squircle>
      ) : (
        <span className="zdl-glass-button__content">{children}</span>
      )}
    </button>
  );
}