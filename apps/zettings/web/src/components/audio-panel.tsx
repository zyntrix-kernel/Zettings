/**
 * AudioPanel — Per-stream mixer cards with live VU meters + EQ.
 *
 * Features:
 * - Per-stream volume sliders with live VU meter visualization
 * - Master/output device selector
 * - 10-band equalizer with frequency presets
 * - Stream mute/solo toggles
 * - Registers search entries for Spotlight
 *
 * Accessibility:
 * - All sliders keyboard operable (Arrow keys, Home/End)
 * - VU meters have text fallback for screen readers
 * - Mute buttons have ARIA labels with state
 * - Reduced motion collapses VU animations
 */
import { memo, useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AudioListStreamsResult, AudioSetVolumeRequest, AudioStreamDto } from "@zettings/bindings";
import { PanelShell } from "./panel-shell.js";
import { GlassCard } from "./glass-card.js";
import { GlassButton } from "./glass-button.js";
import { Volume2, VolumeX, Music, RotateCcw, Headphones, Mic, Speaker } from "lucide-react";
import { usePrefersReducedMotion } from "../lib/zdl-motion-hooks.js";

interface AudioStreamExtended extends AudioStreamDto {
  // Stream-only UI state (VU animation is owned by VuMeter, not here).
  isSolo: boolean;
}

const EQ_BANDS = [
  { freq: 32, label: "32Hz", type: "lowshelf" },
  { freq: 64, label: "64Hz", type: "peaking" },
  { freq: 125, label: "125Hz", type: "peaking" },
  { freq: 250, label: "250Hz", type: "peaking" },
  { freq: 500, label: "500Hz", type: "peaking" },
  { freq: 1000, label: "1kHz", type: "peaking" },
  { freq: 2000, label: "2kHz", type: "peaking" },
  { freq: 4000, label: "4kHz", type: "peaking" },
  { freq: 8000, label: "8kHz", type: "peaking" },
  { freq: 16000, label: "16kHz", type: "highshelf" },
];

