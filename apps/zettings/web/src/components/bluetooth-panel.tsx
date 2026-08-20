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
import type {
  BluetoothConnectRequest,
  BluetoothDisconnectRequest,
  BluetoothListPairedResult,
  BluetoothRemoveRequest,
  BluetoothScanDevicesResult,
  PairedDeviceDto,
} from "@zettings/bindings";
import { PanelShell } from "./panel-shell.js";
import { GlassCard } from "./glass-card.js";
import { GlassButton } from "./glass-button.js";
import { Bluetooth, BluetoothConnected, BluetoothOff, Battery, BatteryCharging, Headphones, Mouse, Keyboard, Phone, Trash2, RotateCcw } from "lucide-react";
import { useModalSpring, useSpring, ZDL_SPRINGS } from "../lib/zdl-motion-hooks.js";

interface BluetoothDeviceExtended extends PairedDeviceDto {
  // Extended UI state
  connecting?: boolean;
  removing?: boolean;
  charging?: boolean; // Not in backend, but we can add for UI
}

const DEVICE_CLASS_ICONS: Record<string, React.ComponentType<{ size?: number; color?: string }>> = {
  Audio: Headphones,
  Peripheral: Mouse,
  Phone: Phone,
  Computer: Keyboard,
  default: Bluetooth,
};

function getDeviceIcon(deviceClass: string): React.ComponentType<{ size?: number; color?: string }> {
  const icon = DEVICE_CLASS_ICONS[deviceClass];
  return (icon ?? DEVICE_CLASS_ICONS.default) as React.ComponentType<{ size?: number; color?: string }>;
}

function getBatteryIcon(percent?: number | null, charging = false) {
  if (charging) return BatteryCharging;
  if (percent === undefined || percent === null) return Battery;
  return Battery;
}

interface DeviceCardProps {
  device: BluetoothDeviceExtended;
  idx: number;
  onConnect: (device: BluetoothDeviceExtended) => void;
  onDisconnect: (address: string) => void;
  onRemove: (device: BluetoothDeviceExtended) => void;
}

/** Paired-device card. Extracted so the panel never calls hooks in a `.map()`. */
function DeviceCard({ device, idx, onConnect, onDisconnect, onRemove }: DeviceCardProps): React.ReactElement {
  const DeviceIcon = getDeviceIcon(device.device_class);
  const BatteryIcon = getBatteryIcon(device.battery_percent, device.charging);
  const isConnected = device.connected;
  const isConnecting = Boolean(device.connecting);
  const isRemoving = Boolean(device.removing);
  const cardSpring = useSpring(isRemoving ? 0 : 1, ZDL_SPRINGS.slider);

  return (
    <GlassCard
      key={device.address}
      elevation={1}
      dataTestId={`bluetooth-device-${idx}`}
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        gap: "var(--space-4)",
        flexDirection: "row",
        opacity: cardSpring.position,
        transform: `scale(${cardSpring.position})`,
        transition: "opacity var(--motion-duration-base) var(--motion-ease-out), transform var(--motion-duration-base) var(--motion-ease-out)",
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: "var(--space-4)", flex: 1, minWidth: 0, padding: "var(--space-4)", paddingRight: 0 }}>
        <GlassCard
          width={48}
          height={48}
          radius={12}
          elevation={1}
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            border: isConnected ? "2px solid var(--accent)" : "1px solid var(--border)",
          }}
        >
          <DeviceIcon size={24} color={isConnected ? "var(--accent)" : "var(--text-muted)"} />
        </GlassCard>
        <div style={{ minWidth: 0 }}>
          <div style={{ display: "flex", alignItems: "center", gap: "var(--space-2)" }}>
            <h3 className="panel-card-title" style={{ margin: 0, whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
              {device.name || device.address}
            </h3>
            {isConnected && <BluetoothConnected size={16} color="var(--accent)" aria-label="Connected" />}
            {isConnecting && <span style={{ fontSize: "var(--text-xs)", color: "var(--accent)" }}>Connecting…</span>}
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: "var(--space-2)", marginTop: "var(--space-1)" }}>
            <span style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)", textTransform: "capitalize" }}>{device.device_class}</span>
            {device.battery_percent !== undefined && device.battery_percent !== null && (
              <>
                <BatteryIcon size={12} color="var(--text-subtle)" />
                <span style={{ fontSize: "var(--text-xs)", color: "var(--text-subtle)", fontWeight: 500 }}>
                  {device.battery_percent}%
                </span>
              </>
            )}
          </div>
        </div>
      </div>

      <div className="glass-container" style={{ display: "flex", alignItems: "center", gap: "var(--space-2)", flexShrink: 0, padding: "var(--space-4)", paddingLeft: 0 }}>
        {isConnected ? (
          <>
            <GlassButton onClick={() => onDisconnect(device.address)} disabled={isRemoving} aria-label={`Disconnect ${device.name || device.address}`} dataTestId={`disconnect-${device.address}`}>
              <BluetoothOff size={16} /> Disconnect
            </GlassButton>
            <GlassButton variant="destructive" onClick={() => onRemove(device)} disabled={isRemoving} aria-label={`Remove ${device.name || device.address}`} dataTestId={`remove-${device.address}`}>
              <Trash2 size={16} /> Remove
            </GlassButton>
          </>
        ) : (
          <GlassButton
            variant="prominent"
            onClick={() => onConnect(device)}
            disabled={isConnecting || isRemoving}
            aria-label={`Connect ${device.name || device.address}`}
            dataTestId={`connect-${device.address}`}
          >
            {isConnecting ? <RotateCcw size={16} className="spin" /> : <Bluetooth size={16} />}
            {isConnecting ? "Connecting…" : "Connect"}
          </GlassButton>
        )}
      </div>
    </GlassCard>
  );
}

