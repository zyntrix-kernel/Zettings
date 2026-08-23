/**
 * Custom titlebar for the frameless Zettings window.
 *
 * The bar carries `data-tauri-drag-region` so empty areas drag the window
 * (Tauri core injects the handler; double-click toggles maximize). Window
 * buttons call the Tauri window API with explicit least-privilege ACL
 * permissions (capabilities/default.json). In a plain-browser dev session
 * there is no window to control — controls render inert and hidden, and the
 * bar stays as the shell's header.
 *
 * Multi-display: position/size/maximized state persists per monitor via the
 * window-state plugin (Rust side); dragging across screens needs nothing
 * special — it is native window movement.
 */
import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, Copy, X } from "lucide-react";
import { isZettingsRuntime } from "../../lib/ipc";

export function Titlebar() {
  const [runtime] = useState(() => isZettingsRuntime());
  const [maximized, setMaximized] = useState(false);

  // Track maximized state so the restore glyph matches reality; the window
  // emits tauri://resize on every geometry change including snapping.
  useEffect(() => {
    if (!runtime) return;
    const win = getCurrentWindow();
    let disposed = false;
    const sync = (): void => {
      void win.isMaximized().then((value) => {
        if (!disposed) setMaximized(value);
      });
    };
    sync();
    const unlisten = win.onResized(sync);
    return () => {
      disposed = true;
      void unlisten.then((fn) => fn());
    };
  }, [runtime]);

  return (
    <header
      className="zdl-titlebar"
      data-tauri-drag-region=""
      style={runtime ? undefined : { display: "none" }}
    >
      <span className="zdl-titlebar__title" data-tauri-drag-region="">
        Zettings
      </span>
      <div className="zdl-titlebar__controls">
        <button
          type="button"
          className="zdl-titlebar__btn"
          aria-label="Minimize window"
          onClick={() => void getCurrentWindow().minimize()}
        >
          <Minus size={14} aria-hidden="true" />
        </button>
        <button
          type="button"
          className="zdl-titlebar__btn"
          aria-label={maximized ? "Restore window" : "Maximize window"}
          onClick={() => void getCurrentWindow().toggleMaximize()}
        >
          {maximized ? <Copy size={12} aria-hidden="true" /> : <Square size={11} aria-hidden="true" />}
        </button>
        <button
          type="button"
          className="zdl-titlebar__btn zdl-titlebar__btn--close"
          aria-label="Close window"
          onClick={() => void getCurrentWindow().close()}
        >
          <X size={14} aria-hidden="true" />
        </button>
      </div>
    </header>
  );
}