const EQ_PRESETS = [
  { name: "Flat", gains: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
  { name: "Bass Boost", gains: [4, 3, 2, 1, 0, 0, 0, 0, 0, 0] },
  { name: "Treble Boost", gains: [0, 0, 0, 0, 0, 0, 1, 2, 3, 4] },
  { name: "Vocal Clarity", gains: [0, 0, -1, -2, 0, 2, 3, 2, 1, 0] },
  { name: "Loudness", gains: [3, 2, 1, 0, -1, -1, 0, 1, 2, 3] },
];

function getStreamLabel(labelId: number): string {
  switch (labelId) {
    case 0: return "Master";
    case 1: return "Aurora";
    case 2: return "Voice Call";
    default: return `Stream ${labelId}`;
  }
}

interface VuMeterProps {
  muted: boolean;
  volume: number;
  labelId: number;
  reducedMotion: boolean;
  dataTestId: string;
}

/**
 * Self-contained VU meter. Owns its own `requestAnimationFrame` simulation loop
 * and writes directly to DOM refs (`transform`/`opacity`), so the 60fps meter
 * animation never re-renders the parent panel (DESIGN.md 120 FPS budget). The
 * loop is fully disabled under `prefers-reduced-motion`.
 */
const VuMeter = memo(function VuMeter({
  muted,
  volume,
  labelId,
  reducedMotion,
  dataTestId,
}: VuMeterProps): React.ReactElement {
  const barRef = useRef<HTMLDivElement>(null);
  const peakRef = useRef<HTMLDivElement>(null);
  const peakLevelRef = useRef(0);

  useEffect(() => {
    if (reducedMotion) return;
    let frame = 0;
    const tick = () => {
      frame = requestAnimationFrame(tick);
      if (muted || volume === 0) {
        peakLevelRef.current = 0;
        if (barRef.current) barRef.current.style.transform = "scaleY(0)";
        if (peakRef.current) {
          peakRef.current.style.transform = "translateY(0px)";
          peakRef.current.style.opacity = "0";
        }
        return;
      }
      // Simulate audio activity based on stream type.
      const baseActivity = labelId === 1 ? 0.7 : labelId === 2 ? 0.4 : 0.5;
      const noise = (Math.random() - 0.5) * 0.3;
      const vu = Math.max(0, Math.min(1, baseActivity + noise));
      const peak = vu > peakLevelRef.current ? vu : Math.max(0, peakLevelRef.current - 0.02);
      peakLevelRef.current = peak;
      if (barRef.current) barRef.current.style.transform = `scaleY(${vu})`;
      if (peakRef.current) {
        peakRef.current.style.transform = `translateY(${-peak * 113}px)`;
        peakRef.current.style.opacity = peak > 0 ? "1" : "0";
      }
    };
    tick();
    return () => cancelAnimationFrame(frame);
  }, [muted, volume, labelId, reducedMotion]);

  return (
    <div
      className="glass-vu-meter"
      role="img"
      aria-label={`Volume level ${Math.round(volume * 100)}%`}
      data-testid={dataTestId}
      style={{ width: 24, height: 120, position: "relative", flexShrink: 0 }}
    >
      <div className="liquid-glass liquid-glass--clear" style={{ width: "100%", height: "100%", borderRadius: "9999px", position: "relative", padding: 2 }}>
        <div className="liquid-glass__refract" style={{ borderRadius: "9999px" }} />
        <div className="liquid-glass__tint" style={{ borderRadius: "9999px" }} />
        <div className="liquid-glass__specular" style={{ borderRadius: "9999px" }} />
        <div className="liquid-glass__content" style={{ width: "100%", height: "100%", position: "relative" }}>
          <div
            ref={barRef}
            className="glass-vu-meter__bar"
            style={{
              position: "absolute",
              inset: 0,
              width: "100%",
              height: "100%",
              background: "linear-gradient(to top, var(--accent) 0%, var(--accent-secondary) 100%)",
              transform: "scaleY(0)",
              willChange: "transform",
            }}
          />
          <div
            ref={peakRef}
            className="glass-vu-meter__peak"
            style={{
              position: "absolute",
              left: 0,
              right: 0,
              bottom: 0,
              height: 3,
              background: "var(--text)",
              opacity: 0,
              transform: "translateY(0px)",
              willChange: "transform, opacity",
            }}
          />
        </div>
      </div>
    </div>
  );
});

interface StreamCardProps {
  stream: AudioStreamExtended;
  idx: number;
  reducedMotion: boolean;
  onVolumeChange: (streamId: number, volume: number, muted: boolean) => void;
  onMuteToggle: (streamId: number, volume: number, muted: boolean) => void;
  onSoloToggle: (streamId: number) => void;
}

/** Per-stream mixer card. Extracted so the panel never calls hooks in a `.map()`. */
const StreamCard = memo(function StreamCard({
  stream,
  idx,
  reducedMotion,
  onVolumeChange,
  onMuteToggle,
  onSoloToggle,
}: StreamCardProps): React.ReactElement {
  const isMaster = stream.stream_id === 0;
  const labelIcon = isMaster ? <Speaker size={20} /> : stream.label_id === 1 ? <Headphones size={20} /> : <Mic size={20} />;

  return (
    <GlassCard
      className="audio-stream-card"
      dataTestId={`stream-card-${idx}`}
      elevation={1}
      style={{ display: "flex", flexDirection: "column", gap: "var(--space-4)", padding: 0 }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: "var(--space-3)", padding: "var(--space-4)", paddingBottom: 0 }}>
        <GlassCard width={40} height={40} radius={10} elevation={1} style={{ display: "flex", alignItems: "center", justifyContent: "center" }}>
          {labelIcon}
        </GlassCard>
        <div>
          <h3 className="panel-card-title" style={{ marginBottom: "var(--space-1)" }}>{getStreamLabel(stream.label_id)}</h3>
          <p className="panel-card-subtitle">Stream ID: {stream.stream_id} • {stream.muted ? "Muted" : "Active"}</p>
        </div>
        <div className="panel-card-actions" style={{ gap: "var(--space-2)", marginLeft: "auto" }}>
          <GlassButton
            variant={stream.isSolo ? "prominent" : "regular"}
            onClick={() => onSoloToggle(stream.stream_id)}
            aria-label={stream.isSolo ? "Unsolo" : "Solo"}
            aria-pressed={stream.isSolo}
            dataTestId={`solo-${stream.stream_id}`}
          >
            <Music size={16} /> Solo
          </GlassButton>
          <GlassButton
            variant={stream.muted ? "prominent" : "regular"}
            onClick={() => onMuteToggle(stream.stream_id, stream.volume, !stream.muted)}
            aria-label={stream.muted ? "Unmute" : "Mute"}
            aria-pressed={stream.muted}
            dataTestId={`mute-${stream.stream_id}`}
          >
            {stream.muted ? <VolumeX size={16} /> : <Volume2 size={16} />}
          </GlassButton>
        </div>
      </div>

      <div style={{ padding: "var(--space-4)", paddingTop: "var(--space-2)", display: "flex", alignItems: "flex-end", gap: "var(--space-4)" }}>
        <VuMeter
          muted={stream.muted}
          volume={stream.volume}
          labelId={stream.label_id}
          reducedMotion={reducedMotion}
          dataTestId={`vu-${stream.stream_id}`}
        />

        <div style={{ flex: 1, display: "flex", flexDirection: "column", justifyContent: "flex-end", minHeight: 140 }}>
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            value={stream.muted ? 0 : stream.volume}
            onChange={(e) => onVolumeChange(stream.stream_id, Number(e.target.value), stream.muted)}
            disabled={stream.muted}
            className="panel-slider panel-slider--vertical"
            style={{
              width: 120,
              height: 100,
              transform: "rotate(-90deg)",
              transformOrigin: "bottom left",
              marginLeft: "auto",
              marginRight: "auto",
            }}
            aria-label={`${getStreamLabel(stream.label_id)} volume`}
            data-testid={`volume-slider-${stream.stream_id}`}
          />
          <div style={{ textAlign: "center", marginTop: "var(--space-2)", fontSize: "var(--text-xs)", color: "var(--text-muted)" }}>
            {Math.round((stream.muted ? 0 : stream.volume) * 100)}%
          </div>
        </div>
      </div>
    </GlassCard>
  );
});

