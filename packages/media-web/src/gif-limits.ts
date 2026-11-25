import { LIMITS, SolanoStickersError } from "@solanostickers/core";

const RGBA_CHANNELS = 4;

export interface GifCanvasSize {
  pixelCount: number;
  byteLength: number;
}

export function assertGifCanvas(width: number, height: number): GifCanvasSize {
  if (width <= 0 || height <= 0) {
    throw new SolanoStickersError("unsupported-image", "GIF has zero dimensions.");
  }
  if (width > LIMITS.MAX_IMAGE_DIMENSION || height > LIMITS.MAX_IMAGE_DIMENSION) {
    throw new SolanoStickersError("file-too-large", `GIF dimension ${width}x${height} exceeds 8000 px.`);
  }

  const pixelCount = width * height;
  if (!Number.isSafeInteger(pixelCount)) {
    throw new SolanoStickersError("file-too-large", "GIF canvas pixel count is not representable.");
  }
  if (pixelCount > LIMITS.MAX_IMAGE_PIXELS) {
    throw new SolanoStickersError("file-too-large", `GIF exceeds 40 megapixels (${width}x${height}).`);
  }
  if (pixelCount > LIMITS.MAX_GIF_FRAME_PIXELS) {
    throw new SolanoStickersError(
      "file-too-large",
      `GIF canvas ${width}x${height} exceeds the ${LIMITS.MAX_GIF_FRAME_PIXELS} pixel GIF limit.`
    );
  }

  const byteLength = pixelCount * RGBA_CHANNELS;
  if (!Number.isSafeInteger(byteLength) || byteLength > LIMITS.MAX_GIF_CANVAS_BYTES) {
    throw new SolanoStickersError(
      "file-too-large",
      `GIF canvas allocation of ${byteLength} bytes exceeds the ${LIMITS.MAX_GIF_CANVAS_BYTES} byte GIF limit.`
    );
  }

  return { pixelCount, byteLength };
}

export function assertGifFrameDescriptor(
  canvasWidth: number,
  canvasHeight: number,
  left: number,
  top: number,
  frameWidth: number,
  frameHeight: number
): number {
  if (frameWidth <= 0 || frameHeight <= 0) {
    throw new SolanoStickersError("unsupported-image", "GIF frame has zero dimensions.");
  }
  if (frameWidth > LIMITS.MAX_IMAGE_DIMENSION || frameHeight > LIMITS.MAX_IMAGE_DIMENSION) {
    throw new SolanoStickersError(
      "file-too-large",
      `GIF frame dimension ${frameWidth}x${frameHeight} exceeds 8000 px.`
    );
  }

  const pixelCount = frameWidth * frameHeight;
  if (!Number.isSafeInteger(pixelCount)) {
    throw new SolanoStickersError("file-too-large", "GIF frame pixel count is not representable.");
  }
  if (pixelCount > LIMITS.MAX_GIF_FRAME_PIXELS) {
    throw new SolanoStickersError(
      "file-too-large",
      `GIF frame ${frameWidth}x${frameHeight} exceeds the ${LIMITS.MAX_GIF_FRAME_PIXELS} pixel GIF limit.`
    );
  }
  if (left + frameWidth > canvasWidth || top + frameHeight > canvasHeight) {
    throw new SolanoStickersError(
      "unsupported-image",
      `GIF frame bounds (${left},${top} ${frameWidth}x${frameHeight}) exceed the ${canvasWidth}x${canvasHeight} canvas.`
    );
  }

  return pixelCount;
}

/**
 * Transient bytes `decodeGifFramesRgba` holds while it decodes one frame: the
 * working canvas, the joined LZW data, the LZW index output (doubled when the
 * frame is interlaced) and, for disposal method 3, the saved canvas region.
 * The image probe uses the same figure so that a GIF that passes the probe
 * also passes export. `crates/image-engine` (`export_gif_frame_working_bytes`)
 * mirrors this formula for the desktop probe.
 */
export function gifFrameWorkingBytes(
  canvasBytes: number,
  lzwDataBytes: number,
  framePixels: number,
  interlaced: boolean,
  disposalMethod: number
): number {
  const indexBytes = framePixels * (interlaced ? 2 : 1);
  const previousBytes = disposalMethod === 3 ? framePixels * RGBA_CHANNELS : 0;
  return canvasBytes + lzwDataBytes + indexBytes + previousBytes;
}

export function assertGifDecodedMemory(
  retainedBytes: number,
  nextRetainedBytes: number,
  workingBytes: number
): void {
  const totalBytes = retainedBytes + nextRetainedBytes + workingBytes;
  if (!Number.isSafeInteger(totalBytes) || totalBytes > LIMITS.MAX_GIF_DECODED_BYTES) {
    throw new SolanoStickersError(
      "file-too-large",
      `GIF decoded RGBA memory exceeds the ${LIMITS.MAX_GIF_DECODED_BYTES} byte limit.`
    );
  }
}

export function allocateGifBuffer(byteLength: number, label: string): Uint8Array {
  try {
    return new Uint8Array(byteLength);
  } catch (error) {
    if (error instanceof RangeError) {
      throw new SolanoStickersError(
        "file-too-large",
        `GIF ${label} allocation of ${byteLength} bytes failed.`
      );
    }
    throw error;
  }
}
