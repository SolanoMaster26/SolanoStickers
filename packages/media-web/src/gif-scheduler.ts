import { LIMITS, SolanoStickersError } from "@solanostickers/core";

export interface GifFrameTiming {
  frameIndex: number;
  delayMs: number;
}

export interface ScheduledVideoFrame {
  frameIndex: number;
  timestampUs: number;
  durationUs: number;
}

/**
 * Loops decoded GIF frames cleanly to cover the target audio duration (capped by the active export limit).
 */
export function scheduleGifFramesForDuration(
  frames: readonly GifFrameTiming[],
  targetDurationMs: number
): ScheduledVideoFrame[] {
  if (frames.length === 0) {
    throw new SolanoStickersError("unsupported-image", "Cannot schedule zero GIF frames.");
  }
  const cappedMs = Math.max(LIMITS.MIN_TRIM_DURATION_MS, Math.min(Math.round(targetDurationMs), LIMITS.DEFAULT_VIDEO_MAX_AUDIO_MS));
  const targetUs = cappedMs * 1000;
  const schedule: ScheduledVideoFrame[] = [];

  let cursorUs = 0;
  let idx = 0;
  while (cursorUs < targetUs) {
    const f = frames[idx % frames.length]!;
    const frameDurUs = Math.max(10_000, Math.round(f.delayMs * 1000));
    const remainingUs = targetUs - cursorUs;
    const clampedDurUs = Math.min(frameDurUs, remainingUs);

    schedule.push({
      frameIndex: f.frameIndex,
      timestampUs: cursorUs,
      durationUs: clampedDurUs
    });

    cursorUs += clampedDurUs;
    idx++;
  }

  return schedule;
}

/**
 * Normalizes a GIF frame delay the way browsers play it in the editor preview:
 * a missing delay or one under 20 ms (0 or 1 centisecond) plays at 100 ms.
 */
export function gifFrameDelayMs(rawDelayMs: number | null | undefined): number {
  if (rawDelayMs == null || !Number.isFinite(rawDelayMs) || rawDelayMs < 20) {
    return 100;
  }
  return Math.round(rawDelayMs);
}

export interface EncoderGifFrame extends ScheduledVideoFrame {
  keyFrame: boolean;
}

/**
 * Resamples a GIF schedule onto a steady frame rate so hardware H.264/VP9
 * encoders advance cleanly. Every frame where a new GIF frame starts is a
 * keyframe, so players that seek or drop frames never skip a GIF frame.
 */
export function resampleGifScheduleForEncoder(
  scheduled: readonly ScheduledVideoFrame[],
  durationMs: number,
  fps = 24
): EncoderGifFrame[] {
  if (scheduled.length === 0) {
    throw new SolanoStickersError("unsupported-image", "Cannot resample zero GIF frames.");
  }
  const totalUs = Math.max(Math.round(1_000_000 / fps), durationMs * 1000);
  const out: EncoderGifFrame[] = [];
  let schedIdx = 0;

  for (let frameIdx = 0; ; frameIdx++) {
    const tUs = Math.round((frameIdx * 1_000_000) / fps);
    if (tUs >= totalUs) break;
    while (
      schedIdx < scheduled.length - 1 &&
      tUs >= scheduled[schedIdx]!.timestampUs + scheduled[schedIdx]!.durationUs
    ) {
      schedIdx++;
    }
    const nextTUs = Math.min(totalUs, Math.round(((frameIdx + 1) * 1_000_000) / fps));
    const frameIndex = scheduled[schedIdx]!.frameIndex;
    const previous = out[out.length - 1];
    out.push({
      frameIndex,
      timestampUs: tUs,
      durationUs: Math.max(1, nextTUs - tUs),
      keyFrame: previous === undefined || previous.frameIndex !== frameIndex || frameIdx % fps === 0
    });
  }
  return out;
}
