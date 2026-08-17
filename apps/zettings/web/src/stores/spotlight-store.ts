/**
 * Spotlight store — global open/close state for the Spotlight modal.
 *
 * A zustand store is used (rather than React context or prop drilling) so the
 * modal can be triggered from anywhere in the tree — the global keydown
 * listener in `zettings.tsx`, the sidebar search affordance, or a future
 * command-palette binding — without passing callbacks through every layer.
 *
 * Accessibility (ui-ux-pro-max/ux — Keyboard Navigation, High severity):
 * the store centralises the open/close toggle so consumers can also drive
 * focus management and aria-modal from a single source of truth.
 */
import { create } from "zustand";

/** Spotlight modal open/close state. */
export interface SpotlightState {
  /** Whether the Spotlight modal is currently mounted / animated in. */
  isOpen: boolean;
  /** Open the Spotlight modal. Idempotent if already open. */
  open: () => void;
  /** Close the Spotlight modal. Idempotent if already closed. */
  close: () => void;
  /** Toggle the Spotlight modal open state. */
  toggle: () => void;
}

/**
 * Spotlight modal zustand store. Consumed by the global keydown listener
 * (`Super+I` / `Ctrl+Space`) in `zettings.tsx` and by the SpotlightModal
 * component itself.
 */
export const useSpotlightStore = create<SpotlightState>((set) => ({
  isOpen: false,
  open: () => set({ isOpen: true }),
  close: () => set({ isOpen: false }),
  toggle: () => set((state) => ({ isOpen: !state.isOpen })),
}));
