import { LIMITS, SolanoStickersError } from "@solanostickers/core";
import {
  allocateGifBuffer,
  assertGifCanvas,
  assertGifDecodedMemory,
  assertGifFrameDescriptor,
  gifFrameWorkingBytes
} from "./gif-limits";
import { gifFrameDelayMs } from "./gif-scheduler";

export interface DecodedGifFrame {
  width: number;
  height: number;
  delayMs: number;
  rgba: Uint8Array;
}

function readU16LE(b: Uint8Array, o: number): number {
  return (b[o] ?? 0) | ((b[o + 1] ?? 0) << 8);
}

function ascii(b: Uint8Array, start: number, len: number): string {
  let s = "";
  for (let i = 0; i < len && start + i < b.length; i++) {
    s += String.fromCharCode(b[start + i]!);
  }
  return s;
}

function lzwDecode(minCodeSize: number, data: Uint8Array, pixelCount: number): Uint8Array {
  if (minCodeSize < 1 || minCodeSize > 11) {
    throw new SolanoStickersError("unsupported-image", `Invalid GIF LZW minimum code size: ${minCodeSize}.`);
  }

  const clearCode = 1 << minCodeSize;
  const eoiCode = clearCode + 1;
  const out = allocateGifBuffer(pixelCount, "LZW output");

  const prefix = new Int32Array(4096);
  const suffix = new Uint8Array(4096);
  const pixelStack = new Uint8Array(4097);

  for (let i = 0; i < clearCode; i++) {
    suffix[i] = i;
  }

  let codeSize = minCodeSize + 1;
  let codeMask = (1 << codeSize) - 1;
  let nextCode = eoiCode + 1;
  let oldCode = -1;
  let firstChar = 0;

  let bitBuf = 0;
  let bitCount = 0;
  let bytePos = 0;
  let outPos = 0;

  while (outPos < pixelCount) {
    while (bitCount < codeSize && bytePos < data.length) {
      bitBuf |= data[bytePos++]! << bitCount;
      bitCount += 8;
    }
    if (bitCount < codeSize) {
      break;
    }

    const code = bitBuf & codeMask;
    bitBuf >>>= codeSize;
    bitCount -= codeSize;

    if (code === clearCode) {
      codeSize = minCodeSize + 1;
      codeMask = (1 << codeSize) - 1;
      nextCode = eoiCode + 1;
      oldCode = -1;
      continue;
    }

    if (code === eoiCode) {
      break;
    }

    if (oldCode === -1) {
      if (code >= clearCode) {
        break;
      }
      out[outPos++] = code;
      oldCode = code;
      firstChar = code;
      continue;
    }

    const inCode = code;
    let cur = code;
    let stackTop = 0;

    if (cur >= nextCode) {
      if (cur > nextCode) {
        break;
      }
      pixelStack[stackTop++] = firstChar;
      cur = oldCode;
    }

    while (cur >= clearCode && stackTop < pixelStack.length) {
      pixelStack[stackTop++] = suffix[cur]!;
      cur = prefix[cur]!;
    }

    if (cur < 0 || cur >= clearCode) {
      break;
    }

    firstChar = cur & 0xff;
    pixelStack[stackTop++] = firstChar;

    if (nextCode < 4096) {
      prefix[nextCode] = oldCode;
      suffix[nextCode] = firstChar;
      nextCode++;
      if ((nextCode & codeMask) === 0 && nextCode < 4096) {
        codeSize++;
        codeMask = (1 << codeSize) - 1;
      }
    }

    while (stackTop > 0 && outPos < pixelCount) {
      out[outPos++] = pixelStack[--stackTop]!;
    }

    oldCode = inCode;
  }

  return out;
}

function deinterlaceIndices(indices: Uint8Array, width: number, height: number): Uint8Array {
  const out = new Uint8Array(indices.length);
  const starts = [0, 4, 2, 1];
  const steps = [8, 8, 4, 2];
  let srcRow = 0;

  for (let pass = 0; pass < 4; pass++) {
    for (let y = starts[pass]!; y < height; y += steps[pass]!) {
      if (srcRow < height) {
        const srcOff = srcRow * width;
        const dstOff = y * width;
        out.set(indices.subarray(srcOff, srcOff + width), dstOff);
        srcRow++;
      }
    }
  }
  return out;
}

