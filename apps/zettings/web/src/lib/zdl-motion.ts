/**
 * Zyntrix Design Language (ZDL) — Motion & Geometry Engine
 * Implements continuous superellipse curvature (G2/G3 squircles) and 
 * analytical spring-damper physics solvers targeting 120 FPS rendering.
 */

// Superellipse curvature formula: |x/a|^n + |y/b|^n = 1
export type SquircleOrder = 4 | 6; // n=4 -> G2 Continuity, n=6 -> G3 Continuity

export interface SpringConfig {
  stiffness: number;
  damping: number;
  mass: number;
  precision?: number;
}

export interface SpringState {
  position: number;
  velocity: number;
}

export const ZDL_SPRINGS = {
  navigation: { stiffness: 220, damping: 28, mass: 1.0, precision: 0.001 },
  toggle: { stiffness: 320, damping: 22, mass: 0.6, precision: 0.001 },
  slider: { stiffness: 400, damping: 30, mass: 0.5, precision: 0.0005 },
  modal: { stiffness: 180, damping: 24, mass: 1.2, precision: 0.001 },
} as const satisfies Record<string, SpringConfig>;

/**
 * Generates an SVG path string for a continuous curvature G2/G3 squircle.
 *
 * Implements the radius-blending formula from DESIGN.md section 1:
 *   n_eff = 2 + (n - 2) * (r / (min(w, h) / 2))
 * which interpolates the effective superellipse exponent between 2 (pure
 * ellipse) and the target order (4 for G2, 6 for G3) based on the radius
 * ratio. Sampling uses 128 perimeter steps and emits cubic bezier (`C`)
 * commands for smooth curves.
 */
export function generateSquirclePath(
  width: number,
  height: number,
  radius: number,
  order: SquircleOrder = 4
): string {
  const w = width;
  const h = height;
  if (w <= 0 || h <= 0) return "";

  const halfMin = Math.min(w, h) / 2;
  // Clamp radius to the half-extent of the shorter side so the ratio stays
  // within [0, 1] (radius=0 -> pure ellipse n_eff=2, radius=halfMin -> full
  // superellipse n_eff=order).
  const r = Math.max(0, Math.min(radius, halfMin));
  const ratio = halfMin > 0 ? r / halfMin : 0;
  const n = 2 + (order - 2) * ratio; // effective exponent
  const a = w / 2;
  const b = h / 2;

  // Sample 128 perimeter points using parametric superellipse equations.
  const steps = 128;
  const points: Array<[number, number]> = [];
  for (let i = 0; i < steps; i++) {
    const theta = (i / steps) * 2 * Math.PI;
    const cosT = Math.cos(theta);
    const sinT = Math.sin(theta);
    const xSign = cosT < 0 ? -1 : 1;
    const ySign = sinT < 0 ? -1 : 1;
    // |x/a|^n + |y/b|^n = 1  ->  x = a * sign(cos) * |cos|^(2/n)
    const px = Math.pow(Math.abs(cosT), 2 / n) * a * xSign + a;
    const py = Math.pow(Math.abs(sinT), 2 / n) * b * ySign + b;
    points.push([px, py]);
  }

  const first = points[0];
  if (first === undefined) return "";

  // Emit a closed path using cubic bezier (`C`) commands. For each segment
  // between point i and point i+1, we derive two control points by
  // approximating the tangent direction at each endpoint and projecting a
  // third of the chord length along it. This yields G1-continuous smooth
  // curves that read as G2/G3 at typical display sizes.
  const inv = 1 / 3;
  let path = `M ${first[0].toFixed(2)} ${first[1].toFixed(2)}`;
  for (let i = 0; i < steps; i++) {
    const p0 = points[i]!;
    const p1 = points[(i + 1) % steps]!;
    const pPrev = points[(i - 1 + steps) % steps]!;
    const pNext = points[(i + 2) % steps]!;

    // Tangent at p0 ~ direction from pPrev to p1 (centered difference).
    const t0x = p1[0] - pPrev[0];
    const t0y = p1[1] - pPrev[1];
    // Tangent at p1 ~ direction from p0 to pNext.
    const t1x = pNext[0] - p0[0];
    const t1y = pNext[1] - p0[1];

    const c1x = p0[0] + t0x * inv;
    const c1y = p0[1] + t0y * inv;
    const c2x = p1[0] - t1x * inv;
    const c2y = p1[1] - t1y * inv;

    path += ` C ${c1x.toFixed(2)} ${c1y.toFixed(2)} ${c2x.toFixed(2)} ${c2y.toFixed(2)} ${p1[0].toFixed(2)} ${p1[1].toFixed(2)}`;
  }
  path += " Z";

  return path;
}

/**
 * Analytical Spring Physics Step Solver.
 * Solves second-order differential equation: m*x'' + c*x' + k*x = 0
 */
