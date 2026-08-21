import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { useState } from "react";
import { ToggleSwitch } from "./toggle-switch";

function Harness({ initial }: { initial: boolean }) {
  const [checked, setChecked] = useState(initial);
  return (
    <ToggleSwitch label="Wi-Fi" checked={checked} onChange={setChecked} />
  );
}

describe("ToggleSwitch", () => {
  it("exposes role=switch with checked state", () => {
    render(<Harness initial={false} />);
    const sw = screen.getByRole("switch", { name: "Wi-Fi" });
    expect(sw).toHaveAttribute("aria-checked", "false");
  });

  it("toggles on click and reports state", async () => {
    render(<Harness initial={false} />);
    const sw = screen.getByRole("switch", { name: "Wi-Fi" });
    await userEvent.click(sw);
    expect(sw).toHaveAttribute("aria-checked", "true");
    await userEvent.click(sw);
    expect(sw).toHaveAttribute("aria-checked", "false");
  });

  it("activates with Space and Enter like a native button", async () => {
    render(<Harness initial={false} />);
    const sw = screen.getByRole("switch", { name: "Wi-Fi" });
    sw.focus();
    await userEvent.keyboard(" ");
    expect(sw).toHaveAttribute("aria-checked", "true");
  });

  it("disabled switches ignore interaction", async () => {
    const onChange = vi.fn();
    render(
      <ToggleSwitch label="Mute" checked onChange={onChange} disabled />,
    );
    await userEvent.click(screen.getByRole("switch", { name: "Mute" }));
    expect(onChange).not.toHaveBeenCalled();
  });
});
