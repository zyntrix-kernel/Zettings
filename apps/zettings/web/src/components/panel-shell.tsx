/**
 * PanelShell — shared outer wrapper for all settings feature panels.
 *
 * Provides consistent layout, header, and GlassPanel material for all six
 * Phase 7 domain panels. Uses the ZDL GlassPanel with elevation 2 and G2
 * continuity (order=4) as the default panel chrome. Panels can override
 * elevation/order for specialized visual treatment.
 *
 * Accessibility: exposes `aria-labelledby` for the panel title. Header icon
 * and title are required for proper labeling (ui-ux-pro-max High:
 * "Accessibility/ARIA Labels" — icon-only buttons labeled).
 */
import { type CSSProperties, type ReactNode, type FC } from "react";
import { GlassPanel, type GlassElevation } from "./glass-panel.js";
import { type SquircleOrder } from "../lib/zdl-motion.js";
import type { LucideProps } from "lucide-react";

export interface PanelShellProps {
  /** Panel title (shown in header). */
  title: string;
  /** Lucide icon component for the header (e.g., Monitor, Volume2, Wifi, etc.). */
  icon: FC<LucideProps>;
  /** Optional subtitle/description below the title. */
  subtitle?: string;
  /** Optional action buttons rendered in the header (e.g., "Add Monitor", "Scan"). */
  actions?: ReactNode;
  /** Glass elevation (1-4). Defaults to 2. */
  elevation?: GlassElevation;
  /** Squircle order: 4 (G2) or 6 (G3). Defaults to 4. */
  order?: SquircleOrder;
  /** Optional corner radius. Defaults to 20. */
  radius?: number;
  /** Optional explicit width/height for the GlassPanel. */
  width?: number;
  height?: number;
  /** Optional className for the GlassPanel. */
  className?: string;
  /** Optional inline style for the GlassPanel. */
  style?: CSSProperties;
  /** Panel content children. */
  children?: ReactNode;
  /** Optional test id for e2e targeting. */
  dataTestId?: string;
}

export const PanelShell = ({
  title,
  icon: Icon,
  subtitle,
  actions,
  elevation = 2,
  order = 4,
  radius = 20,
  width,
  height,
  className,
  style,
  children,
  dataTestId,
}: PanelShellProps): React.ReactElement => {
  // exactOptionalPropertyTypes: only spread defined values
  const glassProps: Record<string, unknown> = {
    elevation,
    order,
    radius,
    className,
    style,
  };
  if (width !== undefined) glassProps.width = width;
  if (height !== undefined) glassProps.height = height;

  return (
    <GlassPanel {...glassProps as import("./glass-panel.js").GlassPanelProps}>
      <header className="panel-header" data-testid={dataTestId ? `${dataTestId}-header` : undefined}>
        <div className="panel-header-icon" aria-hidden="true">
          <Icon className="panel-header-icon-svg" size={24} strokeWidth={2} />
        </div>
        <div className="panel-header-text">
          <h2 className="panel-title">{title}</h2>
          {subtitle && <p className="panel-subtitle">{subtitle}</p>}
        </div>
        {actions && <div className="panel-header-actions">{actions}</div>}
      </header>
      <div className="panel-content" role="region" aria-labelledby={dataTestId ? `${dataTestId}-title` : undefined}>
        {children}
      </div>
    </GlassPanel>
  );
};