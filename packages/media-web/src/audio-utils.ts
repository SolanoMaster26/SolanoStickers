import { LIMITS, SolanoStickersError, type PeakBuckets, type PcmMono48k } from "@solanostickers/core";

export function computePeaksFromFloat32(samples: Float32Array, buckets = LIMITS.WAVEFORM_BUCKETS): PeakBuckets {
  const mins = new Float32Array(buckets);
  const maxs = new Float32Array(buckets);
  if (samples.length === 0 || buckets <= 0) {
    return { buckets, mins, maxs };
  }

  const total = samples.length;
  for (let b = 0; b < buckets; b++) {
    const start = Math.floor((b * total) / buckets);
    const end = Math.min(total, Math.max(start + 1, Math.floor(((b + 1) * total) / buckets)));
    let lo = Infinity;
    let hi = -Infinity;
    for (let i = start; i < end; i++) {
      const v = samples[i]!;
      if (v < lo) lo = v;
      if (v > hi) hi = v;
    }
    mins[b] = Number.isFinite(lo) ? lo : 0;
    maxs[b] = Number.isFinite(hi) ? hi : 0;
  }

  return { buckets, mins, maxs };
}

export function resampleMonoLinear(samples: Float32Array, fromRate: number, toRate: number): Float32Array {
  if (fromRate === toRate || samples.length === 0) {
    return new Float32Array(samples);
  }
  const outLen = Math.round((samples.length * toRate) / fromRate);
  const out = new Float32Array(outLen);
  const ratio = fromRate / toRate;
  for (let i = 0; i < outLen; i++) {
    const pos = i * ratio;
    const idx = Math.min(samples.length - 1, Math.floor(pos));
    const next = Math.min(samples.length - 1, idx + 1);
    const frac = pos - idx;
    out[i] = samples[idx]! * (1 - frac) + samples[next]! * frac;
  }
  return out;
}

export function trimPcmSamples(
  monoSamples: Float32Array,
  sampleRate: number,
  startMs: number,
  endMs: number,
  volume: number,
  maxAllowedMs: number = LIMITS.DEFAULT_VIDEO_MAX_AUDIO_MS
): PcmMono48k {
  const totalDurationMs = Math.round((monoSamples.length / sampleRate) * 1000);
  if (endMs <= startMs || startMs < 0 || endMs > totalDurationMs + LIMITS.DURATION_TOLERANCE_MS) {
    throw new SolanoStickersError("trim-out-of-range", "Trim bounds outside audio duration.");
  }
  const selectedMs = endMs - startMs;
  if (selectedMs < LIMITS.MIN_TRIM_DURATION_MS || selectedMs > maxAllowedMs) {
    throw new SolanoStickersError(
      "trim-out-of-range",
      `Selection must be between ${LIMITS.MIN_TRIM_DURATION_MS} ms and ${maxAllowedMs} ms.`
    );
  }

  const startSample = Math.max(0, Math.floor((startMs / 1000) * sampleRate));
  const endSample = Math.min(monoSamples.length, Math.floor((endMs / 1000) * sampleRate));
  const gain = Math.max(0, Math.min(1, volume));
  const sliced = new Float32Array(Math.max(0, endSample - startSample));
  for (let i = 0; i < sliced.length; i++) {
    sliced[i] = Math.max(-1, Math.min(1, monoSamples[startSample + i]! * gain));
  }

  const resampled = resampleMonoLinear(sliced, sampleRate, LIMITS.OPUS_SAMPLE_RATE);
  const durationMs = Math.round((resampled.length / LIMITS.OPUS_SAMPLE_RATE) * 1000);

  return {
    sampleRate: 48000,
    channels: 1,
    samples: resampled,
    durationMs
  };
}

export function decodeWavToMonoFloat32(bytes: Uint8Array): { samples: Float32Array; sampleRate: number } {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  let pos = 12;
  let channels = 1;
  let sampleRate = 48000;
  let bitsPerSample = 16;
  let dataOffset = -1;
  let dataSize = 0;

  while (pos + 8 <= bytes.byteLength) {
    const id = String.fromCharCode(bytes[pos]!, bytes[pos + 1]!, bytes[pos + 2]!, bytes[pos + 3]!);
    const size = view.getUint32(pos + 4, true);
    if (id === "fmt " && pos + 24 <= bytes.byteLength) {
      channels = view.getUint16(pos + 10, true);
      sampleRate = view.getUint32(pos + 12, true);
      bitsPerSample = view.getUint16(pos + 22, true);
    } else if (id === "data") {
      dataOffset = pos + 8;
      dataSize = Math.min(size, bytes.byteLength - dataOffset);
      break;
    }
    pos += 8 + size + (size % 2);
  }

  if (dataOffset < 0 || bitsPerSample !== 16) {
    throw new SolanoStickersError("unsupported-audio", "Only 16-bit PCM WAV supported in fallback decoder.");
  }

  const frameCount = Math.floor(dataSize / (2 * channels));
  const mono = new Float32Array(frameCount);
  for (let i = 0; i < frameCount; i++) {
    let sum = 0;
    for (let c = 0; c < channels; c++) {
      const sample = view.getInt16(dataOffset + (i * channels + c) * 2, true);
      sum += sample / 32768;
    }
    mono[i] = sum / channels;
  }

  return { samples: mono, sampleRate };
}

export function encodeWavBytes(pcm: PcmMono48k): Uint8Array {
  const dataBytes = pcm.samples.length * 2;
  const buf = new ArrayBuffer(44 + dataBytes);
  const view = new DataView(buf);
  const out = new Uint8Array(buf);

  const writeStr = (off: number, s: string) => {
    for (let i = 0; i < s.length; i++) out[off + i] = s.charCodeAt(i);
  };

  writeStr(0, "RIFF");
  view.setUint32(4, 36 + dataBytes, true);
  writeStr(8, "WAVE");
  writeStr(12, "fmt ");
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, pcm.channels, true);
  view.setUint32(24, pcm.sampleRate, true);
  view.setUint32(28, pcm.sampleRate * pcm.channels * 2, true);
  view.setUint16(32, pcm.channels * 2, true);
  view.setUint16(34, 16, true);
  writeStr(36, "data");
  view.setUint32(40, dataBytes, true);

  for (let i = 0; i < pcm.samples.length; i++) {
    const q = Math.round(Math.max(-1, Math.min(1, pcm.samples[i]!)) * 32767);
    view.setInt16(44 + i * 2, q, true);
  }
  return out;
}
