import { LIMITS } from "./limits";

export interface SquareCropRect {
  srcX: number;
  srcY: number;
  cropSide: number;
  outputSize: number;
}

/**
 * Computes a 1:1 square crop anchored at normalized (cropX, cropY) in [0, 1]
 * (0.5, 0.5 = center). Output resolution depends solely on the input file,
 * capped at 512x512 and aligned to an even pixel count for video encoders.
 */
export function computeSquareCrop(
  srcWidth: number,
  srcHeight: number,
  cropX = 0.5,
  cropY = 0.5
): SquareCropRect {
  const safeW = Math.max(1, Math.round(srcWidth));
  const safeH = Math.max(1, Math.round(srcHeight));
  const cropSide = Math.min(safeW, safeH);

  const rawOut = Math.min(LIMITS.STICKER_DIMENSION_PX, cropSide);
  const outputSize = rawOut <= 2 ? rawOut : rawOut - (rawOut % 2);

  const nx = Number.isFinite(cropX) ? Math.max(0, Math.min(1, cropX)) : 0.5;
  const ny = Number.isFinite(cropY) ? Math.max(0, Math.min(1, cropY)) : 0.5;

  const srcX = Math.round((safeW - cropSide) * nx);
  const srcY = Math.round((safeH - cropSide) * ny);

  return { srcX, srcY, cropSide, outputSize };
}
