import type { ReactNode } from "react";
import { motion } from "motion/react";
import { useMotionPolicy } from "../../lib/motion";

/**
 * Chevron glyph used by navigation affordances (decorative; the row's
 * accessible name carries the meaning).
 */
export function Chevron({ className }: { className?: string }) {
  return (
    <svg
      aria-hidden="true"
      focusable="false"
      viewBox="0 0 24 24"
      width={16}
      height={16}
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
    >
      <path d="m6 9 6 6 6-6" />
    </svg>
  );
}

export interface SettingsCardProps {
  /** Card title (required — every setting is named). */
  title: string;
  /** Supporting description rendered under the title. */
  description?: string;
  /** Decorative leading icon (Lucide name resolved by caller). */
  icon?: ReactNode;
  /**
   * Right-side control zone (switch, combo, status text…). When omitted and
   * `onActivate` is provided the card becomes a navigation affordance.
   */
  control?: ReactNode;
  /** Activates navigation behavior; renders as a button with chevron. */
  onActivate?: () => void;
  /** Disables interaction while keeping full visibility (spec §15). */
  disabled?: boolean;
}

/**
 * Canonical L4 settings entity (DESIGN.md §11).
 *
 * A card is either a passive row with a control, or a navigation affordance —
 * never both ambiguously. Rendered as a native `<button>` when interactive so
 * Enter/Space activation and focus semantics come for free.
 */
export function SettingsCard({
  title,
  description,
  icon,
  control,
  onActivate,
  disabled = false,
}: SettingsCardProps) {
  const policy = useMotionPolicy();
  const body = (
    <>
      {icon !== undefined && <span className="zdl-card__icon">{icon}</span>}
      <span className="zdl-card__body">
        <span className="zdl-card__title">{title}</span>
        {description !== undefined && (
          <span className="zdl-card__description">{description}</span>
        )}
      </span>
      {control !== undefined && (
        <span className="zdl-card__control">{control}</span>
      )}
      {onActivate !== undefined && (
        <span className="zdl-card__chevron" aria-hidden="true">
          <Chevron className="zdl-expander__chevron" />
        </span>
      )}
    </>
  );

  if (onActivate === undefined) {
    return (
      <div className="zdl-card" data-hoverable="true" aria-disabled={disabled}>
        {body}
      </div>
    );
  }

  return (
    <motion.button
      type="button"
      className="zdl-card"
      onClick={onActivate}
      disabled={disabled}
      {...(policy.press.whileTap !== undefined && { whileTap: policy.press.whileTap })}
      {...(policy.press.transition !== undefined && { transition: policy.press.transition })}
    >
      {body}
    </motion.button>
  );
}