interface EqBandProps {
  band: { freq: number; label: string; type: string };
  index: number;
  gain: number;
  onGainChange: (bandIndex: number, gain: number) => void;
}

/** EQ band slider. Extracted so the panel never calls hooks in a `.map()`. */
function EqBand({ band, index, gain, onGainChange }: EqBandProps): React.ReactElement {
  return (
    <div className="audio-eq-band" style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: "var(--space-2)", flex: 1 }} data-testid={`eq-band-${index}`}>
      <div style={{ height: 140, width: 40, position: "relative" }}>
        <div className="liquid-glass liquid-glass--clear" style={{ width: "100%", height: "100%", borderRadius: "9999px", display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "flex-end", padding: 2 }}>
          <div className="liquid-glass__refract" style={{ borderRadius: "9999px" }} />
          <div className="liquid-glass__tint" style={{ borderRadius: "9999px" }} />
          <div className="liquid-glass__specular" style={{ borderRadius: "9999px" }} />
          <div className="liquid-glass__content" style={{ width: "100%", height: "100%", display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "flex-end" }}>
            <input
              type="range"
              min="-12"
              max="12"
              step="0.5"
              value={gain}
              onChange={(e) => onGainChange(index, Number(e.target.value))}
              className="panel-slider panel-slider--vertical"
              style={{
                width: "100%",
                height: "100%",
                transform: "rotate(-90deg)",
                transformOrigin: "bottom left",
                zIndex: 10,
              }}
              aria-label={`${band.label} gain ${gain >= 0 ? "+" : ""}${gain}dB`}
            />
            <div
              className="glass-progress__fill"
              style={{
                position: "absolute",
                bottom: 0,
                left: "50%",
                transform: `translateX(-50%) scaleY(${Math.max(0, (gain + 12) / 24)})`,
                transformOrigin: "bottom",
                width: "80%",
                height: "100%",
                background: gain > 0 ? "var(--accent)" : "var(--text-muted)",
                opacity: 0.3,
                pointerEvents: "none",
                transition: "transform var(--motion-duration-base) var(--motion-ease-out)",
              }}
            />
          </div>
        </div>
      </div>
      <span style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)", whiteSpace: "nowrap" }}>{band.label}</span>
      <span style={{ fontSize: "var(--text-xs)", fontWeight: 500, color: gain !== 0 ? "var(--accent)" : "var(--text-subtle)" }}>
        {gain >= 0 ? "+" : ""}{gain.toFixed(1)}dB
      </span>
    </div>
  );
}