/**
 * Pure TypeScript GIF87a/GIF89a frame decoder that composites every frame onto
 * full RGBA buffers while respecting frame offsets, transparency, interlacing,
 * and GIF disposal methods (none, keep, background, previous).
 */
export function decodeGifFramesRgba(bytes: Uint8Array): DecodedGifFrame[] {
  if (bytes.byteLength < 14) {
    throw new SolanoStickersError("unsupported-image", "GIF buffer too small.");
  }
  if (bytes.byteLength > LIMITS.MAX_IMAGE_BYTES) {
    throw new SolanoStickersError("file-too-large", "GIF exceeds 20 MB limit.");
  }

  const sig = ascii(bytes, 0, 6);
  if (sig !== "GIF87a" && sig !== "GIF89a") {
    throw new SolanoStickersError("unsupported-image", "Not a GIF87a/GIF89a stream.");
  }

  const width = readU16LE(bytes, 6);
  const height = readU16LE(bytes, 8);
  const canvasSize = assertGifCanvas(width, height);

  const packed = bytes[10] ?? 0;
  let pos = 13;
  let gct: Uint8Array | null = null;
  if ((packed & 0x80) !== 0) {
    const gctSize = 3 * (1 << ((packed & 0x07) + 1));
    if (pos + gctSize > bytes.byteLength) {
      throw new SolanoStickersError("unsupported-image", "Truncated GIF global color table.");
    }
    gct = bytes.subarray(pos, pos + gctSize);
    pos += gctSize;
  }

  const currentCanvas = allocateGifBuffer(canvasSize.byteLength, "canvas");
  const frames: DecodedGifFrame[] = [];
  let retainedBytes = 0;
  let descriptorCount = 0;

  let disposalMethod = 0;
  let transparentFlag = false;
  let transparentIndex = 0;
  let delayMs = 100;

  while (pos < bytes.byteLength) {
    const block = bytes[pos++];
    if (block === 0x3b) {
      break;
    }

    if (block === 0x21) {
      if (pos >= bytes.byteLength) break;
      const label = bytes[pos++];
      if (label === 0xf9 && pos < bytes.byteLength) {
        const blockSize = bytes[pos]!;
        if (blockSize < 4 || pos + 1 + blockSize > bytes.byteLength) break;
        const gcePacked = bytes[pos + 1]!;
        disposalMethod = (gcePacked >> 2) & 0x07;
        transparentFlag = (gcePacked & 0x01) !== 0;
        const delayCs = readU16LE(bytes, pos + 2);
        delayMs = gifFrameDelayMs(delayCs * 10);
        transparentIndex = bytes[pos + 4] ?? 0;
        pos += 1 + blockSize;
        while (pos < bytes.byteLength) {
          const subLen = bytes[pos++]!;
          if (subLen === 0) break;
          if (subLen > bytes.byteLength - pos) {
            pos = bytes.byteLength;
            break;
          }
          pos += subLen;
        }
      } else {
        while (pos < bytes.byteLength) {
          const subLen = bytes[pos++] ?? 0;
          if (subLen === 0) break;
          if (subLen > bytes.byteLength - pos) {
            pos = bytes.byteLength;
            break;
          }
          pos += subLen;
        }
      }
      continue;
    }

    if (block === 0x2c) {
      if (descriptorCount >= LIMITS.MAX_GIF_FRAMES) {
        throw new SolanoStickersError(
          "file-too-large",
          `GIF exceeds the ${LIMITS.MAX_GIF_FRAMES} frame limit.`
        );
      }
      descriptorCount++;
      if (pos + 9 > bytes.byteLength) break;
      const left = readU16LE(bytes, pos);
      const top = readU16LE(bytes, pos + 2);
      const frameWidth = readU16LE(bytes, pos + 4);
      const frameHeight = readU16LE(bytes, pos + 6);
      const imgPacked = bytes[pos + 8] ?? 0;
      pos += 9;

      const framePixels = assertGifFrameDescriptor(width, height, left, top, frameWidth, frameHeight);
      const frameBytes = framePixels * 4;

      const hasLct = (imgPacked & 0x80) !== 0;
      const interlaced = (imgPacked & 0x40) !== 0;
      let palette = gct;
      if (hasLct) {
        const lctSize = 3 * (1 << ((imgPacked & 0x07) + 1));
        if (pos + lctSize > bytes.byteLength) break;
        palette = bytes.subarray(pos, pos + lctSize);
        pos += lctSize;
      }

      if (pos >= bytes.byteLength) break;
      const minCodeSize = bytes[pos++]!;

      const chunks: Uint8Array[] = [];
      let totalDataLen = 0;
      while (pos < bytes.byteLength) {
        const subLen = bytes[pos++] ?? 0;
        if (subLen === 0) break;
        const end = Math.min(bytes.byteLength, pos + subLen);
        const slice = bytes.subarray(pos, end);
        chunks.push(slice);
        totalDataLen += slice.length;
        pos = end;
      }

      if (!palette) {
        disposalMethod = 0;
        transparentFlag = false;
        transparentIndex = 0;
        delayMs = 100;
        continue;
      }

      const needsPrevious = disposalMethod === 3;
      const workingBytes = gifFrameWorkingBytes(
        canvasSize.byteLength,
        totalDataLen,
        framePixels,
        interlaced,
        disposalMethod
      );
      assertGifDecodedMemory(retainedBytes, canvasSize.byteLength, workingBytes);

      const lzwData = allocateGifBuffer(totalDataLen, "LZW data");
      let chunkOff = 0;
      for (const c of chunks) {
        lzwData.set(c, chunkOff);
        chunkOff += c.length;
      }

      let indices = lzwDecode(minCodeSize, lzwData, framePixels);
      if (interlaced) {
        indices = deinterlaceIndices(indices, frameWidth, frameHeight);
      }

      let prevRegion: Uint8Array | null = null;
      if (needsPrevious) {
        prevRegion = allocateGifBuffer(frameBytes, "disposal buffer");
        for (let y = 0; y < frameHeight; y++) {
          const srcOff = ((top + y) * width + left) * 4;
          prevRegion.set(currentCanvas.subarray(srcOff, srcOff + frameWidth * 4), y * frameWidth * 4);
        }
      }

      for (let y = 0; y < frameHeight; y++) {
        const cy = top + y;
        for (let x = 0; x < frameWidth; x++) {
          const cx = left + x;
          const colorIdx = indices[y * frameWidth + x]!;
          if (transparentFlag && colorIdx === transparentIndex) {
            continue;
          }
          const palOff = colorIdx * 3;
          if (palOff + 2 >= palette.length) {
            continue;
          }
          const dstOff = (cy * width + cx) * 4;
          currentCanvas[dstOff] = palette[palOff]!;
          currentCanvas[dstOff + 1] = palette[palOff + 1]!;
          currentCanvas[dstOff + 2] = palette[palOff + 2]!;
          currentCanvas[dstOff + 3] = 255;
        }
      }

      const frameRgba = allocateGifBuffer(canvasSize.byteLength, "frame buffer");
      frameRgba.set(currentCanvas);
      frames.push({
        width,
        height,
        delayMs,
        rgba: frameRgba
      });
      retainedBytes += canvasSize.byteLength;

      if (disposalMethod === 2) {
        for (let y = 0; y < frameHeight; y++) {
          const cy = top + y;
          for (let x = 0; x < frameWidth; x++) {
            const cx = left + x;
            const dstOff = (cy * width + cx) * 4;
            currentCanvas[dstOff] = 0;
            currentCanvas[dstOff + 1] = 0;
            currentCanvas[dstOff + 2] = 0;
            currentCanvas[dstOff + 3] = 0;
          }
        }
      } else if (disposalMethod === 3 && prevRegion) {
        for (let y = 0; y < frameHeight; y++) {
          const dstOff = ((top + y) * width + left) * 4;
          const srcOff = y * frameWidth * 4;
          currentCanvas.set(prevRegion.subarray(srcOff, srcOff + frameWidth * 4), dstOff);
        }
      }

      disposalMethod = 0;
      transparentFlag = false;
      transparentIndex = 0;
      delayMs = 100;
      continue;
    }

    break;
  }

  if (frames.length === 0) {
    throw new SolanoStickersError("unsupported-image", "GIF contained zero decodable frames.");
  }

  return frames;
}
