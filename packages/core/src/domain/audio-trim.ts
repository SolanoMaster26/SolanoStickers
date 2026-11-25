import { LIMITS } from "./limits";
import { SolanoStickersError } from "./errors";

export interface AudioTrimRange {
  start_ms: number;
  end_ms: number;
  volume: number;
  fade_in_ms: number;
  fade_out_ms: number;
}

export function defaultTrimForDuration(
  durationMs: number,
  maxAudioMs: number = LIMITS.DEFAULT_VIDEO_MAX_AUDIO_MS
): AudioTrimRange {
  const clampedDuration = Math.max(LIMITS.MIN_TRIM_DURATION_MS, Math.round(durationMs));
  const endMs = Math.min(clampedDuration, maxAudioMs);
  return {
    start_ms: 0,
    end_ms: endMs,
    volume: 1,
    fade_in_ms: 0,
    fade_out_ms: 0
  };
}

export function slideTrimWindow(
  desiredStartMs: number,
  windowDurationMs: number,
  totalDurationMs: number,
  maxAudioMs: number = LIMITS.DEFAULT_VIDEO_MAX_AUDIO_MS
): { start_ms: number; end_ms: number } {
  const safeWindow = Math.max(
    LIMITS.MIN_TRIM_DURATION_MS,
    Math.min(Math.round(windowDurationMs), maxAudioMs, Math.round(totalDurationMs))
  );
  const maxStart = Math.max(0, Math.round(totalDurationMs) - safeWindow);
  const start = Math.max(0, Math.min(Math.round(desiredStartMs), maxStart));
  return {
    start_ms: start,
    end_ms: start + safeWindow
  };
}

export function validateAudioTrim(
  edit: AudioTrimRange,
  totalDurationMs: number,
  maxAudioMs: number = LIMITS.DEFAULT_VIDEO_MAX_AUDIO_MS,
  minTrimMs: number = LIMITS.MIN_TRIM_DURATION_MS
): void {
  if (!Number.isInteger(edit.start_ms) || !Number.isInteger(edit.end_ms)) {
    throw new SolanoStickersError("trim-out-of-range", "Trim timestamps must be integer milliseconds.");
  }
  if (edit.start_ms < 0 || edit.end_ms > totalDurationMs) {
    throw new SolanoStickersError("trim-out-of-range", "Trim bounds exceed audio duration.");
  }
  const effectiveMinMs = Math.max(
    LIMITS.MIN_TRIM_DURATION_MS,
    Math.min(Math.round(totalDurationMs), Math.round(minTrimMs), maxAudioMs)
  );
  const selectedMs = edit.end_ms - edit.start_ms;
  if (selectedMs < effectiveMinMs || selectedMs > maxAudioMs) {
    throw new SolanoStickersError(
      "trim-out-of-range",
      `Selected duration ${selectedMs} ms must be within ${effectiveMinMs}..${maxAudioMs} ms.`
    );
  }
  if (edit.fade_in_ms < 0 || edit.fade_out_ms < 0 || edit.fade_in_ms + edit.fade_out_ms > selectedMs) {
    throw new SolanoStickersError("trim-out-of-range", "Fade durations exceed selected audio length.");
  }
}

export function clampVolume(volume: number): number {
  if (!Number.isFinite(volume)) return 1;
  return Math.max(0, Math.min(1, volume));
}
