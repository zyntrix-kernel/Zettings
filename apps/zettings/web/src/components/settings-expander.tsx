/**
 * SettingsExpander — the L5 "one-level expander" from the Win11 Settings
 * spec (§5.3): a header row that discloses subordinate settings.
 *
 * Disclosure pattern (keyboard + navigation skills):
 *   - Native <button> with `aria-expanded` + `aria-controls`.
 *   - Content region labelled by the header title.
 *   - Expansion animates via CSS `grid-template-rows: 0fr → 1fr`
 *     (compositor-friendly; no JS height measurement, no width/height
 *     animation per ui-ux-pro-max §7).
 *   - Reduced motion collapses the transition (zdl.css).
 */
import { useId, useState, type ReactNode } from "react";
import { ChevronDown } from "lucide-react";
import type { LucideProps } from "lucide-react";
import { useSquircleClip } from "./squircle-surface.js";

export interface SettingsExpanderProps {
  /** Header label. */
  title: string;
  /** Supporting text under the header label. */
  description?: string;
  /** Leading icon. */
  icon?: React.ComponentType<LucideProps>;
  /** Optional summary value shown before the chevron (e.g. current state). */
  summary?: ReactNode;
  /** Collapsed by default; controlled usage optional via these two props. */
  defaultOpen?: boolean;
  open?: boolean;
  onToggle?: (open: boolean) => void;
  /** Disclosed content — typically nested SettingsCards/fields. */
  children: ReactNode;
}

export function SettingsExpander({
  title,
  description,
  icon: Icon,
  summary,
  defaultOpen = false,
  open,
  onToggle,
  children,
}: SettingsExpanderProps): React.ReactElement {
  const [uncontrolledOpen, setUncontrolledOpen] = useState(defaultOpen);
  const isOpen = open ?? uncontrolledOpen;

  const contentId = useId();
  const headerClip = useSquircleClip(14, 4);

  const toggle = (): void => {
    const next = !isOpen;
    if (open === undefined) setUncontrolledOpen(next);
    onToggle?.(next);
  };

  return (
    <div className="settings-expander">
      <button
        ref={headerClip.ref as React.Ref<HTMLButtonElement>}
        type="button"
        className="settings-card"
        style={headerClip.clipStyle}
        aria-expanded={isOpen}
        aria-controls={contentId}
        onClick={toggle}
      >
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
        {summary ? <span className="settings-card-control">{summary}</span> : null}
        <ChevronDown
          size={18}
          className="settings-card-chevron"
          style={{ transform: isOpen ? "rotate(180deg)" : "rotate(0deg)" }}
          aria-hidden="true"
        />
      </button>
      {headerClip.defs}
      <div id={contentId} className="settings-expander-body" data-open={isOpen}>
        <div className="settings-expander-inner">
          <div className="settings-expander-content">{children}</div>
        </div>
      </div>
    </div>
  );
}
