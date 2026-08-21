/**
 * PageHeader — L2 page title block (Win11 spec §5.1/§6.1): ~28px semibold
 * title with optional muted subtitle, rendered OUTSIDE any glass panel so
 * content typography stays primary (liquid-glass-foundations).
 */
import { useId, type ReactNode } from "react";

export interface PageHeaderProps {
  /** Page title. */
  title: string;
  /** Optional supporting line under the title. */
  subtitle?: string | undefined;
}

export function PageHeader({ title, subtitle }: PageHeaderProps): React.ReactElement {
  return (
    <header className="page-header">
      <h1 className="page-title">{title}</h1>
      {subtitle ? <p className="page-subtitle">{subtitle}</p> : null}
    </header>
  );
}

export interface SettingsSectionProps {
  /** Section heading (L3). */
  title: string;
  /** Cards / expanders / fields in this group. */
  children: ReactNode;
}

/** SettingsSection — L3 group: named region + vertically stacked cards. */
export function SettingsSection({ title, children }: SettingsSectionProps): React.ReactElement {
  const headingId = useId();
  return (
    <section className="settings-section" aria-labelledby={headingId}>
      <h2 className="settings-section-header" id={headingId}>
        {title}
      </h2>
      <div className="settings-stack">{children}</div>
    </section>
  );
}
