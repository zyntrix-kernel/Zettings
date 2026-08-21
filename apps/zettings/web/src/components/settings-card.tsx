/**
 * SettingsCard — the L4 "setting entity" primitive from the Windows 11
 * Settings reconstruction spec (§5.2):
 *
 *   ┌────────────────────────────────────────────────────────┐
 *   │ [icon]  Setting title                         [control] │
 *   │         Description / supporting text                  │
 *   └────────────────────────────────────────────────────────┘
 *
 * Render rules (keyboard skill: native elements only):
 *   - `href`      → <a> (real link semantics for navigation)
 *   - `onClick`   → <button type="button"> (Enter/Space activation free)
 *   - otherwise   → <div> (pure status row)
 *
 * Material: light Liquid Glass row clipped to a G2 squircle via
 * `useSquircleClip`; hover lifts on the compositor (transform only).
 */
import { type CSSProperties, type ReactNode } from "react";
import { ChevronRight } from "lucide-react";
import type { LucideProps } from "lucide-react";
import { useSquircleClip } from "./squircle-surface.js";

export interface SettingsCardProps {
  /** Leading icon (decorative; the title carries the accessible name). */
  icon?: React.ComponentType<LucideProps>;
  /** Primary label. */
  title: string;
  /** Supporting text under the title. */
  description?: string;
  /** Right-side control slot: toggle, slider, value text, button… */
  control?: ReactNode;
  /** When set, the whole card is a real navigation link. */
  href?: string;
  /** When set (and no href), the whole card is a real button. */
  onClick?: () => void;
  /** Show a trailing chevron affordance (navigation rows). */
  chevron?: boolean;
  /** Accessible name override when the visible title needs context. */
  ariaLabel?: string;
  /** Disabled state (button/link variants only). */
  disabled?: boolean;
}

export function SettingsCard({
  icon: Icon,
  title,
  description,
  control,
  href,
  onClick,
  chevron = false,
  ariaLabel,
  disabled = false,
}: SettingsCardProps): React.ReactElement {
  const { ref, clipStyle, defs } = useSquircleClip(14, 4);

  const inner = (
    <>
      {Icon ? (
        <span className="settings-card-icon" aria-hidden="true">
          <Icon size={20} strokeWidth={2} />
        </span>
      ) : null}
      <span className="settings-card-text">
        <span className="settings-card-title">{title}</span>
        {description ? (
          <span className="settings-card-description">{description}</span>
        ) : null}
      </span>
      {control ? <span className="settings-card-control">{control}</span> : null}
      {chevron ? (
        <ChevronRight size={18} className="settings-card-chevron" aria-hidden="true" />
      ) : null}
    </>
  );

  // .settings-card is display:flex per zdl.css; spans above need wrappers to
  // participate correctly, so the text/icon/control classes carry their own
  // flex roles. Interactive semantics come from the native element below.
  let element: React.ReactElement;
  const style: CSSProperties | undefined = clipStyle;

  if (href !== undefined && !disabled) {
    element = (
      <a ref={ref as React.Ref<HTMLAnchorElement>} href={href} className="settings-card" style={style} aria-label={ariaLabel}>
        {inner}
      </a>
    );
  } else if (onClick !== undefined && !disabled) {
    element = (
      <button ref={ref as React.Ref<HTMLButtonElement>} type="button" onClick={onClick} className="settings-card" style={style} aria-label={ariaLabel}>
        {inner}
      </button>
    );
  } else {
    element = (
      <div ref={ref as React.Ref<HTMLDivElement>} className="settings-card" style={style} aria-label={ariaLabel} aria-disabled={disabled || undefined}>
        {inner}
      </div>
    );
  }

  return (
    <>
      {element}
      {defs}
    </>
  );
}
