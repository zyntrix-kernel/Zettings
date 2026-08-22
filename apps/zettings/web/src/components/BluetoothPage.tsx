/**
 * Bluetooth L2 page (spec §14 Template C — device manager): adapter power
 * and state from the live BlueZ area of the system snapshot; paired and
 * known devices as status rows. Mutations round-trip through the backend
 * and surface failures inline.
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

export function BluetoothPage() {
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

  if (state.phase === "loading") return <Loading label="Reading Bluetooth…" />;
  if (state.phase === "error") {
    return <ErrorBar title="Bluetooth unavailable" detail={state.message} onRetry={load} />;
  }

  const bluetooth = state.snapshot.bluetooth;
  const available = bluetooth.capability.state === "available";
  const reason =
    bluetooth.capability.state === "available"
      ? ""
      : bluetooth.capability.reason;

  const setPowered = (enabled: boolean): void => {
    setBusy(true);
    invokeIpc<void>("set_bluetooth_powered", { enabled })
      .then(() => {
        setNotice(enabled ? "Bluetooth turned on." : "Bluetooth turned off.");
        load();
      })
      .catch((cause: unknown) => {
        const message = cause instanceof Error ? cause.message : String(cause);
        setNotice(`Could not change Bluetooth: ${message}`);
      })
      .finally(() => setBusy(false));
  };

  return (
    <>
      <h1 tabIndex={-1} className="zdl-page-title">
        Bluetooth &amp; devices
      </h1>
      <p className="zdl-page-description">
        Manage the adapter and devices known to BlueZ.
      </p>

      <div className="zdl-card-grid">
        <SettingsCard
          title="Bluetooth"
          description={
            available
              ? bluetooth.powered === true
                ? `Adapter is on · ${bluetooth.devices.length} known device${bluetooth.devices.length === 1 ? "" : "s"}.`
                : "Adapter is off."
              : reason
          }
          control={
            available && bluetooth.powered !== null ? (
              <ToggleSwitch label="Bluetooth" checked={bluetooth.powered} onChange={setPowered} disabled={busy} />
            ) : undefined
          }
        />
      </div>

      {notice !== null && (
        <p role="status" className="zdl-action-notice">
          {notice}
        </p>
      )}

      <h2 className="zdl-section-title">Devices</h2>
      {available && bluetooth.devices.length > 0 ? (
        <div className="zdl-card-grid">
          {bluetooth.devices.map((device) => (
            <SettingsCard
              key={device.alias}
              title={device.alias}
              description={`${device.paired ? "Paired" : "Not paired"}${device.connected ? " · connected" : ""}`}
            />
          ))}
        </div>
      ) : (
        <p className="zdl-status">
          {available ? "No devices known yet. Pair a device to see it here." : "Device list requires the Bluetooth adapter."}
        </p>
      )}
    </>
  );
}
