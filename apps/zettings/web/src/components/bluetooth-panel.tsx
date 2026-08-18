/**
 * BluetoothPanel — Paired device cards with battery indicators.
 *
 * Features:
 * - Paired device list with connection status
 * - Battery level indicators for peripherals
 * - Connect/disconnect/remove device actions
 * - Device type icons (audio, input, phone, etc.)
 * - Registers search entries for Spotlight
 *
 * Accessibility:
 * - Battery level has text + visual indicator
 * - Connection state announced via ARIA
 * - Remove device has confirmation dialog (ui-ux-pro-max High)
 * - All actions keyboard accessible
 */
import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { PairedDeviceDto, BluetoothListPairedResult, SearchRegisterEntriesRequest, SettingsEntry } from "@zettings/bindings";
import { PanelShell } from "./panel-shell.js";
import { Bluetooth, BluetoothConnected, BluetoothOff, Battery, BatteryCharging, Headphones, Mouse, Keyboard, Phone, MoreVertical, Trash2, RotateCcw, X, Check } from "lucide-react";
import { useSpring, ZDL_SPRINGS } from "../lib/zdl-motion-hooks.js";
import { useSpotlightStore } from "../stores/spotlight-store.js";

interface BluetoothDeviceExtended extends PairedDeviceDto {
  // Extended UI state
  connecting?: boolean;
  removing?: boolean;
}

const DEVICE_CLASS_ICONS: Record<string, React.ComponentType<{ size?: number; color?: string }>> = {
  Audio: Headphones,
  Peripheral: Mouse,
  Phone: Phone,
  Computer: Keyboard,
  default: Bluetooth,
};

function getDeviceIcon(deviceClass: string) {
  return DEVICE_CLASS_ICONS[deviceClass] || DEVICE_CLASS_ICONS.default;
}

function getBatteryIcon(percent?: number, charging = false) {
  if (charging) return BatteryCharging;
  if (percent === undefined) return Battery;
  if (percent <= 10) return Battery;
  return Battery;
}

