import { describe, expect, it } from 'vitest'
import {
  FPS_BUDGET_MS,
  summarizeFrames,
  type FrameSample,
} from './lib/perf'

function sample(timestamp: number, frameMs: number): FrameSample {
  return { timestamp, frameMs }
}

describe('summarizeFrames', () => {
  it('reports zero stats for an empty window', () => {
    const stats = summarizeFrames([], 1000, 1000)
    expect(stats.frameCount).toBe(0)
    expect(stats.fps).toBe(0)
    expect(stats.droppedFrames).toBe(0)
    expect(stats.p50FrameMs).toBe(0)
    expect(stats.p95FrameMs).toBe(0)
  })

  it('prunes samples older than the window', () => {
    const samples = [
      sample(-1, 16.7), // expired (before cutoff 0)
      sample(1000, 8.0), // exactly at the window edge -> kept
      sample(1050, 16.7), // over budget
    ]
    const stats = summarizeFrames(samples, 1000, 1000)
    expect(stats.frameCount).toBe(2)
    expect(stats.droppedFrames).toBe(1)
  })

  it('computes p50/p95 over the sorted window', () => {
    const samples = [10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29].map(
      (ms, i) => sample(i * 10, ms),
    )
    const stats = summarizeFrames(samples, 190, 1000)
    expect(stats.frameCount).toBe(20)
    expect(stats.p50FrameMs).toBe(19) // ceil(0.5*20)-1 = index 9 -> 19
    expect(stats.p95FrameMs).toBe(28) // ceil(0.95*20)-1 = index 18 -> 28
  })

  it('flags withinBudget only when p95 stays under the 120 FPS budget', () => {
    const under = [7, 7.5, 8.2, 8.3, 8.3, 8.0].map((ms, i) => sample(i * 10, ms))
    expect(summarizeFrames(under, 50, 1000).withinBudget).toBe(true)

    const over = [7, 8, 12, 9, 10, 8.5].map((ms, i) => sample(i * 10, ms))
    expect(summarizeFrames(over, 50, 1000).withinBudget).toBe(false)
  })

  it('exposes the DESIGN.md 120 FPS budget constant', () => {
    expect(FPS_BUDGET_MS).toBeCloseTo(8.33, 2)
  })
})