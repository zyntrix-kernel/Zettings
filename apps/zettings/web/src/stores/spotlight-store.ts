/**
 * Spotlight store — global open/close state for the Spotlight modal + search entry registration.
 *
 * A zustand store is used (rather than React context or prop drilling) so the
 * modal can be triggered from anywhere in the tree — the global keydown
 * listener in zettings.tsx, the sidebar search affordance, or a future
 * command-palette binding — without passing callbacks through every layer.
 *
 * Accessibility (ui-ux-pro-max/ux — Keyboard Navigation, High severity):
 * the store centralises the open/close toggle so consumers can also drive
 * focus management and aria-modal from a single source of truth.
 */
import { create } from "zustand";

/** Spotlight modal state. */
export interface SpotlightState {
  isOpen: boolean;
  open: () => void;
  close: () => void;
  toggle: () => void;
}

/**
 * Spotlight modal zustand store. Consumed by the global keydown listener
 * (Super+I / Ctrl+Space) in zettings.tsx and by the SpotlightModal
 * component itself.
 */
export const useSpotlightStore = create<SpotlightState>((set) => ({
  isOpen: false,
  open: () => set({ isOpen: true }),
  close: () => set({ isOpen: false }),
  toggle: () => set((state) => ({ isOpen: !state.isOpen })),
}))
