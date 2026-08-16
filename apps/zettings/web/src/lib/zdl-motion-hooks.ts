/**
 * ZDL Motion Engine — React hooks for spring-driven animation.
 *
 * Phase 3 scaffolding. These hooks wrap the pure solvers in `zdl-motion.ts`
 * with `requestAnimationFrame`-driven re-computation and `React.useState`-
 * backed state (per ui-ux-pro-max React guidance: custom hooks named with the
 * `use` prefix, simple hooks prefer `useState`, no infinite decorative
 * animations). All motion respects `prefers-reduced-motion` (High severity
 * accessibility rule): when the user has reduced motion enabled, springs
 * snap to their target with zero velocity instead of animating.
 */
import { useCallback, useEffect, useRef, useState } from "react";
import {
  type SpringConfig,
  type SpringState,
  ZDL_MOTION_FRAME_DT,
  ZDL_SPRINGS,
  solveSpringStep,
} from "./zdl-motion.js";

export { ZDL_SPRINGS, type SpringConfig, type SpringState } from "./zdl-motion.js";

/**
 * Reads `prefers-reduced-motion: reduce` once per hook instance. Used by all
 * motion hooks to short-circuit animation per the ui-ux-pro-max High-severity
 * "Reduced Motion" rule.
 */
function usePrefersReducedMotion(): boolean {
  const [reduced, setReduced] = useState<boolean>(() => {
    if (typeof window === "undefined" || !window.matchMedia) return false;
    return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  });
  useEffect(() => {
    if (typeof window === "undefined" || !window.matchMedia) return;
    const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
    const handler = (e: MediaQueryListEvent) => setReduced(e.matches);
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, []);
  return reduced;
}

/**
 * Spring-animated scalar value. Drives `requestAnimationFrame` to integrate
 * the analytical spring solver toward `target`, exposing `{ value, velocity }`
 * so consumers can chain springs or react to arrival. When the system settles
 * within `config.precision` the rAF loop halts (no infinite decorative
 * animation per ui-ux-pro-max Medium-severity "Continuous Animation" rule).
 *
 * @param target    Desired rest position.
 * @param config    Spring configuration (`ZDL_SPRINGS.*` presets are typical).
 * @param initial   Initial state, defaulting to `{ position: target, velocity: 0 }`.
 */
export function useSpring(
  target: number,
  config: SpringConfig,
  initial?: SpringState
): SpringState {
  const reducedMotion = usePrefersReducedMotion();
  const [state, setState] = useState<SpringState>(
    () => initial ?? { position: target, velocity: 0 }
  );
  const targetRef = useRef<number>(target);
  const configRef = useRef<SpringConfig>(config);
  const stateRef = useRef<SpringState>(state);
  const rafRef = useRef<number | null>(null);
  const lastTimeRef = useRef<number | null>(null);

  // Keep refs in sync so the rAF callback reads fresh values without
  // re-subscribing on every target/config change.
  targetRef.current = target;
  configRef.current = config;
  stateRef.current = state;

  const tick = useCallback(
    (now: number) => {
      const last = lastTimeRef.current;
      // Cap δt at 4 frames to avoid huge jumps after tab throttling.
      const dt = last === null ? ZDL_MOTION_FRAME_DT : Math.min((now - last) / 1000, ZDL_MOTION_FRAME_DT * 4);
      lastTimeRef.current = now;

      const next = solveSpringStep(
        stateRef.current,
        targetRef.current,
        configRef.current,
        dt
      );
      stateRef.current = next;
      setState(next);

      const precision = configRef.current.precision ?? 0.001;
      const settled =
        Math.abs(next.position - targetRef.current) < precision &&
        Math.abs(next.velocity) < precision;
      if (!settled) {
        rafRef.current = requestAnimationFrame(tick);
      } else {
        rafRef.current = null;
      }
    },
    []
  );

  useEffect(() => {
    if (reducedMotion) {
      // Snap to target with zero velocity per the Reduced Motion rule.
      setState({ position: target, velocity: 0 });
      stateRef.current = { position: target, velocity: 0 };
      return;
    }
    lastTimeRef.current = null;
    rafRef.current = requestAnimationFrame(tick);
    return () => {
      if (rafRef.current !== null) cancelAnimationFrame(rafRef.current);
      rafRef.current = null;
    };
  }, [target, config, reducedMotion, tick]);

  return state;
}

/**
 * Spring-driven boolean toggle hook intended for switch / check components.
 * Animates between 0 (off) and 1 (on) using the `toggle` spring preset,
 * returning the continuous interpolated value so the caller can drive
 * clip-path, opacity, translate, or other continuous properties.
 */
export function useToggleSpring(active: boolean): number {
  const state = useSpring(active ? 1 : 0, ZDL_SPRINGS.toggle);
  return state.position;
}

/**
 * Spring-driven slider-thumb hook. Tracks a continuous 0..1 ratio that callers
 * can map to a pixel offset. The `slider` preset is stiffer and lightly
 * damped so thumbs track the pointer without overshoot.
 */
export function useSliderSpring(ratio: number): number {
  const clamped = Math.max(0, Math.min(1, ratio));
  const state = useSpring(clamped, ZDL_SPRINGS.slider);
  return state.position;
}

/**
 * Spring-driven modal enter/exit hook. Returns a 0..1 progress value that
 * callers can use for backdrop opacity, content translate, or scale. When the
 * modal is closed (`open === false`) the spring springs back to 0, allowing
 * the same hook to drive both enter and exit transitions (exit-faster-than-enter
 * is handled by the asymmetric spring preset, not a second hook).
 */
export function useModalSpring(open: boolean): number {
  const state = useSpring(open ? 1 : 0, ZDL_SPRINGS.modal);
  return state.position;
}

/**
 * Velocity-preserving route transition scaffold.
 *
 * Phase 3 scaffold (skeleton only): maintains a shared spring state across
 * route changes so the outgoing page's exit velocity seeds the incoming
 * page's enter motion (continuous-curvature transitions, per PLAN.md Phase 3
 * "continuous curvature transitions across tab changes"). The present scaffold
 * returns the current progress plus a setter to feed a new velocity when a
 * route change is dispatched; wiring to a real router is deferred so the
 * Phase 3 design can be reviewed before integration.
 */
export interface RouteTransitionState {
  /** 0 = rest at old route, 1 = rest at new route. Spring drives this. */
  progress: number;
  /** Live spring velocity — passed forward to preserve continuity. */
  velocity: number;
  /** Function invoked on route change to seed the enter transition. */
  startEnter: (incomingVelocity?: number) => void;
}

export function useRouteTransitionSpring(): RouteTransitionState {
  const reducedMotion = usePrefersReducedMotion();
  const [progress, setProgress] = useState<number>(0);
  const velocityRef = useRef<number>(0);
  const targetRef = useRef<number>(0);
  const rafRef = useRef<number | null>(null);
  const lastTimeRef = useRef<number | null>(null);
  const stateRef = useRef<SpringState>({ position: 0, velocity: 0 });

  const tick = useCallback((now: number) => {
    const last = lastTimeRef.current;
    const dt = last === null ? ZDL_MOTION_FRAME_DT : Math.min((now - last) / 1000, ZDL_MOTION_FRAME_DT * 4);
    lastTimeRef.current = now;
    const next = solveSpringStep(stateRef.current, targetRef.current, ZDL_SPRINGS.navigation, dt);
    stateRef.current = next;
    velocityRef.current = next.velocity;
    setProgress(next.position);
    const precision = ZDL_SPRINGS.navigation.precision ?? 0.001;
    const settled = Math.abs(next.position - targetRef.current) < precision && Math.abs(next.velocity) < precision;
    if (!settled) {
      rafRef.current = requestAnimationFrame(tick);
    } else {
      rafRef.current = null;
    }
  }, []);

  const startEnter = useCallback((incomingVelocity?: number) => {
    if (reducedMotion) {
      // Reduced-motion: snap to settled enter state.
      stateRef.current = { position: 1, velocity: 0 };
      velocityRef.current = 0;
      targetRef.current = 1;
      setProgress(1);
      return;
    }
    targetRef.current = 1;
    // Seed velocity from the outgoing transition to preserve continuity.
    stateRef.current = {
      position: stateRef.current.position,
      velocity: incomingVelocity ?? velocityRef.current,
    };
    lastTimeRef.current = null;
    if (rafRef.current === null) {
      rafRef.current = requestAnimationFrame(tick);
    }
  }, [reducedMotion, tick]);

  useEffect(() => () => {
    if (rafRef.current !== null) cancelAnimationFrame(rafRef.current);
    rafRef.current = null;
  }, [tick]);

  return {
    progress,
    velocity: velocityRef.current,
    startEnter,
  };
}
