/**
 * Reduced-motion policy (DESIGN.md §10): when the user prefers reduced
 * motion, every transform-based transition collapses to an opacity-only
 * cross-fade ≤ 120 ms; parallax/secondary motion is disabled entirely.
 */
import { useReducedMotion } from "motion/react";
import type { TargetAndTransition, Transition } from "motion/react";
import {
  EXPAND_TRANSITION,
  MODAL_TRANSITION,
  PAGE_TRANSITION,
  PRESS_FEEDBACK,
} from "./tokens";

/** Named transition bundles a component tree may consume. */
export interface PageTransitionSet {
  initial: TargetAndTransition;
  animate: TargetAndTransition;
  exit: TargetAndTransition;
  transition: Transition;
}

export interface PressFeedbackSet {
  whileTap?: TargetAndTransition;
  whileHover?: TargetAndTransition;
  transition?: Transition;
}

/** The active motion policy for the current render. */
export interface MotionPolicy {
  /** True when transforms must be replaced by opacity-only fades. */
  reduced: boolean;
  page: PageTransitionSet;
  expand: Transition;
  modal: PageTransitionSet;
  press: PressFeedbackSet;
}

const FADE_ONLY: PageTransitionSet = {
  initial: { opacity: 0 },
  animate: { opacity: 1 },
  exit: { opacity: 0 },
  transition: { duration: 0.12 },
};

const NO_PRESS: PressFeedbackSet = {};

const FULL_POLICY: MotionPolicy = {
  reduced: false,
  page: PAGE_TRANSITION,
  expand: EXPAND_TRANSITION,
  modal: MODAL_TRANSITION,
  press: PRESS_FEEDBACK,
};

const REDUCED_POLICY: MotionPolicy = {
  reduced: true,
  page: FADE_ONLY,
  expand: { duration: 0.12 },
  modal: FADE_ONLY,
  press: NO_PRESS,
};

/**
 * React hook returning the policy-aware transition set for a component tree.
 */
export function useMotionPolicy(): MotionPolicy {
  const reduced = useReducedMotion() ?? false;
  return reduced ? REDUCED_POLICY : FULL_POLICY;
}
