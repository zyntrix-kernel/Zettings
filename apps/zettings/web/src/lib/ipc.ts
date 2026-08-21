/**
 * Typed IPC bridge.
 *
 * All shell-to-backend calls go through {@link invokeIpc} so the runtime
 * detection and error contract stay in one place. Outside the Tauri runtime
 * (plain-browser `vite` session) the bridge fails honestly — it never
 * fabricates backend data.
 */
import { invoke } from "@tauri-apps/api/core";

/** Returns whether the webview runs inside the Zettings desktop runtime. */
export function isZettingsRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** Invokes a typed command; arguments are optional and default to none. */
export async function invokeIpc<TResponse>(
  command: string,
  args?: Record<string, unknown>,
): Promise<TResponse> {
  if (!isZettingsRuntime()) {
    throw new Error(
      "Zettings runtime not detected. Launch the app through the desktop shell.",
    );
  }
  return args === undefined
    ? invoke<TResponse>(command)
    : invoke<TResponse>(command, args);
}