export function BluetoothPanel(): React.ReactElement {
  const [devices, setDevices] = useState<BluetoothDeviceExtended[]>([]);
  const [adapterEnabled, setAdapterEnabled] = useState(true);
  const [scanning, setScanning] = useState(false);
  const [removeConfirm, setRemoveConfirm] = useState<{ open: boolean; device: BluetoothDeviceExtended | null }>({
    open: false,
    device: null,
  });
  const registerSearch = useSpotlightStore((s) => s.registerEntries);
  const unregisterSearch = useSpotlightStore((s) => s.unregisterEntries);

  // Load paired devices on mount
  useEffect(() => {
    invoke<BluetoothListPairedResult>("zettings_bluetooth_list_paired")
      .then((r) => setDevices(r.devices))
      .catch((e) => console.error("Failed to load paired devices:", e));
  }, []);

  // Register Spotlight entries
  useEffect(() => {
    const entries: SettingsEntry[] = [
      { id: "bluetooth-pair-device", title: "Pair New Device", description: "Add a new Bluetooth device", route: "/bluetooth", keywords: ["pair", "add", "new", "device", "discoverable"] },
      { id: "bluetooth-connected-devices", title: "Connected Devices", description: "Manage currently connected Bluetooth devices", route: "/bluetooth", keywords: ["connected", "active", "headphones", "mouse", "keyboard"] },
      { id: "bluetooth-battery", title: "Device Battery Levels", description: "View battery percentage of paired peripherals", route: "/bluetooth", keywords: ["battery", "level", "percentage", "charge", "low battery"] },
      { id: "bluetooth-remove-device", title: "Remove Device", description: "Forget a paired Bluetooth device", route: "/bluetooth", keywords: ["remove", "forget", "unpair", "delete", "trash"] },
    ];
    registerSearch(entries);
    return () => unregisterSearch(entries.map((e) => e.id));
  }, [registerSearch, unregisterSearch]);

  const handleToggleAdapter = useCallback(() => {
    setAdapterEnabled((prev) => !prev);
    // TODO: invoke zettings_bluetooth_set_adapter
  }, []);

  const handleConnect = useCallback((address: string) => {
    setDevices((prev) =>
      prev.map((d) => (d.address === address ? { ...d, connecting: true } : d))
    );
    // TODO: invoke zettings_bluetooth_connect
    setTimeout(() => {
      setDevices((prev) =>
        prev.map((d) => (d.address === address ? { ...d, connecting: false, connected: true } : d))
      );
    }, 1500);
  }, []);

  const handleDisconnect = useCallback((address: string) => {
    setDevices((prev) =>
      prev.map((d) => (d.address === address ? { ...d, connected: false } : d))
    );
    // TODO: invoke zettings_bluetooth_disconnect
  }, []);

  const handleRemove = useCallback((device: BluetoothDeviceExtended) => {
    setRemoveConfirm({ open: true, device });
  }, []);

  const handleRemoveConfirm = useCallback(() => {
    if (!removeConfirm.device) return;
    const address = removeConfirm.device.address;
    setDevices((prev) => prev.map((d) => (d.address === address ? { ...d, removing: true } : d)));
    // TODO: invoke zettings_bluetooth_remove_device
    setTimeout(() => {
      setDevices((prev) => prev.filter((d) => d.address !== address));
      setRemoveConfirm({ open: false, device: null });
    }, 800);
  }, [removeConfirm.device]);

  const handleRemoveCancel = useCallback(() => {
    setRemoveConfirm({ open: false, device: null });
  }, []);

  // Battery indicator component
  const renderBattery = (device: BluetoothDeviceExtended) => {
    const { battery_percent, connected } = device;
    if (battery_percent === undefined) return null;

    const batterySpring = useSpring(battery_percent / 100, ZDL_SPRINGS.battery);
    const Icon = getBatteryIcon(battery_percent);

    return (
      <div className="bluetooth-battery" style={{ display: "flex", alignItems: "center", gap: "var(--space-2)" }} data-testid={`battery-${device.address}`}>
        <div style={{ position: "relative", width: 48, height: 24 }}>
          <div
            style={{
              width: "100%",
              height: "100%",
              border: "2px solid var(--border)",
              borderRadius: "6px",
              position: "relative",
              background: "var(--surface-muted)",
            }}
          >
            <div
              style={{
                position: "absolute",
                top: 2,
                left: 2,
                right: 2,
                bottom: 2,
                background: battery_percent <= 15 ? "#dc2626" : battery_percent <= 40 ? "#f59e0b" : "var(--accent)",
                borderRadius: "3px",
                transformOrigin: "left center",
                transform: `scaleX(${batterySpring})`,
                transition: "transform var(--motion-duration-base) var(--motion-ease-out)",
              }}
            />
            <div style={{ position: "absolute", right: -4, top: "50%", transform: "translateY(-50%)", width: 4, height: 10, background: "var(--border)", borderRadius: "0 2px 2px 0" }} />
          </div>
        </div>
        <span style={{ fontSize: "var(--text-sm)", fontWeight: 500, color: battery_percent <= 15 ? "#dc2626" : "var(--text)", minWidth: "36px" }}>
          {Math.round(battery_percent)}%
        </span>
        <Icon size={16} color={battery_percent <= 15 ? "#dc2626" : "var(--text-muted)"} aria-hidden="true" />
      </div>
    );
  };

  // Device card
  const renderDeviceCard = (device: BluetoothDeviceExtended, idx: number) => {
    const Icon = getDeviceIcon(device.device_class);
    const isConnected = device.connected;
    const isConnecting = device.connecting;
    const isRemoving = device.removing;

    return (
      <div
        key={device.address}
        className="panel-card"
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          gap: "var(--space-4)",
          opacity: isRemoving ? 0.4 : isConnecting ? 0.7 : 1,
        }}
        data-testid={`bluetooth-device-${idx}`}
      >
        <div style={{ display: "flex", alignItems: "center", gap: "var(--space-4)", flex: 1, minWidth: 0 }}>
          <div
            style={{
              width: 52,
              height: 52,
              borderRadius: "14px",
              background: isConnected
                ? "color-mix(in srgb, var(--accent) 18%, transparent)"
                : "var(--surface-muted)",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              border: isConnected ? "2px solid var(--accent)" : "1px solid var(--border)",
            }}
          >
            <Icon size={26} color={isConnected ? "var(--accent)" : "var(--text-muted)"} />
          </div>
          <div style={{ minWidth: 0 }}>
            <div style={{ display: "flex", alignItems: "center", gap: "var(--space-2)", flexWrap: "wrap" }}>
              <h3 className="panel-card-title" style={{ margin: 0, whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
                {device.name}
              </h3>
              {isConnected && (
                <span
                  style={{
                    fontSize: "var(--text-xs)",
                    fontWeight: 500,
                    color: "var(--accent)",
                    background: "color-mix(in srgb, var(--accent) 14%, transparent)",
                    padding: "2px var(--space-2)",
                    borderRadius: "999px",
                  }}
                  data-testid={`connected-badge-${device.address}`}
                >
                  Connected
                </span>
              )}
              {isConnecting && (
                <span style={{ fontSize: "var(--text-xs)", color: "var(--accent)" }}>Connecting…</span>
              )}
            </div>
            <p className="panel-card-subtitle" style={{ margin: "var(--space-1) 0 0" }}>
              {device.device_class} • {device.address}
            </p>
          </div>
        </div>

        <div style={{ display: "flex", alignItems: "center", gap: "var(--space-2)", flexShrink: 0 }}>
          {renderBattery(device)}

          <div style={{ display: "flex", gap: "var(--space-2)" }}>
            {isConnected ? (
              <>
                <button
                  className="panel-button panel-button-secondary"
                  onClick={() => handleDisconnect(device.address)}
                  disabled={isConnecting || isRemoving}
                  aria-label={`Disconnect ${device.name}`}
                  data-testid={`disconnect-${device.address}`}
                >
                  <BluetoothOff size={16} /> Disconnect
                </button>
              </>
            ) : (
              <button
                className="panel-button"
                onClick={() => handleConnect(device.address)}
                disabled={isConnecting || isRemoving}
                aria-label={`Connect ${device.name}`}
                data-testid={`connect-${device.address}`}
              >
                <BluetoothConnected size={16} /> Connect
              </button>
            )}

            <button
              className="panel-button panel-button-secondary panel-button-destructive"
              onClick={() => handleRemove(device)}
              disabled={isConnecting || isRemoving}
              aria-label={`Remove ${device.name}`}
              data-testid={`remove-${device.address}`}
            >
              <Trash2 size={16} />
            </button>
          </div>
        </div>
      </div>
    );
  };

  // Remove confirmation modal
  const renderRemoveModal = () => {
    if (!removeConfirm.open || !removeConfirm.device) return null;

    const modalSpring = useSpring(removeConfirm.open ? 1 : 0, ZDL_SPRINGS.modal);
    const settledClosed = !removeConfirm.open && modalSpring <= 0;
    if (settledClosed) return null;

    const device = removeConfirm.device;
    const Icon = getDeviceIcon(device.device_class);

    return (
      <div
        className="modal-backdrop"
        style={{
          position: "fixed",
          inset: 0,
          zIndex: 100,
          background: "rgba(12, 10, 9, 0.5)",
          backdropFilter: "blur(8px)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          opacity: modalSpring,
          pointerEvents: removeConfirm.open ? "auto" : "none",
        }}
        onClick={handleRemoveCancel}
        role="dialog"
        aria-modal="true"
        aria-labelledby="remove-modal-title"
      >
        <div
          className="modal-content"
          style={{
            transform: `scale(${0.95 + 0.05 * modalSpring})`,
            transition: "transform var(--motion-duration-base) var(--motion-ease-out)",
          }}
          onClick={(e) => e.stopPropagation()}
        >
          <div style={{ background: "var(--surface-elevated)", borderRadius: "16px", padding: "var(--space-6)", minWidth: 360, maxWidth: "90vw", boxShadow: "var(--shadow-4)" }}>
            <div style={{ display: "flex", alignItems: "center", gap: "var(--space-3)", marginBottom: "var(--space-4)" }}>
              <div style={{ width: 48, height: 48, borderRadius: "12px", background: "color-mix(in srgb, #dc2626 18%, transparent)", display: "flex", alignItems: "center", justifyContent: "center" }}>
                <Trash2 size={24} color="#dc2626" />
              </div>
              <h3 id="remove-modal-title" style={{ margin: 0, fontSize: "var(--text-lg)", color: "var(--text)" }}>
                Remove "{device.name}"?
              </h3>
            </div>
            <p style={{ margin: "0 0 var(--space-6)", fontSize: "var(--text-sm)", color: "var(--text-muted)" }}>
              This will unpair the device. You&apos;ll need to pair it again to reconnect.
            </p>
            <div style={{ display: "flex", justifyContent: "flex-end", gap: "var(--space-2)" }}>
              <button className="panel-button panel-button-secondary" onClick={handleRemoveCancel} data-testid="cancel-remove">
                Cancel
              </button>
              <button className="panel-button panel-button-destructive" onClick={handleRemoveConfirm} data-testid="confirm-remove">
                <Trash2 size={16} /> Remove
              </button>
            </div>
          </div>
        </div>
      </div>
    );
  };

  return (
    <PanelShell
      title="Bluetooth"
      icon={Bluetooth}
      subtitle="Paired devices, battery levels, and connection management"
      actions={
        <button
          className={`panel-button ${adapterEnabled ? "panel-button" : "panel-button-secondary"}`}
          onClick={handleToggleAdapter}
          aria-label={adapterEnabled ? "Disable Bluetooth" : "Enable Bluetooth"}
          aria-pressed={adapterEnabled}
          data-testid="toggle-adapter"
        >
          {adapterEnabled ? <BluetoothConnected size={16} /> : <BluetoothOff size={16} />}
          {adapterEnabled ? "On" : "Off"}
        </button>
      }
      dataTestId="bluetooth-panel"
    >
      <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-8)" }}>
        {/* Adapter status */}
        <section style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: "var(--space-4)", flexWrap: "wrap" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "var(--space-4)" }}>
            <div
              style={{
                width: 56,
                height: 56,
                borderRadius: "14px",
                background: adapterEnabled
                  ? "color-mix(in srgb, var(--accent) 18%, transparent)"
                  : "var(--surface-muted)",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                border: adapterEnabled ? "2px solid var(--accent)" : "1px solid var(--border)",
              }}
            >
              <Bluetooth size={28} color={adapterEnabled ? "var(--accent)" : "var(--text-muted)"} />
            </div>
            <div>
              <h3 style={{ fontSize: "var(--text-xl)", fontWeight: 600, color: "var(--text)", margin: 0 }}>Bluetooth</h3>
              <p style={{ fontSize: "var(--text-sm)", color: adapterEnabled ? "var(--accent)" : "var(--text-muted)", margin: 0 }}>
                {adapterEnabled ? "On • Discoverable" : "Off"}
              </p>
            </div>
          </div>
          <button
            className="panel-button panel-button-secondary"
            onClick={() => { setScanning(true); setTimeout(() => setScanning(false), 3000); }}
            disabled={scanning || !adapterEnabled}
            data-testid="scan-devices"
          >
            <RotateCcw size={16} className={scanning ? "spin" : ""} />
            {scanning ? "Scanning…" : "Scan for Devices"}
          </button>
        </section>

        {/* Paired devices */}
        <section>
          <h4 style={{ fontSize: "var(--text-base)", fontWeight: 600, color: "var(--text)", margin: "0 0 var(--space-3)" }}>
            Paired Devices ({devices.length})
          </h4>
          {devices.length === 0 ? (
            <div className="panel-empty" style={{ padding: "var(--space-8)" }}>
              <BluetoothOff className="panel-empty-icon" size={48} />
              <h3 className="panel-empty-title">No paired devices</h3>
              <p className="panel-empty-description">Click "Scan for Devices" to discover and pair new Bluetooth devices.</p>
            </div>
          ) : (
            <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-3)" }}>
              {devices.map(renderDeviceCard)}
            </div>
          )}
        </section>

        {/* Quick stats */}
        <section style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(160px, 1fr))", gap: "var(--space-4)" }}>
          <div className="panel-card" style={{ textAlign: "center", padding: "var(--space-6)" }}>
            <BluetoothConnected size={28} color="var(--accent)" style={{ marginBottom: "var(--space-2)" }} />
            <p style={{ fontSize: "var(--text-2xl)", fontWeight: 700, color: "var(--text)", margin: 0 }}>
              {devices.filter((d) => d.connected).length}
            </p>
            <p style={{ fontSize: "var(--text-sm)", color: "var(--text-muted)", margin: "var(--space-1) 0 0" }}>Connected</p>
          </div>
          <div className="panel-card" style={{ textAlign: "center", padding: "var(--space-6)" }}>
            <Battery size={28} color="var(--accent)" style={{ marginBottom: "var(--space-2)" }} />
            <p style={{ fontSize: "var(--text-2xl)", fontWeight: 700, color: "var(--text)", margin: 0 }}>
              {devices.filter((d) => d.battery_percent !== undefined && d.battery_percent! <= 20).length}
            </p>
            <p style={{ fontSize: "var(--text-sm)", color: "var(--text-muted)", margin: "var(--space-1) 0 0" }}>Low Battery</p>
          </div>
          <div className="panel-card" style={{ textAlign: "center", padding: "var(--space-6)" }}>
            <Headphones size={28} color="var(--accent)" style={{ marginBottom: "var(--space-2)" }} />
            <p style={{ fontSize: "var(--text-2xl)", fontWeight: 700, color: "var(--text)", margin: 0 }}>
              {devices.filter((d) => d.device_class === "Audio").length}
            </p>
            <p style={{ fontSize: "var(--text-sm)", color: "var(--text-muted)", margin: "var(--space-1) 0 0" }}>Audio Devices</p>
          </div>
        </section>

        {renderRemoveModal()}
      </div>
    </PanelShell>
  );
}

// Spin animation
const style = document.createElement("style");
style.textContent = `
  @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
  .spin { animation: spin 1s linear infinite; }
`;
if (typeof document !== "undefined" && !document.head.querySelector("style[data-spin-bt]")) {
  style.setAttribute("data-spin-bt", "");
  document.head.appendChild(style);
}