interface RemoveConfirmModalProps {
  device: BluetoothDeviceExtended | null;
  onConfirm: () => void;
  onClose: () => void;
}

/** Remove-device confirmation modal. Always mounted so hooks are unconditional. */
function RemoveConfirmModal({ device, onConfirm, onClose }: RemoveConfirmModalProps): React.ReactElement {
  const modalSpring = useModalSpring(device !== null);
  const visible = device !== null;
  const settledClosed = !visible && modalSpring <= 0;
  if (settledClosed) return <></>;

  return (
    <div
      className="glass-modal__backdrop"
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 100,
        background: "rgba(12, 10, 9, 0.5)",
        backdropFilter: "blur(8px) url(#liquid-glass-refract)",
        WebkitBackdropFilter: "blur(8px) url(#liquid-glass-refract)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        opacity: modalSpring,
        pointerEvents: visible ? "auto" : "none",
      }}
      onClick={onClose}
      role="dialog"
      aria-modal="true"
      aria-labelledby="remove-modal-title"
    >
      <GlassCard
        width={440}
        radius={16}
        elevation={4}
        dataTestId="remove-modal"
        style={{
          transform: `scale(${0.95 + 0.05 * modalSpring})`,
          transition: "transform var(--motion-duration-base) var(--motion-ease-out)",
          padding: "var(--space-6)",
          maxWidth: "90vw",
          background: "var(--surface-elevated)",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <h3 id="remove-modal-title" style={{ margin: "0 0 var(--space-2)", fontSize: "var(--text-lg)", color: "var(--text)" }}>
          Remove Device
        </h3>
        <p style={{ margin: "0 0 var(--space-6)", fontSize: "var(--text-sm)", color: "var(--text-muted)" }}>
          Are you sure you want to remove <strong>{device?.name || device?.address}</strong>? This will unpair the device.
        </p>
        <div style={{ display: "flex", justifyContent: "flex-end", gap: "var(--space-2)" }}>
          <GlassButton onClick={onClose} dataTestId="cancel-remove">
            Cancel
          </GlassButton>
          <GlassButton variant="destructive" onClick={onConfirm} dataTestId="confirm-remove">
            Remove
          </GlassButton>
        </div>
      </GlassCard>
    </div>
  );
}

