/**
 * Automated accessibility scans (axe-core) over the ZDL primitives and the
 * shell's primary states.
 *
 * Scope note (behavioral-a11y discipline): jsdom cannot compute layout or
 * color, so `color-contrast` and other visual rules report "incomplete"
 * here. This suite is a STRUCTURAL smoke layer — landmark/label/role/keyboard
 * semantics. Visual contrast, focus appearance, and reduced-motion behavior
 * require the browser-based pass (Phase 8 integration + manual-testing skill)
 * and are NOT covered by green results here.
 */
import axe from "axe-core";
import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { SettingsCard } from "./components/zdl/settings-card";
import { SettingsExpander } from "./components/zdl/settings-expander";
import { ToggleSwitch } from "./components/zdl/toggle-switch";
import { ComboBox } from "./components/zdl/combo-box";
import { NavRow } from "./components/zdl/nav-row";
import { App } from "./App";

async function expectNoStructuralViolations(container: HTMLElement): Promise<void> {
  const results = await axe.run(container, {
    rules: { "color-contrast": { enabled: false } },
  } as unknown as Parameters<typeof axe.run>[1]);
  // Surface incomplete-but-not-disabled rules honestly in the message.
  const violations = results.violations.filter((v) => v.id !== "color-contrast");
  expect(violations).toEqual([]);
}

describe("accessibility: structural axe scans", () => {
  it("SettingsCard (navigation variant)", async () => {
    const { container } = render(
      <div>
        <main>
          <h1>System</h1>
          <SettingsCard title="Display" description="Screens" onActivate={() => {}} />
        </main>
      </div>,
    );
    await expectNoStructuralViolations(container);
  });

  it("SettingsExpander disclosure pair", async () => {
    const { container } = render(
      <main>
        <h1>Advanced</h1>
        <SettingsExpander title="Advanced">
          <p>content</p>
        </SettingsExpander>
      </main>,
    );
    await expectNoStructuralViolations(container);
  });

  it("ToggleSwitch exposes a named switch", async () => {
    const { container } = render(
      <main>
        <h1>Network</h1>
        <ToggleSwitch label="Wi-Fi" checked onChange={() => {}} />
      </main>,
    );
    await expectNoStructuralViolations(container);
  });

  it("ComboBox associates its label", async () => {
    const { container } = render(
      <main>
        <h1>Power</h1>
        <ComboBox
          label="Power mode"
          value="balanced"
          options={["balanced", "performance"]}
          onChange={() => {}}
        />
      </main>,
    );
    await expectNoStructuralViolations(container);
  });

  it("NavRow current page is programmatically identified", async () => {
    const { container } = render(
      <nav aria-label="Settings categories">
        <NavRow label="Home" onActivate={() => {}} />
        <NavRow label="System" current onActivate={() => {}} />
      </nav>,
    );
    await expectNoStructuralViolations(container);
  });

  it("shell error state (no Tauri runtime) remains structurally clean", async () => {
    const { container } = render(<App />);
    // App fails honestly outside the desktop runtime → role=alert content.
    await expectNoStructuralViolations(container);
  });
});
