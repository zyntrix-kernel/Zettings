/**
 * ZDL motion engine public surface. Components import from `lib/motion`
 * only; physics values live in tokens.ts and are never hand-tuned inline.
 */
export {
  SPRING_NAVIGATION,
  SPRING_CONTROL,
  SPRING_MODAL,
  EASE_STANDARD,
  DURATION,
  PAGE_TRANSITION,
  EXPAND_TRANSITION,
  MODAL_TRANSITION,
  PRESS_FEEDBACK,
} from "./tokens";
export { useMotionPolicy } from "./policy";
export type { MotionPolicy } from "./policy";
export { FrameMonitor } from "./frame-monitor";
export type { FrameStats } from "./frame-monitor";