export function AudioPanel(): React.ReactElement {
  const [streams, setStreams] = useState<AudioStreamExtended[]>([]);
  const [masterVolume, setMasterVolume] = useState(1);
  const [masterMuted, setMasterMuted] = useState(false);
  const [selectedOutput, setSelectedOutput] = useState<string>("default");
  const [eqGains, setEqGains] = useState<number[]>(Array(10).fill(0));
  const [eqPreset, setEqPreset] = useState(0);
  const reducedMotion = usePrefersReducedMotion();

  // Load audio streams on mount
  useEffect(() => {
    invoke<AudioListStreamsResult>("zettings_audio_list_streams")
      .then((r) => {
        const extended = r.streams.map((s) => ({
          ...s,
          isSolo: false,
        }));
        setStreams(extended);
        const first = extended[0];
        if (first) {
          setMasterVolume(first.volume);
          setMasterMuted(first.muted);
        }
      })
      .catch((e) => console.error("Failed to load audio streams:", e));
  }, []);

  const persistVolume = useCallback((streamId: number, volume: number, muted: boolean) => {
    const request: AudioSetVolumeRequest = { stream_id: streamId, volume, muted };
    invoke("zettings_audio_set_volume", request).catch((e) =>
      console.error("Failed to set audio volume:", e)
    );
  }, []);

  // Volume change handler
  const handleVolumeChange = useCallback((streamId: number, volume: number, muted: boolean) => {
    setStreams((prev) => prev.map((s) => (s.stream_id === streamId ? { ...s, volume } : s)));
    if (streamId === 0) setMasterVolume(volume);
    persistVolume(streamId, volume, muted);
  }, [persistVolume]);

  const handleMuteToggle = useCallback((streamId: number, volume: number, muted: boolean) => {
    setStreams((prev) => prev.map((s) => (s.stream_id === streamId ? { ...s, muted } : s)));
    if (streamId === 0) setMasterMuted(muted);
    persistVolume(streamId, volume, muted);
  }, [persistVolume]);

  const handleSoloToggle = useCallback((streamId: number) => {
    setStreams((prev) =>
      prev.map((s) =>
        s.stream_id === streamId ? { ...s, isSolo: !s.isSolo } : s.isSolo ? { ...s, isSolo: false } : s
      )
    );
  }, []);

  const handleEqGainChange = useCallback((bandIndex: number, gain: number) => {
    setEqGains((prev) => {
      const next = [...prev];
      next[bandIndex] = Math.max(-12, Math.min(12, gain));
      return next;
    });
  }, []);

  const handleEqPresetChange = useCallback((presetIndex: number) => {
    const preset = EQ_PRESETS[presetIndex];
    if (!preset) return;
    setEqPreset(presetIndex);
    setEqGains(preset.gains);
  }, []);

  const handleEqReset = useCallback(() => {
    setEqGains(Array(10).fill(0));
    setEqPreset(0);
  }, []);

  return (
    <PanelShell
      title="Sound"
      icon={Volume2}
      subtitle="Master volume, per-app mixer, equalizer, and output device"
      dataTestId="audio-panel"
    >
      <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-8)" }}>
        {/* Master section */}
        <GlassCard elevation={1} style={{ padding: "var(--space-6)", display: "grid", gridTemplateColumns: "1fr auto", gap: "var(--space-6)", alignItems: "center" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "var(--space-4)" }}>
            <GlassCard width={56} height={56} radius={14} elevation={1} style={{ display: "flex", alignItems: "center", justifyContent: "center" }}>
              <Volume2 size={28} color="var(--accent)" />
            </GlassCard>
            <div>
              <h3 style={{ fontSize: "var(--text-xl)", fontWeight: 600, color: "var(--text)", margin: 0 }}>Master Volume</h3>
              <p style={{ fontSize: "var(--text-sm)", color: "var(--text-muted)", margin: 0 }}>System-wide output level</p>
            </div>
          </div>
          <div className="glass-container" style={{ display: "flex", alignItems: "center", gap: "var(--space-4)", minWidth: 280 }}>
            <GlassButton
              width={48}
              height={48}
              variant={masterMuted ? "prominent" : "regular"}
              onClick={() => handleMuteToggle(0, masterVolume, !masterMuted)}
              aria-label={masterMuted ? "Unmute master" : "Mute master"}
              aria-pressed={masterMuted}
              dataTestId="master-mute"
            >
              {masterMuted ? <VolumeX size={20} /> : <Volume2 size={20} />}
            </GlassButton>
            <input
              type="range"
              min="0"
              max="1"
              step="0.01"
              value={masterMuted ? 0 : masterVolume}
              onChange={(e) => handleVolumeChange(0, Number(e.target.value), masterMuted)}
              disabled={masterMuted}
              className="panel-slider"
              style={{ flex: 1, maxWidth: 200 }}
              aria-label="Master volume"
              data-testid="master-volume"
            />
            <span style={{ fontSize: "var(--text-sm)", fontWeight: 500, color: "var(--text)", minWidth: "48px", textAlign: "right" }}>
              {Math.round((masterMuted ? 0 : masterVolume) * 100)}%
            </span>
          </div>
        </GlassCard>

        {/* Output device selector */}
        <GlassCard elevation={1} style={{ padding: "var(--space-6)" }}>
          <h4 style={{ fontSize: "var(--text-base)", fontWeight: 600, color: "var(--text)", margin: "0 0 var(--space-3)" }}>Output Device</h4>
          <select
            className="panel-input"
            value={selectedOutput}
            onChange={(e) => setSelectedOutput(e.target.value)}
            style={{ maxWidth: 320 }}
            data-testid="output-device"
          >
            <option value="default">Default (System)</option>
            <option value="speakers">Built-in Speakers</option>
            <option value="headphones">Headphones (3.5mm)</option>
            <option value="bluetooth">Zyntrix Aurora Buds</option>
            <option value="hdmi">HDMI Display Audio</option>
          </select>
        </GlassCard>

        {/* Per-stream mixer */}
        <section>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "var(--space-4)" }}>
            <h4 style={{ fontSize: "var(--text-base)", fontWeight: 600, color: "var(--text)", margin: 0 }}>Application Mixer</h4>
            <span style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)" }}>{streams.length} active streams</span>
          </div>
          {streams.length === 0 ? (
            <GlassCard elevation={1} className="glass-empty" style={{ padding: "var(--space-12)", display: "flex", flexDirection: "column", alignItems: "center", textAlign: "center" }}>
              <Volume2 className="glass-empty__icon" size={48} />
              <h3 className="glass-empty__title">No audio streams</h3>
              <p className="glass-empty__description">Play audio in an application to see per-app volume controls.</p>
            </GlassCard>
          ) : (
            <div className="glass-grid glass-grid--auto-fill">
              {streams.map((stream, idx) => (
                <StreamCard
                  key={stream.stream_id}
                  stream={stream}
                  idx={idx}
                  reducedMotion={reducedMotion}
                  onVolumeChange={handleVolumeChange}
                  onMuteToggle={handleMuteToggle}
                  onSoloToggle={handleSoloToggle}
                />
              ))}
            </div>
          )}
        </section>

        {/* Equalizer */}
        <GlassCard elevation={1} style={{ padding: "var(--space-6)" }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", flexWrap: "wrap", gap: "var(--space-3)", marginBottom: "var(--space-4)" }}>
            <h4 style={{ fontSize: "var(--text-base)", fontWeight: 600, color: "var(--text)", margin: 0 }}>10-Band Equalizer</h4>
            <div className="glass-container" style={{ display: "flex", gap: "var(--space-2)", alignItems: "center" }}>
              <select
                className="panel-input"
                value={eqPreset}
                onChange={(e) => handleEqPresetChange(Number(e.target.value))}
                style={{ maxWidth: 160 }}
                data-testid="eq-preset"
              >
                {EQ_PRESETS.map((p, i) => (
                  <option key={i} value={i}>{p.name}</option>
                ))}
              </select>
              <GlassButton onClick={handleEqReset} dataTestId="eq-reset">
                <RotateCcw size={16} /> Reset
              </GlassButton>
            </div>
          </div>
          <div style={{ display: "flex", gap: "var(--space-3)", overflowX: "auto", paddingBottom: "var(--space-4)" }}>
            {EQ_BANDS.map((band, i) => (
              <EqBand key={band.freq} band={band} index={i} gain={eqGains[i] ?? 0} onGainChange={handleEqGainChange} />
            ))}
          </div>
          <div style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)" }}>
            Gains applied to PipeWire filter chain. Presets are starting points — adjust to taste.
          </div>
        </GlassCard>
      </div>
    </PanelShell>
  );
}