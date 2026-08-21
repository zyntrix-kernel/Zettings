import { useId } from "react";

export interface ComboBoxProps {
  /** Accessible label for the selection. */
  label: string;
  /** Current value. */
  value: string;
  /** Allowed options. */
  options: readonly string[];
  /** Change handler; controlled. */
  onChange: (value: string) => void;
  /** Disables interaction while staying visible/explained. */
  disabled?: boolean;
}

/**
 * Selection control on a SettingsCard: a native `<select>` so keyboard,
 * screen-reader announcement, and platform conventions come free
 * (keyboard skill: never override native select behavior).
 */
export function ComboBox({
  label,
  value,
  options,
  onChange,
  disabled = false,
}: ComboBoxProps) {
  const id = useId();
  return (
    <>
      <label htmlFor={id} className="visually-hidden">
        {label}
      </label>
      <select
        id={id}
        className="zdl-select"
        value={value}
        disabled={disabled}
        onChange={(e) => onChange(e.target.value)}
      >
        {!options.includes(value) && value !== "" && (
          <option value={value}>{value}</option>
        )}
        {options.map((option) => (
          <option key={option} value={option}>
            {option}
          </option>
        ))}
      </select>
    </>
  );
}
