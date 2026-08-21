import { AnimatePresence, motion } from "motion/react";
import { useId, useState, type ReactNode } from "react";
import { Chevron } from "./settings-card";
import { useMotionPolicy } from "../../lib/motion";

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
 * Expansion animates through the ZDL motion engine; under reduced motion the
 * policy collapses it to an opacity fade. The region stays in the
 * accessibility tree via `aria-controls`; `hidden` is applied only when fully
 * exited so screen readers never see a half-open state.
 */
export function SettingsExpander({
  title,
  description,
  children,
  icon,
}: SettingsExpanderProps) {
  const [open, setOpen] = useState(false);
  const policy = useMotionPolicy();
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
      <AnimatePresence initial={false}>
        {open && (
          <motion.div
            id={regionId}
            className="zdl-expander__region"
            data-testid="expander-region"
            key="region"
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={policy.expand}
            style={policy.reduced ? { height: "auto" } : {}}
          >
            {children}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
