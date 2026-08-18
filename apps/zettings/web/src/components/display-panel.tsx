/**
 * DisplayPanel — Monitor arrangement canvas with snapping alignment.
 *
 * Features:
 * - Drag-to-arrange monitor rectangles on a canvas
 * - Smart snapping to edges/centers of other monitors
 * - Per-monitor resolution/refresh rate selector
 * - Night light toggle + intensity slider
 * - Registers search entries for Spotlight discoverability
 *
 * Accessibility:
 * - Keyboard operable drag (Enter to pick up, arrows to move, Enter to drop)
 * - ARIA labels on all controls (ui-ux-pro-max High)
 * - Focus visible on all interactive elements
 * - Reduced motion collapses spring animations
 */
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { DisplayListOutputsResult, DisplayOutputDto, DisplayModeDto, SearchRegisterEntriesRequest, SettingsEntry, SearchHit } from "@zettings/bindings";
import { PanelShell } from "./panel-shell.js";
import { Monitor, RotateCcw, Sun, Moon, Maximize, Minimize, MoreHorizontal, Drag } from "lucide-react";
import { useSquircle } from "../lib/squircle.js";
import { useSpring, ZDL_SPRINGS } from "../lib/zdl-motion-hooks.js";
import { useSpotlightStore } from "../stores/spotlight-store.js";

interface PositionedMonitor {
  outputId: string;
  x: number;
  y: number;
  width: number;
  height: number;
  scale: number; // visual scale factor
  selectedMode: DisplayModeDto;
  nightLightEnabled: boolean;
  nightLightIntensity: number; // 0..1
}

interface SnapGuide {
  type: "vertical" | "horizontal";
  position: number;
  axis: "x" | "y";
}

const CANVAS_PADDING = 60;
const MIN_MONITOR_W = 160;
const MAX_MONITOR_W = 400;
const SNAP_THRESHOLD = 12; // px

