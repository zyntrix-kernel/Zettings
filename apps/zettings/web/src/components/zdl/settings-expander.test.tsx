import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import { SettingsExpander } from "./settings-expander";

describe("SettingsExpander", () => {
  it("starts collapsed with aria-expanded=false and no mounted region", () => {
    render(
      <SettingsExpander title="Advanced">
        <p>Hidden content</p>
      </SettingsExpander>,
    );
    const trigger = screen.getByRole("button", { name: /advanced/i });
    expect(trigger).toHaveAttribute("aria-expanded", "false");
    // AnimatePresence keeps the region out of the tree while closed.
    expect(screen.queryByTestId("expander-region")).not.toBeInTheDocument();
  });

  it("toggles aria-expanded and mounts content on activation", async () => {
    render(
      <SettingsExpander title="Advanced">
        <p>Hidden content</p>
      </SettingsExpander>,
    );
    const trigger = screen.getByRole("button", { name: /advanced/i });
    await userEvent.click(trigger);
    expect(trigger).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByTestId("expander-region")).toBeInTheDocument();
    await userEvent.click(trigger);
    expect(trigger).toHaveAttribute("aria-expanded", "false");
  });

  it("links the control to its region via aria-controls", () => {
    render(
      <SettingsExpander title="More">
        <p>x</p>
      </SettingsExpander>,
    );
    const trigger = screen.getByRole("button", { name: /more/i });
    const regionId = trigger.getAttribute("aria-controls");
    expect(regionId).toBeTruthy();
  });
});
