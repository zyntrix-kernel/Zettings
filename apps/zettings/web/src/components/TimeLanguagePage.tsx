/**
 * Date & time L2 page: the first `time-language` surface, backed live by
 * systemd `timedated` through the typed IPC layer.
 *
 * Read-only facts (timezone, NTP sync status) render as explained cards;
 * mutations (NTP toggle, timezone selection) go through `set_datetime`,
 * validate against timedated, and surface failures inline (spec §15).
 * Privileged changes authenticate via timedated's own PolicyKit policy —
 * Zettings never prompts inside its own UI.
 */
import { useCallback, useEffect, useState } from "react";
import type { TimedateSnapshotDto } from "@zettings/bindings";
import { invokeIpc } from "../lib/ipc";
import { ComboBox, SettingsCard, ToggleSwitch } from "./zdl";
import { ErrorBar, InfoBar, Loading } from "./shell/status";
type SnapshotState =
  | { phase: "loading" }
  | { phase: "ready"; snapshot: TimedateSnapshotDto }
  | { phase: "error"; message: string };

export function TimeLanguagePage() {
  const [state, setState] = useState<SnapshotState>({ phase: "loading" });
  const [busy, setBusy] = useState(false);
  const [actionNotice, setActionNotice] = useState<string | null>(null);

  const load = useCallback((): void => {
    setState({ phase: "loading" });
    invokeIpc<TimedateSnapshotDto>("datetime_snapshot")
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

  if (state.phase === "loading") return <Loading label="Reading clock settings…" />;
  if (state.phase === "error") {
    return (
      <ErrorBar title="Date & time unavailable" detail={state.message} onRetry={load} />
    );
  }

  const snapshot = state.snapshot;
  const controllable = snapshot.capability.state === "available";

  const setNtp = (enabled: boolean): void => {
    setBusy(true);
    invokeIpc<void>("set_datetime", { ntp: enabled })
      .then(() => {
        setActionNotice(enabled ? "Automatic time sync on." : "Automatic time sync off.");
        load();
      })
      .catch((cause: unknown) => {
        const message = cause instanceof Error ? cause.message : String(cause);
        setActionNotice(`Could not change time sync: ${message}`);
      })
      .finally(() => setBusy(false));
  };

  const setTimezone = (timezone: string): void => {
    setBusy(true);
    invokeIpc<void>("set_datetime", { timezone })
      .then(() => {
        setActionNotice(`Time zone set to ${timezone}.`);
        load();
      })
      .catch((cause: unknown) => {
        const message = cause instanceof Error ? cause.message : String(cause);
        setActionNotice(`Could not change time zone: ${message}`);
      })
      .finally(() => setBusy(false));
  };

  return (
    <>
      <h1 tabIndex={-1} className="zdl-page-title">
        Date &amp; time
      </h1>
      <p className="zdl-page-description">
        System clock configuration, managed by systemd-timedated.
      </p>

      {!controllable && (
        <InfoBar variant="info">
          The clock service is not reachable on this system
          {snapshot.capability.state === "unavailable"
            ? `: ${snapshot.capability.reason ?? ""}`
            : ""}
          . Controls stay disabled until it responds.
        </InfoBar>
      )}

      <h2 className="zdl-section-title">Synchronization</h2>
      <div className="zdl-card-grid">
        <SettingsCard
          title="Set time automatically"
          description={
            snapshot.ntp_available
              ? snapshot.ntp_synchronized
                ? "The system clock is synchronized with network time."
                : "Waiting for the next successful synchronization."
              : "This system has no network time source available."
          }
          control={
            <ToggleSwitch
              label="Set time automatically"
              checked={snapshot.ntp_enabled}
              disabled={!controllable || !snapshot.ntp_available || busy}
              onChange={setNtp}
            />
          }
        />
      </div>

      <h2 className="zdl-section-title">Time zone</h2>
      <div className="zdl-card-grid">
        <SettingsCard
          title="Time zone"
          description={`Currently ${snapshot.timezone}.`}
          control={
            <ComboBox
              label="Time zone"
              value={snapshot.timezone}
              options={snapshot.available_timezones}
              disabled={!controllable || busy}
              onChange={setTimezone}
            />
          }
        />
        <SettingsCard
          title="Hardware clock"
          description={
            snapshot.local_rtc
              ? "The hardware clock stores local time."
              : "The hardware clock stores UTC (recommended for dual-boot safety)."
          }
          control={<span className="zdl-area-status">Managed by systemd</span>}
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