export function DisplayPanel(): React.ReactElement {
  const [outputs, setOutputs] = useState<DisplayOutputDto[]>([]);
  const [positioned, setPositioned] = useState<PositionedMonitor[]>([]);
  const [draggingId, setDraggingId] = useState<string | null>(null);
  const [dragOffset, setDragOffset] = useState({ x: 0, y: 0 });
  const [snapGuides, setSnapGuides] = useState<SnapGuide[]>([]);
  const [canvasSize, setCanvasSize] = useState({ width: 800, height: 500 });
  const [keyboardDraggingId, setKeyboardDraggingId] = useState<string | null>(null);
  const [keyboardDragPos, setKeyboardDragPos] = useState({ x: 0, y: 0 });
  const canvasRef = useRef<HTMLDivElement>(null);
  const registerSearch = useSpotlightStore((s) => s.registerEntries);
  const unregisterSearch = useSpotlightStore((s) => s.unregisterEntries);

  // Spring for snap guide fade-in/out
  const snapGuideSpring = useSpring(snapGuides.length > 0 ? 1 : 0, ZDL_SPRINGS.slider);

  // Load display outputs on mount
  useEffect(() => {
    invoke<DisplayListOutputsResult>("zettings_display_list_outputs")
      .then((r) => {
        setOutputs(r.outputs);
        // Initialize positions in a horizontal row
        const initial: PositionedMonitor[] = r.outputs.map((o, i) => ({
          outputId: o.output_id,
          x: CANVAS_PADDING + i * (MIN_MONITOR_W + 40),
          y: CANVAS_PADDING,
          width: MIN_MONITOR_W,
          height: Math.round(MIN_MONITOR_W * 9 / 16),
          scale: 1,
          selectedMode: o.modes[0] ?? { width: 1920, height: 1080, refresh_hz: 60 },
          nightLightEnabled: false,
          nightLightIntensity: 0.5,
        }));
        setPositioned(initial);
      })
      .catch((e) => console.error("Failed to load displays:", e));
  }, []);

  // Register Spotlight entries for this panel
  useEffect(() => {
    const entries: SettingsEntry[] = [
      { id: "display-arrange", title: "Arrange Displays", description: "Drag to reposition monitors", route: "/display", keywords: ["monitor", "screen", "arrange", "position", "layout"] },
      { id: "display-night-light", title: "Night Light", description: "Reduce blue light at night", route: "/display", keywords: ["night light", "blue light", "warm", "color temperature", "eye strain"] },
      { id: "display-resolution", title: "Display Resolution", description: "Change monitor resolution and refresh rate", route: "/display", keywords: ["resolution", "refresh rate", "hz", "1080p", "1440p", "4k"] },
    ];
    registerSearch(entries);
    return () => unregisterSearch(entries.map((e) => e.id));
  }, [registerSearch, unregisterSearch]);

  // Compute canvas bounds based on positioned monitors
  const canvasBounds = useMemo(() => {
    if (positioned.length === 0) return { width: 800, height: 500 };
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    positioned.forEach((m) => {
      minX = Math.min(minX, m.x);
      minY = Math.min(minY, m.y);
      maxX = Math.max(maxX, m.x + m.width);
      maxY = Math.max(maxY, m.y + m.height);
    });
    return {
      width: Math.max(800, maxX - minX + CANVAS_PADDING * 2),
      height: Math.max(500, maxY - minY + CANVAS_PADDING * 2),
    };
  }, [positioned]);

  // Snap detection
  const computeSnap = useCallback((moving: PositionedMonitor, others: PositionedMonitor[], candidateX: number, candidateY: number) => {
    const guides: SnapGuide[] = [];
    const movingRight = candidateX + moving.width;
    const movingBottom = candidateY + moving.height;
    const movingCenterX = candidateX + moving.width / 2;
    const movingCenterY = candidateY + moving.height / 2;

    others.forEach((other) => {
      if (other.outputId === moving.outputId) return;
      const otherRight = other.x + other.width;
      const otherBottom = other.y + other.height;
      const otherCenterX = other.x + other.width / 2;
      const otherCenterY = other.y + other.height / 2;

      // Vertical snaps (left/right/center edges)
      if (Math.abs(candidateX - other.x) < SNAP_THRESHOLD) guides.push({ type: "vertical", position: other.x, axis: "x" });
      if (Math.abs(movingRight - otherRight) < SNAP_THRESHOLD) guides.push({ type: "vertical", position: otherRight - moving.width, axis: "x" });
      if (Math.abs(movingCenterX - otherCenterX) < SNAP_THRESHOLD) guides.push({ type: "vertical", position: otherCenterX - moving.width / 2, axis: "x" });
      if (Math.abs(candidateX - otherRight) < SNAP_THRESHOLD) guides.push({ type: "vertical", position: otherRight, axis: "x" });
      if (Math.abs(movingRight - other.x) < SNAP_THRESHOLD) guides.push({ type: "vertical", position: other.x - moving.width, axis: "x" });

      // Horizontal snaps (top/bottom/center edges)
      if (Math.abs(candidateY - other.y) < SNAP_THRESHOLD) guides.push({ type: "horizontal", position: other.y, axis: "y" });
      if (Math.abs(movingBottom - otherBottom) < SNAP_THRESHOLD) guides.push({ type: "horizontal", position: otherBottom - moving.height, axis: "y" });
      if (Math.abs(movingCenterY - otherCenterY) < SNAP_THRESHOLD) guides.push({ type: "horizontal", position: otherCenterY - moving.height / 2, axis: "y" });
      if (Math.abs(candidateY - otherBottom) < SNAP_THRESHOLD) guides.push({ type: "horizontal", position: otherBottom, axis: "y" });
      if (Math.abs(movingBottom - other.y) < SNAP_THRESHOLD) guides.push({ type: "horizontal", position: other.y - moving.height, axis: "y" });
    });
    return guides;
  }, []);

  const applySnap = useCallback((guides: SnapGuide[], x: number, y: number) => {
    let snappedX = x;
    let snappedY = y;
    guides.forEach((g) => {
      if (g.axis === "x" && Math.abs(x - g.position) < SNAP_THRESHOLD) snappedX = g.position;
      if (g.axis === "y" && Math.abs(y - g.position) < SNAP_THRESHOLD) snappedY = g.position;
    });
    return { x: snappedX, y: snappedY };
  }, []);

  // Mouse drag handlers
  const handleMouseDown = (e: React.MouseEvent, monitor: PositionedMonitor) => {
    if ((e.target as HTMLElement).closest("select, button, input, label")) return;
    e.preventDefault();
    const rect = canvasRef.current?.getBoundingClientRect();
    if (!rect) return;
    setDraggingId(monitor.outputId);
    setDragOffset({ x: e.clientX - rect.left - monitor.x, y: e.clientY - rect.top - monitor.y });
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (!draggingId || !canvasRef.current) return;
    const rect = canvasRef.current.getBoundingClientRect();
    const rawX = e.clientX - rect.left - dragOffset.x;
    const rawY = e.clientY - rect.top - dragOffset.y;
    const moving = positioned.find((m) => m.outputId === draggingId);
    if (!moving) return;
    const others = positioned.filter((m) => m.outputId !== draggingId);
    const guides = computeSnap(moving, others, rawX, rawY);
    setSnapGuides(guides);
    const { x, y } = applySnap(guides, rawX, rawY);
    setPositioned((prev) => prev.map((m) => (m.outputId === draggingId ? { ...m, x, y } : m)));
  };

  const handleMouseUp = () => {
    setDraggingId(null);
    setSnapGuides([]);
  };

  // Keyboard drag handlers (accessibility)
  const handleKeyDown = (e: React.KeyboardEvent, monitor: PositionedMonitor) => {
    if (keyboardDraggingId === monitor.outputId) {
      // In drag mode
      const step = e.shiftKey ? 1 : 10;
      let dx = 0, dy = 0;
      switch (e.key) {
        case "ArrowLeft": dx = -step; break;
        case "ArrowRight": dx = step; break;
        case "ArrowUp": dy = -step; break;
        case "ArrowDown": dy = step; break;
        case "Enter":
        case "Escape":
          setKeyboardDraggingId(null);
          setSnapGuides([]);
          e.preventDefault();
          return;
      }
      if (dx !== 0 || dy !== 0) {
        e.preventDefault();
        const newX = Math.max(0, keyboardDragPos.x + dx);
        const newY = Math.max(0, keyboardDragPos.y + dy);
        const moving = positioned.find((m) => m.outputId === keyboardDraggingId);
        if (moving) {
          const others = positioned.filter((m) => m.outputId !== keyboardDraggingId);
          const guides = computeSnap(moving, others, newX, newY);
          setSnapGuides(guides);
          const { x, y } = applySnap(guides, newX, newY);
          setKeyboardDragPos({ x, y });
          setPositioned((prev) => prev.map((m) => (m.outputId === keyboardDraggingId ? { ...m, x, y } : m)));
        }
      }
    } else if (e.key === "Enter" || e.key === " ") {
      // Start keyboard drag
      e.preventDefault();
      setKeyboardDraggingId(monitor.outputId);
      setKeyboardDragPos({ x: monitor.x, y: monitor.y });
    }
  };

  // Global mouse up/move for drag
  useEffect(() => {
    window.addEventListener("mousemove", handleMouseMove as EventListener);
    window.addEventListener("mouseup", handleMouseUp);
    return () => {
      window.removeEventListener("mousemove", handleMouseMove as EventListener);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [draggingId, dragOffset, positioned, computeSnap, applySnap, handleMouseMove, handleMouseUp]);

  // Mode change handler
  const handleModeChange = (outputId: string, mode: DisplayModeDto) => {
    setPositioned((prev) => prev.map((m) => (m.outputId === outputId ? { ...m, selectedMode: mode } : m)));
    // TODO: invoke zettings_display_apply_mode when implemented
  };

  // Night light handlers
  const handleNightLightToggle = (outputId: string) => {
    setPositioned((prev) => prev.map((m) => (m.outputId === outputId ? { ...m, nightLightEnabled: !m.nightLightEnabled } : m)));
  };

  const handleNightLightIntensity = (outputId: string, value: number) => {
    setPositioned((prev) => prev.map((m) => (m.outputId === outputId ? { ...m, nightLightIntensity: value } : m)));
  };

  // Scale monitor visual size based on resolution
  const getMonitorScale = (mode: DisplayModeDto) => {
    const baseW = 1920;
    return Math.max(0.6, Math.min(1.3, mode.width / baseW));
  };

  // Render monitor card (sidebar)
  const renderMonitorCard = (monitor: PositionedMonitor, idx: number) => {
    const isDragging = draggingId === monitor.outputId || keyboardDraggingId === monitor.outputId;
    const isKeyboardDragging = keyboardDraggingId === monitor.outputId;
    const squircle = useSquircle({ width: 200, height: 120, radius: 12, order: 4 });

    return (
      <div
        key={monitor.outputId}
        className="panel-card"
        style={{ opacity: isDragging ? 0.6 : 1, transform: isKeyboardDragging ? "scale(1.02)" : undefined }}
        tabIndex={0}
        onKeyDown={(e) => handleKeyDown(e, monitor)}
        onMouseDown={(e) => handleMouseDown(e, monitor)}
        data-testid={`display-card-${idx}`}
      >
        <div className="panel-card-header">
          <div>
            <h3 className="panel-card-title">{monitor.outputId}</h3>
            <p className="panel-card-subtitle">
              {monitor.selectedMode.width}×{monitor.selectedMode.height} @{monitor.selectedMode.refresh_hz}Hz
            </p>
          </div>
          <div className="panel-card-actions">
            <button
              className="panel-button panel-button-secondary"
              onClick={(e) => { e.stopPropagation(); handleNightLightToggle(monitor.outputId); }}
              aria-label={monitor.nightLightEnabled ? "Disable night light" : "Enable night light"}
              data-testid={`night-light-toggle-${idx}`}
            >
              {monitor.nightLightEnabled ? <Moon size={16} /> : <Sun size={16} />}
            </button>
          </div>
        </div>
        <div className="panel-card-body">
          <div className="panel-field">
            <label className="panel-field-label" htmlFor={`mode-${idx}`}>Resolution & Refresh Rate</label>
            <select
              id={`mode-${idx}`}
              className="panel-input"
              value={`${monitor.selectedMode.width}x${monitor.selectedMode.height}@${monitor.selectedMode.refresh_hz}`}
              onChange={(e) => {
                const [wh, hz] = e.target.value.split("@");
                const [w, h] = wh.split("x").map(Number);
                const mode = outputs.find((o) => o.output_id === monitor.outputId)?.modes.find(
                  (m) => m.width === w && m.height === h && m.refresh_hz === Number(hz)
                );
                if (mode) handleModeChange(monitor.outputId, mode);
              }}
              data-testid={`mode-select-${idx}`}
            >
              {outputs.find((o) => o.output_id === monitor.outputId)?.modes.map((m) => (
                <option key={`${m.width}x${m.height}@${m.refresh_hz}`} value={`${m.width}x${m.height}@${m.refresh_hz}`}>
                  {m.width}×{m.height} @{m.refresh_hz}Hz
                </option>
              ))}
            </select>
          </div>
          {monitor.nightLightEnabled && (
            <div className="panel-field">
              <label className="panel-field-label" htmlFor={`night-intensity-${idx}`}>
                Intensity: {Math.round(monitor.nightLightIntensity * 100)}%
              </label>
              <input
                id={`night-intensity-${idx}`}
                type="range"
                min="0"
                max="1"
                step="0.05"
                value={monitor.nightLightIntensity}
                onChange={(e) => handleNightLightIntensity(monitor.outputId, Number(e.target.value))}
                className="panel-slider"
                data-testid={`night-intensity-${idx}`}
              />
            </div>
          )}
        </div>
      </div>
    );
  };

  // Canvas monitor visual
  const renderCanvasMonitor = (monitor: PositionedMonitor, idx: number) => {
    const isDragging = draggingId === monitor.outputId || keyboardDraggingId === monitor.outputId;
    const scale = getMonitorScale(monitor.selectedMode);
    const w = monitor.width * scale;
    const h = monitor.height * scale;
    const squircle = useSquircle({ width: w, height: h, radius: 12 * scale, order: 4 });

    return (
      <div
        key={monitor.outputId}
        style={{
          position: "absolute",
          left: monitor.x,
          top: monitor.y,
          width: w,
          height: h,
          zIndex: isDragging ? 10 : 1,
          cursor: draggingId ? "grabbing" : "grab",
          opacity: isDragging ? 0.85 : 1,
          filter: monitor.nightLightEnabled ? `sepia(${monitor.nightLightIntensity * 0.6}) saturate(1.2) hue-rotate(-10deg)` : "none",
          transition: draggingId ? "none" : "transform 100ms ease-out, opacity 100ms ease-out",
        }}
        onMouseDown={(e) => handleMouseDown(e, monitor)}
        tabIndex={0}
        onKeyDown={(e) => handleKeyDown(e, monitor)}
        data-testid={`canvas-monitor-${idx}`}
      >
        <div style={squircle.clipPath ? { clipPath: squircle.clipPath, width: "100%", height: "100%" } : {}}>
          <div style={{
            width: "100%",
            height: "100%",
            background: "linear-gradient(135deg, var(--surface-elevated) 0%, var(--surface-muted) 100%)",
            border: "2px solid var(--border)",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            justifyContent: "center",
            padding: "8px",
            fontSize: "11px",
            color: "var(--text-muted)",
          }}>
            <Monitor size={20} style={{ marginBottom: "4px", color: "var(--accent)" }} />
            <strong style={{ color: "var(--text)" }}>{monitor.outputId}</strong>
            <span>{monitor.selectedMode.width}×{monitor.selectedMode.height}</span>
            {monitor.nightLightEnabled && <Moon size={12} style={{ color: "var(--accent)", marginTop: "4px" }} />}
          </div>
        </div>
        {/* Resize handle - bottom right */}
        <div
          style={{
            position: "absolute",
            right: -6,
            bottom: -6,
            width: 12,
            height: 12,
            background: "var(--accent)",
            border: "2px solid var(--surface)",
            borderRadius: "50%",
            cursor: "se-resize",
            opacity: isDragging ? 1 : 0,
            transition: "opacity 100ms",
          }}
          onMouseDown={(e) => { e.stopPropagation(); /* TODO: resize drag */ }}
          aria-label={`Resize ${monitor.outputId}`}
        />
      </div>
    );
  };

  // Snap guide lines
  const renderSnapGuides = () => {
    if (snapGuideSpring < 0.01) return null;
    return (
      <div style={{ pointerEvents: "none", position: "absolute", inset: 0, opacity: snapGuideSpring }}>
        {snapGuides.map((g, i) => (
          <div
            key={i}
            style={{
              position: "absolute",
              [g.axis === "x" ? "left" : "top"]: g.position,
              [g.axis === "x" ? "width" : "height"]: 2,
              [g.axis === "x" ? "height" : "width"]: "100%",
              background: "var(--accent)",
              opacity: 0.6,
              pointerEvents: "none",
            }}
          />
        ))}
      </div>
    );
  };

  return (
    <PanelShell
      title="Displays"
      icon={Monitor}
      subtitle="Arrange monitors, configure resolution, refresh rate, and night light"
      dataTestId="display-panel"
    >
      <div className="panel-grid" style={{ display: "grid", gridTemplateColumns: "280px 1fr", gap: "var(--space-8)", alignItems: "start" }}>
        {/* Sidebar - monitor list */}
        <aside style={{ display: "flex", flexDirection: "column", gap: "var(--space-4)", maxHeight: "calc(100vh - 280px)", overflow: "auto" }}>
          {positioned.length === 0 ? (
            <div className="panel-empty" style={{ padding: "var(--space-8)" }}>
              <Monitor className="panel-empty-icon" size={48} />
              <h3 className="panel-empty-title">No displays detected</h3>
              <p className="panel-empty-description">Connect a monitor or enable the mock backend to see sample displays.</p>
            </div>
          ) : (
            positioned.map(renderMonitorCard)
          )}
        </aside>

        {/* Canvas - visual arrangement */}
        <div style={{ position: "relative", minHeight: "500px" }}>
          <div
            ref={canvasRef}
            style={{
              position: "relative",
              width: canvasBounds.width,
              height: canvasBounds.height,
              background: "var(--surface-muted)",
              border: "1px dashed var(--border)",
              borderRadius: "12px",
              overflow: "hidden",
            }}
            data-testid="display-canvas"
          >
            {renderSnapGuides()}
            {positioned.map(renderCanvasMonitor)}
          </div>
          {/* Canvas controls */}
          <div style={{ display: "flex", gap: "var(--space-2)", marginTop: "var(--space-4)", flexWrap: "wrap" }}>
            <button className="panel-button panel-button-secondary" data-testid="reset-layout">
              <RotateCcw size={16} /> Reset Layout
            </button>
            <button className="panel-button panel-button-secondary" data-testid="auto-arrange">
              <Maximize size={16} /> Auto Arrange
            </button>
            <button className="panel-button panel-button-secondary" data-testid="mirror-displays">
              <Minimize size={16} /> Mirror Displays
            </button>
          </div>
        </div>
      </div>
    </PanelShell>
  );
}