import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { SettingsCard } from "./settings-card";

describe("SettingsCard", () => {
  it("renders title and description as plain row when no activation", () => {
    render(<SettingsCard title="Scale" description="Change text size" />);
    expect(screen.getByText("Scale")).toBeInTheDocument();
    expect(screen.getByText("Change text size")).toBeInTheDocument();
    // No interactive role: not a navigation affordance.
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });

  it("becomes a keyboard-operable button with chevron for navigation", async () => {
    const onActivate = vi.fn();
    render(<SettingsCard title="Display" onActivate={onActivate} />);
    const button = screen.getByRole("button", { name: /display/i });
    await userEvent.click(button);
    expect(onActivate).toHaveBeenCalledOnce();
  });

  it("activates with Enter and Space (native button semantics)", async () => {
    const onActivate = vi.fn();
    render(<SettingsCard title="Power" onActivate={onActivate} />);
    const button = screen.getByRole("button", { name: /power/i });
    button.focus();
    await userEvent.keyboard("{Enter}");
    await userEvent.keyboard(" ");
    expect(onActivate).toHaveBeenCalledTimes(2);
  });

  it("disabled navigation keeps visibility but blocks activation", async () => {
    const onActivate = vi.fn();
    render(<SettingsCard title="HDR" onActivate={onActivate} disabled />);
    const button = screen.getByRole("button", { name: /hdr/i });
    expect(button).toBeDisabled();
    await userEvent.click(button);
    expect(onActivate).not.toHaveBeenCalled();
  });
});
