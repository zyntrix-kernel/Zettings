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
 * The visual track is 40×24 px; the wrapping hit area guarantees the 44 px
 * comfortable target without enlarging the visual.
 */
export function ToggleSwitch({
  label,
  checked,
  onChange,
  disabled = false,
}: ToggleSwitchProps) {
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
      />
    </span>
  );
}