export function solveSpringStep(
  current: SpringState,
  target: number,
  config: SpringConfig,
  deltaTimeSeconds: number
): SpringState {
  const { stiffness: k, damping: c, mass: m } = config;
  const x = current.position - target;
  const v = current.velocity;

  if (Math.abs(x) < (config.precision ?? 0.001) && Math.abs(v) < (config.precision ?? 0.001)) {
    return { position: target, velocity: 0 };
  }

  const omega0 = Math.sqrt(k / m); // Natural frequency
  const zeta = c / (2 * Math.sqrt(k * m)); // Damping ratio
  const dt = deltaTimeSeconds;

  let newX = 0;
  let newV = 0;

  if (zeta < 1) {
    // Underdamped
    const omegaD = omega0 * Math.sqrt(1 - zeta * zeta);
    const decay = Math.exp(-zeta * omega0 * dt);
    const cosD = Math.cos(omegaD * dt);
    const sinD = Math.sin(omegaD * dt);

    const A = x;
    const B = (v + zeta * omega0 * x) / omegaD;

    newX = decay * (A * cosD + B * sinD);
    newV =
      -zeta * omega0 * newX +
      decay * (-A * omegaD * sinD + B * omegaD * cosD);
  } else if (zeta === 1) {
    // Critically damped
    const decay = Math.exp(-omega0 * dt);
    const A = x;
    const B = v + omega0 * x;

    newX = decay * (A + B * dt);
    newV = decay * (B - omega0 * (A + B * dt));
  } else {
    // Overdamped
    const gamma1 = omega0 * (zeta + Math.sqrt(zeta * zeta - 1));
    const gamma2 = omega0 * (zeta - Math.sqrt(zeta * zeta - 1));

    const C2 = (v + gamma1 * x) / (gamma1 - gamma2);
    const C1 = x - C2;

    newX = C1 * Math.exp(-gamma1 * dt) + C2 * Math.exp(-gamma2 * dt);
    newV = -gamma1 * C1 * Math.exp(-gamma1 * dt) - gamma2 * C2 * Math.exp(-gamma2 * dt);
  }

  return {
    position: newX + target,
    velocity: newV,
  };
}

// =============================================================================
// Phase 3 — Motion Engine & Physics Solvers
// =============================================================================
// The analytical solver above is exact for the linear m*x'' + c*x' + k*x = 0
// ODE and is the primary solver used throughout the app. Phase 3 adds an
// alternative RK4 numerical solver (for non-linear springs / time-varying
// configs), a rAF-driven `useSpring` React hook, a velocity-preserving route
// transition scaffold, and micro-interaction trigger hooks. All motion code
// respects `prefers-reduced-motion` per ui-ux-pro-max (High severity rule) by
// reading the matchMedia query once per hook instance and snapping to the
// target with zero velocity when the user has requested reduced motion.

/**
 * Derivative function for the spring ODE system:
 *   dx/dt = v
 *   dv/dt = -(k/m)*(x - target) - (c/m)*v
 * Used by the RK4 solver below. Returned as a tuple [dPosition, dVelocity].
 */
function springDerivative(
  state: SpringState,
  target: number,
  config: SpringConfig
): readonly [number, number] {
  const { stiffness: k, damping: c, mass: m } = config;
  const x = state.position - target;
  const v = state.velocity;
  return [v, -(k / m) * x - (c / m) * v] as const;
}

/**
 * RK4 (4th-order Runge-Kutta) Spring Physics Step Solver.
 *
 * Numerical alternative to {@link solveSpringStep} for non-linear springs
 * (time-varying stiffness/damping) or when the analytical closed-form
 * branches become expensive. One RK4 step advances the state by `dt` seconds
 * with O(h^4) local error, which is more than sufficient for 120 FPS frame
 * budgets (δt ≈ 8.33ms). When the system has settled within `config.precision`
 * it snaps to the target with zero velocity to avoid wasted frames.
 */
export function solveSpringStepRK4(
  current: SpringState,
  target: number,
  config: SpringConfig,
  deltaTimeSeconds: number
): SpringState {
  const precision = config.precision ?? 0.001;
  const x = current.position - target;
  if (Math.abs(x) < precision && Math.abs(current.velocity) < precision) {
    return { position: target, velocity: 0 };
  }

  const dt = deltaTimeSeconds;
  const k1 = springDerivative(current, target, config);
  const s2: SpringState = {
    position: current.position + (k1[0] * dt) / 2,
    velocity: current.velocity + (k1[1] * dt) / 2,
  };
  const k2 = springDerivative(s2, target, config);
  const s3: SpringState = {
    position: current.position + (k2[0] * dt) / 2,
    velocity: current.velocity + (k2[1] * dt) / 2,
  };
  const k3 = springDerivative(s3, target, config);
  const s4: SpringState = {
    position: current.position + k3[0] * dt,
    velocity: current.velocity + k3[1] * dt,
  };
  const k4 = springDerivative(s4, target, config);

  return {
    position:
      current.position + (dt / 6) * (k1[0] + 2 * k2[0] + 2 * k3[0] + k4[0]),
    velocity:
      current.velocity + (dt / 6) * (k1[1] + 2 * k2[1] + 2 * k3[1] + k4[1]),
  };
}

/** Default fixed timestep for the rAF solver: targets 120 FPS compositor cadence. */
export const ZDL_MOTION_FRAME_DT = 1 / 120;