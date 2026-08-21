/**
 * Status-state primitives (spec §15, aria-live-regions skill).
 *
 * Loading → `role="status"`; errors → `role="alert"` (important condition);
 * empty/unavailable states render honest explanations with an action when
 * one exists. No fixed-delay announcement tricks anywhere.
 */
import type { ReactNode } from "react";

/** Advisory loading indicator (polite). */
export function Loading({ label }: { label: string }) {
  return (
    <p role="status" className="zdl-status">
      {label}
    </p>
  );
}

export interface ErrorBarProps {
  title: string;
  detail?: string;
  /** Retry action; rendered as a real button when provided. */
  onRetry?: () => void;
}

/** Important failure state (assertive semantics via role=alert). */
export function ErrorBar({ title, detail, onRetry }: ErrorBarProps) {
  return (
    <div role="alert" className="zdl-infobar" data-variant="error">
      <div className="zdl-infobar__body">
        <p className="zdl-card__title">{title}</p>
        {detail !== undefined && (
          <p className="zdl-card__description">{detail}</p>
        )}
      </div>
      {onRetry !== undefined && (
        <button type="button" className="zdl-button" onClick={onRetry}>
          Try again
        </button>
      )}
    </div>
  );
}

export interface EmptyStateProps {
  title: string;
  /** Why this surface is empty (honest unavailable states, spec §15). */
  explanation: string;
  action?: ReactNode;
}

/** Honest empty/unavailable state. */
export function EmptyState({ title, explanation, action }: EmptyStateProps) {
  return (
    <div className="zdl-empty">
      <p className="zdl-card__title">{title}</p>
      <p className="zdl-card__description">{explanation}</p>
      {action}
    </div>
  );
}

export interface InfoBarProps {
  /** InfoBar tone; never color-only — icon+text always accompany. */
  variant?: "info" | "success";
  children: ReactNode;
}

/** Inline informational banner (Windows-spec InfoBar analog). */
export function InfoBar({ variant = "info", children }: InfoBarProps) {
  return (
    <div className="zdl-infobar" data-variant={variant} role="status">
      <div className="zdl-infobar__body">{children}</div>
    </div>
  );
}
