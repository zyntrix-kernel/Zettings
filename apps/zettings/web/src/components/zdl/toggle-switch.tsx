import { motion, useReducedMotion } from "motion/react";
import { SPRING_CONTROL } from "../../lib/motion/tokens";

export interface ToggleSwitchProps {
  /** Accessible name (visible label lives outside the switch). */
  label: string;
  /** Checked state. */
  checked: boolean;
  /** Change handler; the switch is controlled. */
  onChange: (checked: boolean) => void;
  /** Disables interaction; state stays visible and explainable. */
  disabled?: boolean;
}

/**
 * Canonical toggle control (DESIGN.md §11): a native `<button>` with
 * `role="switch"` + `aria-checked`, so Enter/Space toggling, focus, and
 * screen-reader state reporting are provided by the platform.
 *
 * The knob travels on a control spring (ZDL motion tokens); under reduced
 * motion it snaps without transition. The visual track is 40×24 px; the
 * wrapping hit area guarantees the 44 px comfortable target.
 */
export function ToggleSwitch({
  label,
  checked,
  onChange,
  disabled = false,
}: ToggleSwitchProps) {
  const reduced = useReducedMotion() ?? false;

  return (
    <span className="zdl-switch-hit">
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        aria-label={label}
        className="zdl-switch"
        disabled={disabled}
        onClick={() => onChange(!checked)}
      >
        <motion.span
          aria-hidden="true"
          className="zdl-switch__knob"
          initial={false}
          animate={{ x: checked ? 16 : 0 }}
          transition={
            reduced ? { duration: 0 } : SPRING_CONTROL
          }
        />
      </button>
    </span>
  );
}
