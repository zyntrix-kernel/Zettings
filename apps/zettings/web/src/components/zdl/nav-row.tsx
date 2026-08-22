import type { ReactNode } from "react";
import { motion } from "motion/react";
import { useMotionPolicy } from "../../lib/motion";

export interface NavRowProps {
  /** Visible label; also the accessible name. */
  label: string;
  /** Decorative leading icon (aria-hidden). */
  icon?: ReactNode;
  /** Whether this row is the current page (aria-current="page"). */
  current?: boolean;
  /** Activation handler (route change in Phase 4). */
  onActivate: () => void;
}

/**
 * Navigation pane row (DESIGN.md §11). Native button semantics with spring
 * press feedback; the rail container owns roving-tabindex arrow keys.
 */
export function NavRow({ label, icon, current = false, onActivate }: NavRowProps) {
  const policy = useMotionPolicy();
  return (
    <motion.button
      type="button"
      className="zdl-nav-row"
      aria-current={current ? "page" : undefined}
      onClick={onActivate}
      {...(policy.press.whileTap !== undefined && { whileTap: policy.press.whileTap })}
      {...(policy.press.transition !== undefined && { transition: policy.press.transition })}
    >
      {icon !== undefined && (
        <span aria-hidden="true" style={{ display: "inline-flex" }}>
          {icon}
        </span>
      )}
      <span>{label}</span>
    </motion.button>
  );
}
