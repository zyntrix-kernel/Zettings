/**
 * ZDL ShellFrame — Tauri custom title bar shell.
 *
 * Provides the outermost window chrome: a slim transparent title bar with a
 * drag region, the Zettings brand title + glyph on the left, and space on
 * the right that `tauri-plugin-decorum`'s `controls.js` injects the native
 * minimize / maximize-toggle / close buttons into.
 *
 * Decorum protocol:
 *   1. App must call `WebviewWindowExt::create_overlay_titlebar()` on the
 *      main window (see `apps/zettings/src/main.rs`).
 *   2. `tauri.conf.json` must set `app.withGlobalTauri: true` so the
 *      injected `controls.js` can find `window.__TAURI__`.
 *   3. This component renders a container with `data-tauri-decorum-tb`
 *      and an inner `data-tauri-drag-region` element — that's all decorum
 *      needs to position the title bar and install native controls.
 *
 * Performance note (ui-ux-pro-max/react): this component is intentionally
 * NOT memoized — it receives arbitrary `children` and would re-render
 * regardless; wrapping in memo would only add comparison overhead.
 *
 * Accessibility (ui-ux-pro-max/ux): no infinite decorative animations.
 * The brand glyph is `aria-hidden`; the title text carries the semantic
 * label. prefers-reduced-motion is respected via the global token override
 * in zdl.css (motion durations collapse to 0ms).
 */
import { type ReactNode } from "react";

export interface ShellFrameProps {
  /** Static text shown on the left of the title bar. */
  title?: string;
  /** Main content rendered below the title bar. */
  children?: ReactNode;
}

export function ShellFrame({
  title = "Zettings",
  children,
}: ShellFrameProps): React.ReactElement {
  return (
    <div className="shell-frame">
      {/*
        data-tauri-decorum-tb marks this as the titlebar for tauri-plugin-decorum.
        The inner drag-region element is required so the user can move the window.
        Decorum's controls.js will append the .decorum-tb-btn buttons here.
      */}
      <div
        className="shell-titlebar"
        data-tauri-decorum-tb=""
        role="banner"
        aria-label={`${title} window controls`}
      >
        <div className="shell-titlebar-brand" aria-hidden="true">
          <span className="brand-glyph" />
          <span className="shell-titlebar-title">{title}</span>
        </div>
        <div
          className="shell-titlebar-drag"
          data-tauri-drag-region=""
          aria-hidden="true"
        />
      </div>
      {children}
    </div>
  );
}