export function BluetoothPanel(): React.ReactElement {
  const [devices, setDevices] = useState<BluetoothDeviceExtended[]>([]);
  const [adapterEnabled, setAdapterEnabled] = useState(true);
  const [scanning, setScanning] = useState(false);
  const [removeTarget, setRemoveTarget] = useState<BluetoothDeviceExtended | null>(null);

  // Load paired devices on mount
  useEffect(() => {
    invoke<BluetoothListPairedResult>("zettings_bluetooth_list_paired")
      .then((r) => {
        setDevices(r.devices.map((d) => ({ ...d, connecting: false, removing: false })));
      })
      .catch((e) => console.error("Failed to load paired devices:", e));
  }, []);

  const handleConnect = useCallback((device: BluetoothDeviceExtended) => {
    const request: BluetoothConnectRequest = { address: device.address };
    setDevices((prev) =>
      prev.map((d) => (d.address === device.address ? { ...d, connecting: true } : d))
    );
    invoke("zettings_bluetooth_connect", request)
      .then(() => {
        setDevices((prev) =>
          prev.map((d) => (d.address === device.address ? { ...d, connecting: false, connected: true } : d))
        );
      })
      .catch((e) => {
        console.error("Failed to connect Bluetooth device:", e);
        setDevices((prev) =>
          prev.map((d) => (d.address === device.address ? { ...d, connecting: false } : d))
        );
      });
  }, []);

  const handleDisconnect = useCallback((address: string) => {
    const request: BluetoothDisconnectRequest = { address };
    invoke("zettings_bluetooth_disconnect", request).catch((e) =>
      console.error("Failed to disconnect Bluetooth device:", e)
    );
    setDevices((prev) =>
      prev.map((d) => (d.address === address ? { ...d, connected: false } : d))
    );
  }, []);

  const handleRemove = useCallback((device: BluetoothDeviceExtended) => {
    setRemoveTarget(device);
  }, []);

  const confirmRemove = useCallback(() => {
    if (!removeTarget) return;
    const address = removeTarget.address;
    const request: BluetoothRemoveRequest = { address };
    setDevices((prev) =>
      prev.map((d) => (d.address === address ? { ...d, removing: true } : d))
    );
    setRemoveTarget(null);
    invoke("zettings_bluetooth_remove", request)
      .then(() => {
        setDevices((prev) => prev.filter((d) => d.address !== address));
      })
      .catch((e) => {
        console.error("Failed to remove Bluetooth device:", e);
        setDevices((prev) =>
          prev.map((d) => (d.address === address ? { ...d, removing: false } : d))
        );
      });
  }, [removeTarget]);

  const handleCloseRemove = useCallback(() => setRemoveTarget(null), []);

  const handleScan = useCallback(async () => {
    setScanning(true);
    try {
      const result = await invoke<BluetoothScanDevicesResult>("zettings_bluetooth_scan_devices");
      setDevices((prev) => {
        const existing = new Set(prev.map((d) => d.address));
        const fresh = result.devices.map((d) => ({
          ...d,
          connecting: false,
          removing: false,
        }));
        return [...prev, ...fresh.filter((d) => !existing.has(d.address))];
      });
    } catch (e) {
      console.error("Failed to scan:", e);
    } finally {
      setScanning(false);
    }
  }, []);

  return (
    <PanelShell
      title="Bluetooth"
      icon={Bluetooth}
      subtitle="Paired devices, battery levels, and connection management"
      actions={
        <GlassButton onClick={() => void handleScan()} disabled={scanning} dataTestId="scan-bluetooth">
          <RotateCcw size={16} className={scanning ? "spin" : ""} />
          {scanning ? "Scanning…" : "Scan"}
        </GlassButton>
      }
      dataTestId="bluetooth-panel"
    >
      <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-8)" }}>
        {/* Adapter toggle */}
        <GlassCard elevation={1} style={{ padding: "var(--space-4)", display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "var(--space-4)" }}>
            <GlassCard
              width={48}
              height={48}
              radius={12}
              elevation={1}
              style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                border: adapterEnabled ? "2px solid var(--accent)" : "1px solid var(--border)",
              }}
            >
              {adapterEnabled ? <BluetoothConnected size={24} color="var(--accent)" /> : <BluetoothOff size={24} color="var(--text-muted)" />}
            </GlassCard>
            <div>
              <h3 style={{ fontSize: "var(--text-lg)", fontWeight: 600, color: "var(--text)", margin: 0 }}>Bluetooth Adapter</h3>
              <p style={{ fontSize: "var(--text-sm)", color: adapterEnabled ? "var(--accent)" : "var(--text-muted)", margin: 0 }}>
                {adapterEnabled ? "Enabled" : "Disabled"}
              </p>
            </div>
          </div>
          <GlassButton
            variant="prominent"
            onClick={() => setAdapterEnabled((a) => !a)}
            aria-label={adapterEnabled ? "Disable Bluetooth" : "Enable Bluetooth"}
            dataTestId="toggle-adapter"
          >
            {adapterEnabled ? <BluetoothOff size={16} /> : <Bluetooth size={16} />}
            {adapterEnabled ? "Disable" : "Enable"}
          </GlassButton>
        </GlassCard>

        {/* Paired devices */}
        <section>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "var(--space-4)" }}>
            <h4 style={{ fontSize: "var(--text-base)", fontWeight: 600, color: "var(--text)", margin: 0 }}>Paired Devices</h4>
            <span style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)" }}>{devices.length} device{devices.length !== 1 ? "s" : ""}</span>
          </div>
          {devices.length === 0 ? (
            <GlassCard elevation={1} className="glass-empty" style={{ padding: "var(--space-12)", display: "flex", flexDirection: "column", alignItems: "center", textAlign: "center" }}>
              <BluetoothOff className="glass-empty__icon" size={48} />
              <h3 className="glass-empty__title">No paired devices</h3>
              <p className="glass-empty__description">Click "Scan" to discover and pair new Bluetooth devices.</p>
            </GlassCard>
          ) : (
            <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-3)" }}>
              {devices.map((device, idx) => (
                <DeviceCard
                  key={device.address}
                  device={device}
                  idx={idx}
                  onConnect={handleConnect}
                  onDisconnect={handleDisconnect}
                  onRemove={handleRemove}
                />
              ))}
            </div>
          )}
        </section>

        <RemoveConfirmModal device={removeTarget} onConfirm={confirmRemove} onClose={handleCloseRemove} />
      </div>
    </PanelShell>
  );
}