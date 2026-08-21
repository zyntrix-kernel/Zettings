import { useId, useState, type ReactNode } from "react";
import { Chevron } from "./settings-card";

export interface SettingsExpanderProps {
  /** Expander header title. */
  title: string;
  /** Optional header description. */
  description?: string;
  /** Subordinate content revealed on expansion (ONE level only). */
  children: ReactNode;
  /** Leading icon. */
  icon?: ReactNode;
}

/**
 * Canonical L5 inline expander (DESIGN.md §11): a disclosure button with
 * `aria-expanded`/`aria-controls` and a single expansion level — nested
 * expanders are forbidden by the Windows reconstruction spec §5.3.
 *
 * Native `<button>` semantics provide Enter/Space activation; no custom
 * keyboard handlers are added.
 */
export function SettingsExpander({
  title,
  description,
  children,
  icon,
}: SettingsExpanderProps) {
  const [open, setOpen] = useState(false);
  const regionId = useId();

  return (
    <div>
      <button
        type="button"
        className="zdl-card"
        aria-expanded={open}
        aria-controls={regionId}
        onClick={() => setOpen((v) => !v)}
      >
        {icon !== undefined && <span className="zdl-card__icon">{icon}</span>}
        <span className="zdl-card__body">
          <span className="zdl-card__title">{title}</span>
          {description !== undefined && (
            <span className="zdl-card__description">{description}</span>
          )}
        </span>
        <span className="zdl-card__chevron" aria-hidden="true">
          <Chevron className="zdl-expander__chevron" />
        </span>
      </button>
      <div
        id={regionId}
        className="zdl-expander__region"
        hidden={!open}
        data-testid="expander-region"
      >
        {children}
      </div>
    </div>
  );
}
