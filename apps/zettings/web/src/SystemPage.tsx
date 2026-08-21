/**
 * System category page (PLAN §21 Tier 1): the first registry-driven page
 * consuming live backend adapters over typed IPC.
 *
 * Every area renders its honest capability state; mutations validate through
 * the backend and surface failures inline (spec §15).
 */
import { useCallback, useEffect, useState } from "react";
import type { NetworkStatusDto, SystemSnapshotDto } from "@zettings/bindings";
import { invokeIpc } from "./lib/ipc";
import { ComboBox, SettingsCard, ToggleSwitch } from "./components/zdl";
import { EmptyState, ErrorBar, InfoBar, Loading } from "./components/shell/status";

type SnapshotState =
  | { phase: "loading" }
  | { phase: "ready"; snapshot: SystemSnapshotDto }
  | { phase: "error"; message: string };

const POWER_LABELS: Record<string, string> = {
  "power-saver": "Power saver",
  balanced: "Balanced",
  performance: "Performance",
};

function labelProfile(profile: string): string {
  return POWER_LABELS[profile] ?? profile;
}

export function SystemPage() {
  const [state, setState] = useState<SnapshotState>({ phase: "loading" });
  const [busy, setBusy] = useState(false);
  const [actionNotice, setActionNotice] = useState<string | null>(null);

  const load = useCallback((): void => {
    setState({ phase: "loading" });
    invokeIpc<SystemSnapshotDto>("system_snapshot")
      .then((snapshot) => setState({ phase: "ready", snapshot }))
      .catch((cause: unknown) => {
        const message =
          cause instanceof Error ? cause.message : String(cause);
        setState({ phase: "error", message });
      });
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  if (state.phase === "loading") return <Loading label="Reading system state…" />;
  if (state.phase === "error") {
    return (
      <ErrorBar title="System information unavailable" detail={state.message} onRetry={load} />
    );
  }

  const { power, network, session, audio, bluetooth, display }: SystemSnapshotDto =
    state.snapshot;
  const powerAvailable = power.capability.state === "available";
  const audioAvailable = audio.capability.state === "available";
  const bluetoothAvailable = bluetooth.capability.state === "available";

  const setSinkMute = (sink: string, muted: boolean): void => {
    setBusy(true);
    invokeIpc<void>("set_audio_sink", { sink, muted })
      .then(() => {
        setActionNotice(muted ? "Output muted." : "Output unmuted.");
        load();
      })
      .catch((cause: unknown) => {
        const message = cause instanceof Error ? cause.message : String(cause);
        setActionNotice(`Could not change output: ${message}`);
      })
      .finally(() => setBusy(false));
  };

  const toggleBluetooth = (enabled: boolean): void => {
    setBusy(true);
    invokeIpc<void>("set_bluetooth_powered", { enabled })
      .then(() => {
        setActionNotice(enabled ? "Bluetooth turned on." : "Bluetooth turned off.");
        load();
      })
      .catch((cause: unknown) => {
        const message = cause instanceof Error ? cause.message : String(cause);
        setActionNotice(`Could not change Bluetooth: ${message}`);
      })
      .finally(() => setBusy(false));
  };


  const setProfile = (profile: string): void => {
    setBusy(true);
    invokeIpc<void>("set_power_profile", { profile })
      .then(() => {
        setActionNotice(`Power mode set to ${labelProfile(profile)}.`);
        load();
      })
      .catch((cause: unknown) => {
        const message = cause instanceof Error ? cause.message : String(cause);
        setActionNotice(`Could not change power mode: ${message}`);
      })
      .finally(() => setBusy(false));
  };

  const toggleWireless = (enabled: boolean): void => {
    setBusy(true);
    invokeIpc<void>("set_wireless_enabled", { enabled })
      .then(() => {
        setActionNotice(enabled ? "Wi-Fi turned on." : "Wi-Fi turned off.");
        load();
      })
      .catch((cause: unknown) => {
        const message = cause instanceof Error ? cause.message : String(cause);
        setActionNotice(`Could not change Wi-Fi: ${message}`);
      })
      .finally(() => setBusy(false));
  };

  return (
    <>
      <h2 className="zdl-section-title">Power</h2>
      <div className="zdl-card-grid">
        <SettingsCard
          title="Power mode"
          description={
            powerAvailable
              ? "Controls system-wide performance and battery behavior."
              : reasonOf(power.capability)
          }
          control={
            powerAvailable && power.available.length > 0 ? (
              <ComboBox
                label="Power mode"
                value={power.active}
                options={power.available}
                onChange={setProfile}
                disabled={busy}
              />
            ) : undefined
          }
          disabled={busy}
        />
      </div>

      <h2 className="zdl-section-title">Network</h2>
      <div className="zdl-card-grid">
        <SettingsCard
          title="Wi-Fi"
          description={
            network.capability.state === "available"
              ? `${describeDevices(network)}`
              : reasonOf(network.capability)
          }
          control={
            network.capability.state === "available" ? (
              <ToggleSwitch
                label="Wi-Fi"
                checked={network.wireless_enabled}
                onChange={toggleWireless}
                disabled={busy}
              />
            ) : undefined
          }
        />
        <SettingsCard
          title="Networking"
          description={
            network.networking_enabled
              ? "NetworkManager is managing connections."
              : "All networking is disabled."
          }
          control={<span className="zdl-card__description">{network.networking_enabled ? "On" : "Off"}</span>}
        />
      </div>

      <h2 className="zdl-section-title">Audio</h2>
      <div className="zdl-card-grid">
        {audioAvailable ? (
          audio.sinks.map((sink) => (
            <SettingsCard
              key={sink.name}
              title={sink.description !== "" ? sink.description : sink.name}
              description={`Volume ${sink.volume_percent}%${sink.is_default ? " · default output" : ""}`}
              control={
                <ToggleSwitch
                  label={`Mute ${sink.description || sink.name}`}
                  checked={sink.muted}
                  onChange={(muted) => setSinkMute(sink.name, muted)}
                  disabled={busy}
                />
              }
            />
          ))
        ) : (
          <SettingsCard
            title="Audio output"
            description={reasonOf(audio.capability)}
          />
        )}
      </div>

      <h2 className="zdl-section-title">Bluetooth</h2>
      <div className="zdl-card-grid">
        {bluetoothAvailable ? (
          <>
            <SettingsCard
              title="Bluetooth"
              description={
                bluetooth.powered === true
                  ? `${bluetooth.devices.length} device${bluetooth.devices.length === 1 ? "" : "s"} known.`
                  : "Adapter is off."
              }
              control={
                bluetooth.powered !== null && (
                  <ToggleSwitch
                    label="Bluetooth"
                    checked={bluetooth.powered}
                    onChange={toggleBluetooth}
                    disabled={busy}
                  />
                )
              }
            />
            {bluetooth.devices.map((device) => (
              <SettingsCard
                key={device.alias}
                title={device.alias}
                description={`${device.paired ? "Paired" : "Not paired"}${device.connected ? " · connected" : ""}`}
              />
            ))}
          </>
        ) : (
          <SettingsCard
            title="Bluetooth"
            description={reasonOf(bluetooth.capability)}
          />
        )}
      </div>

      <h2 className="zdl-section-title">Display</h2>
      <div className="zdl-card-grid">
        {display.capability.state === "available" ? (
          display.outputs.length > 0 ? (
            display.outputs.map((output) => (
              <SettingsCard
                key={output.name}
                title={output.name}
                description={
                  output.connected
                    ? output.current_mode !== ""
                      ? `Connected · running ${output.current_mode}`
                      : "Connected"
                    : "Disconnected"
                }
              />
            ))
          ) : (
            <EmptyState
              title="No display connectors detected"
              explanation="The kernel did not report any DRM connectors to this session."
            />
          )
        ) : (
          <SettingsCard
            title="Displays"
            description={reasonOf(display.capability)}
          />
        )}
      </div>

      <h2 className="zdl-section-title">Session</h2>
      <div className="zdl-card-grid">
        <SettingsCard
          title="Power actions"
          description={
            session.capability.state === "available"
              ? `Power off: ${session.can_power_off} · Reboot: ${session.can_reboot} · Suspend: ${session.can_suspend}`
              : reasonOf(session.capability)
          }
          control={<span className="zdl-card__description">logind</span>}
        />
      </div>

      {actionNotice !== null && (
        <InfoBar variant={actionNotice.startsWith("Could not") ? "info" : "success"}>
          {actionNotice}
        </InfoBar>
      )}
    </>
  );
}

function reasonOf(capability: { state: string; reason?: string }): string {
  return capability.reason !== undefined && capability.reason !== ""
    ? `Unavailable — ${capability.reason}`
    : "Unavailable on this system.";
}

function describeDevices(network: NetworkStatusDto): string {
  const wifi = network.devices.filter((d) => d.kind === "wifi").length;
  const ethernet = network.devices.filter((d) => d.kind === "ethernet").length;
  const parts: string[] = [];
  if (wifi > 0) parts.push(`${wifi} wireless device${wifi === 1 ? "" : "s"}`);
  if (ethernet > 0) parts.push(`${ethernet} wired device${ethernet === 1 ? "" : "s"}`);
  return parts.length > 0
    ? `Radio is ${network.wireless_enabled ? "on" : "off"} · ${parts.join(", ")}.`
    : `Radio is ${network.wireless_enabled ? "on" : "off"}.`;
}
