import type { ReactNode } from "react";

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
 * Navigation pane row (DESIGN.md §11). Rendered as a native button; the rail
 * container owns roving-tabindex arrow-key behavior (Phase 4 shell), so this
 * component stays a plain tab stop until then.
 */
export function NavRow({ label, icon, current = false, onActivate }: NavRowProps) {
  return (
    <button
      type="button"
      className="zdl-nav-row"
      aria-current={current ? "page" : undefined}
      onClick={onActivate}
    >
      {icon !== undefined && (
        <span aria-hidden="true" style={{ display: "inline-flex" }}>
          {icon}
        </span>
      )}
      <span>{label}</span>
    </button>
  );
}
