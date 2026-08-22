/**
 * Network & Internet L2 page: live connectivity and Wi-Fi state from the
 * NetworkManager adapter; Wi-Fi toggles round-trip through the backend.
 */
import { useCallback, useEffect, useState } from "react";
import type { SystemSnapshotDto } from "@zettings/bindings";
import { invokeIpc } from "../lib/ipc";
import { SettingsCard, ToggleSwitch } from "./zdl";
import { ErrorBar, Loading } from "./shell/status";

type SnapshotState =
  | { phase: "loading" }
  | { phase: "ready"; snapshot: SystemSnapshotDto }
  | { phase: "error"; message: string };

function capabilityReason(
  capability: SystemSnapshotDto["network"]["capability"],
): string {
  return capability.state === "available" ? "" : capability.reason;
}

export function NetworkPage() {
  const [state, setState] = useState<SnapshotState>({ phase: "loading" });
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);

  const load = useCallback((): void => {
    setState({ phase: "loading" });
    invokeIpc<SystemSnapshotDto>("system_snapshot")
      .then((snapshot) => setState({ phase: "ready", snapshot }))
      .catch((cause: unknown) => {
        const message = cause instanceof Error ? cause.message : String(cause);
        setState({ phase: "error", message });
      });
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  if (state.phase === "loading") return <Loading label="Reading network…" />;
  if (state.phase === "error") {
    return <ErrorBar title="Network unavailable" detail={state.message} onRetry={load} />;
  }

  const network = state.snapshot.network;
  const available = network.capability.state === "available";
  const reason = capabilityReason(network.capability);

  const setWireless = (enabled: boolean): void => {
    setBusy(true);
    invokeIpc<void>("set_wireless_enabled", { enabled })
      .then(() => {
        setNotice(enabled ? "Wi-Fi turned on." : "Wi-Fi turned off.");
        load();
      })
      .catch((cause: unknown) => {
        const message = cause instanceof Error ? cause.message : String(cause);
        setNotice(`Could not change Wi-Fi: ${message}`);
      })
      .finally(() => setBusy(false));
  };

  return (
    <>
      <h1 tabIndex={-1} className="zdl-page-title">
        Internet status
      </h1>
      <p className="zdl-page-description">Live connectivity from NetworkManager.</p>

      <div className="zdl-card-grid">
        <SettingsCard
          title="NetworkManager"
          description={
            available
              ? network.networking_enabled
                ? `Networking is on · ${network.devices.length} managed device${network.devices.length === 1 ? "" : "s"}.`
                : "Networking is disabled."
              : reason
          }
        />
        <SettingsCard
          title="Wi-Fi"
          description={
            available
              ? network.wireless_enabled
                ? "Wireless is enabled."
                : "Wireless is disabled."
              : reason
          }
          control={
            available ? (
              <ToggleSwitch
                label="Wi-Fi"
                checked={network.wireless_enabled}
                onChange={setWireless}
                disabled={busy}
              />
            ) : undefined
          }
        />
      </div>

      {notice !== null && (
        <p role="status" className="zdl-action-notice">
          {notice}
        </p>
      )}
    </>
  );
}
