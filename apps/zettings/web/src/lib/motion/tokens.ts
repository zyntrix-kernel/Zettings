/**
 * ZDL motion tokens (DESIGN.md §10) expressed as Motion (framer-motion
 * successor) values. Single source of truth for physics; components never
 * hand-tune springs.
 *
 * Authority split: HOW to animate = motion-framer / react-spring-physics
 * skills; WHAT values = this file + DESIGN.md.
 */

/** Spring for navigation-level transitions (page slides, rail morphs). */
export const SPRING_NAVIGATION = { type: "spring", stiffness: 220, damping: 28, mass: 1.0 } as const;

/** Spring for control feedback (toggles, knobs, press returns). */
export const SPRING_CONTROL = { type: "spring", stiffness: 320, damping: 22, mass: 0.6 } as const;

/** Spring for modal/dialog presentation. */
export const SPRING_MODAL = { type: "spring", stiffness: 180, damping: 24, mass: 1.2 } as const;

/** Standard decelerate curve for non-spring tweens. */
export const EASE_STANDARD = [0.2, 0, 0, 1] as const;

/** Token durations in seconds (Motion API unit). */
export const DURATION = {
  instant: 0.08,
  quick: 0.14,
  normal: 0.22,
  deliberate: 0.32,
} as const;

/** Enter/exit pair for L2 page swaps: directional slide + fade. */
export const PAGE_TRANSITION = {
  initial: { opacity: 0, x: 24 },
  animate: { opacity: 1, x: 0 },
  exit: { opacity: 0, x: -16 },
  transition: { ...SPRING_NAVIGATION, opacity: { duration: DURATION.quick, ease: EASE_STANDARD } },
} as const;

/** Expansion reveal used by SettingsExpander regions. */
export const EXPAND_TRANSITION = { duration: DURATION.normal, ease: EASE_STANDARD } as const;

/** Modal/flyout presentation on the G3 overlay surface. */
export const MODAL_TRANSITION = {
  initial: { opacity: 0, scale: 0.96, y: 8 },
  animate: { opacity: 1, scale: 1, y: 0 },
  exit: { opacity: 0, scale: 0.98, y: 4 },
  transition: { ...SPRING_MODAL, opacity: { duration: DURATION.quick, ease: EASE_STANDARD } },
} as const;

/** Press feedback: compress then spring back; overshoot capped ≤4%. */
export const PRESS_FEEDBACK = {
  whileTap: { scale: 0.98 },
  whileHover: { scale: 1.004 },
  transition: SPRING_CONTROL,
} as const;
