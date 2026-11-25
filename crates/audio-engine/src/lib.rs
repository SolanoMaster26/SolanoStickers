use std::io::Cursor;

use serde::{Deserialize, Serialize};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use thiserror::Error;

pub const MAX_AUDIO_BYTES: usize = 25 * 1024 * 1024;
pub const MAX_AUDIO_DURATION_MS: u32 = 180_000;
pub const MIN_TRIM_DURATION_MS: u32 = 100;
pub const DEFAULT_VIDEO_MAX_MS: u32 = 10_000;
pub const WAVEFORM_BUCKETS: usize = 1024;
pub const TARGET_SAMPLE_RATE: u32 = 48_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioContainerFormat {
    Mp3,
    Wav,
    Ogg,
    M4a,
    Aac,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioProbeInfo {
    pub format: AudioContainerFormat,
    pub duration_ms: u32,
    pub sample_rate: u32,
    pub channels: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeakBuckets {
    pub buckets: usize,
    pub mins: Vec<f32>,
    pub maxs: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PcmMono48k {
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Vec<f32>,
    pub duration_ms: u32,
}

#[derive(Debug, Error)]
pub enum AudioEngineError {
    #[error("unsupported-audio: {0}")]
    UnsupportedAudio(String),
    #[error("file-too-large: {0}")]
    FileTooLarge(String),
    #[error("trim-out-of-range: {0}")]
    TrimOutOfRange(String),
}

pub fn sniff_audio_format(bytes: &[u8]) -> Result<AudioContainerFormat, AudioEngineError> {
    if bytes.len() < 12 {
        return Err(AudioEngineError::UnsupportedAudio(
            "Audio header too short".to_string(),
        ));
    }
    if bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WAVE" {
        return Ok(AudioContainerFormat::Wav);
    }
    if bytes.starts_with(b"OggS") {
        return Ok(AudioContainerFormat::Ogg);
    }
    if bytes.starts_with(b"ID3") || (bytes[0] == 0xFF && (bytes[1] & 0xE0) == 0xE0) {
        return Ok(AudioContainerFormat::Mp3);
    }
    if &bytes[4..8] == b"ftyp" {
        return Ok(AudioContainerFormat::M4a);
    }
    if bytes[0] == 0xFF && (bytes[1] & 0xF6) == 0xF0 {
        return Ok(AudioContainerFormat::Aac);
    }
    Err(AudioEngineError::UnsupportedAudio(
        "Unrecognized audio container bytes".to_string(),
    ))
}

struct DecodedAudio {
    format: AudioContainerFormat,
    sample_rate: u32,
    channels: u16,
    mono_samples: Vec<f32>,
    duration_ms: u32,
}

fn decode_to_mono(bytes: &[u8]) -> Result<DecodedAudio, AudioEngineError> {
    if bytes.is_empty() {
        return Err(AudioEngineError::UnsupportedAudio("Empty audio".to_string()));
    }
    if bytes.len() > MAX_AUDIO_BYTES {
        return Err(AudioEngineError::FileTooLarge(
            "Audio exceeds 25 MB limit".to_string(),
        ));
    }

    let format = sniff_audio_format(bytes)?;
    let mut hint = Hint::new();
    match format {
        AudioContainerFormat::Mp3 => {
            hint.with_extension("mp3");
        }
        AudioContainerFormat::Wav => {
            hint.with_extension("wav");
        }
        AudioContainerFormat::Ogg => {
            hint.with_extension("ogg");
        }
        AudioContainerFormat::M4a => {
            hint.with_extension("m4a");
        }
        AudioContainerFormat::Aac => {
            hint.with_extension("aac");
        }
    }

    let mss = MediaSourceStream::new(Box::new(Cursor::new(bytes.to_vec())), Default::default());
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| AudioEngineError::UnsupportedAudio(format!("Probe failed: {e}")))?;

    let mut reader = probed.format;
    let track = reader
        .default_track()
        .ok_or_else(|| AudioEngineError::UnsupportedAudio("No default audio track".to_string()))?;
    let track_id = track.id;
    let codec_params = track.codec_params.clone();

    let sample_rate = codec_params
        .sample_rate
        .ok_or_else(|| AudioEngineError::UnsupportedAudio("Missing sample rate".to_string()))?;
    let channels = codec_params
        .channels
        .map(|c| c.count() as u16)
        .unwrap_or(1)
        .max(1);

    let mut decoder = symphonia::default::get_codecs()
        .make(&codec_params, &DecoderOptions::default())
        .map_err(|e| AudioEngineError::UnsupportedAudio(format!("Decoder init failed: {e}")))?;

    let mut mono_samples = Vec::new();
    let max_samples_allowed = ((u64::from(MAX_AUDIO_DURATION_MS) + 500) * u64::from(sample_rate)) / 1000;

    while let Ok(packet) = reader.next_packet() {
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(e) => {
                return Err(AudioEngineError::UnsupportedAudio(format!(
                    "Decode error: {e}"
                )))
            }
        };
        let spec = *decoded.spec();
        let ch_count = spec.channels.count().max(1);
        let mut sbuf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        sbuf.copy_interleaved_ref(decoded);
        for frame in sbuf.samples().chunks_exact(ch_count) {
            let sum: f32 = frame.iter().copied().sum();
            mono_samples.push(sum / ch_count as f32);
        }
        if mono_samples.len() as u64 > max_samples_allowed {
            return Err(AudioEngineError::FileTooLarge(
                "Audio exceeds 3 minute editing limit".to_string(),
            ));
        }
    }

    if mono_samples.is_empty() {
        return Err(AudioEngineError::UnsupportedAudio(
            "Decoded zero audio samples".to_string(),
        ));
    }

    let duration_ms = ((mono_samples.len() as u64 * 1000) / u64::from(sample_rate)) as u32;
    if duration_ms > MAX_AUDIO_DURATION_MS {
        return Err(AudioEngineError::FileTooLarge(
            "Audio exceeds 3 minute editing limit".to_string(),
        ));
    }
    if duration_ms < MIN_TRIM_DURATION_MS {
        return Err(AudioEngineError::UnsupportedAudio(
            "Audio shorter than 100 ms".to_string(),
        ));
    }

    Ok(DecodedAudio {
        format,
        sample_rate,
        channels,
        mono_samples,
        duration_ms,
    })
}

pub fn probe_audio(bytes: &[u8]) -> Result<AudioProbeInfo, AudioEngineError> {
    let decoded = decode_to_mono(bytes)?;
    Ok(AudioProbeInfo {
        format: decoded.format,
        duration_ms: decoded.duration_ms,
        sample_rate: decoded.sample_rate,
        channels: decoded.channels,
    })
}

pub fn compute_waveform_peaks(bytes: &[u8]) -> Result<PeakBuckets, AudioEngineError> {
    let decoded = decode_to_mono(bytes)?;
    Ok(peaks_from_samples(&decoded.mono_samples, WAVEFORM_BUCKETS))
}

pub fn peaks_from_samples(samples: &[f32], buckets: usize) -> PeakBuckets {
    let mut mins = vec![0.0f32; buckets];
    let mut maxs = vec![0.0f32; buckets];
    if samples.is_empty() || buckets == 0 {
        return PeakBuckets {
            buckets,
            mins,
            maxs,
        };
    }

    let total = samples.len();
    for b in 0..buckets {
        let start = (b * total) / buckets;
        let end = (((b + 1) * total) / buckets).max(start + 1).min(total);
        let mut lo = f32::INFINITY;
        let mut hi = f32::NEG_INFINITY;
        for &s in &samples[start..end] {
            if s < lo {
                lo = s;
            }
            if s > hi {
                hi = s;
            }
        }
        mins[b] = if lo.is_finite() { lo } else { 0.0 };
        maxs[b] = if hi.is_finite() { hi } else { 0.0 };
    }

    PeakBuckets {
        buckets,
        mins,
        maxs,
    }
}

fn resample_linear(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate || samples.is_empty() {
        return samples.to_vec();
    }
    let out_len = ((samples.len() as f64) * f64::from(to_rate) / f64::from(from_rate)).round() as usize;
    let mut out = Vec::with_capacity(out_len);
    let ratio = f64::from(from_rate) / f64::from(to_rate);
    for i in 0..out_len {
        let pos = i as f64 * ratio;
        let idx = (pos.floor() as usize).min(samples.len() - 1);
        let next = (idx + 1).min(samples.len() - 1);
        let frac = (pos - idx as f64) as f32;
        out.push(samples[idx] * (1.0 - frac) + samples[next] * frac);
    }
    out
}

pub fn render_trimmed_pcm_48k(
    bytes: &[u8],
    start_ms: u32,
    end_ms: u32,
    volume: f32,
    max_allowed_ms: u32,
) -> Result<PcmMono48k, AudioEngineError> {
    let decoded = decode_to_mono(bytes)?;
    if end_ms <= start_ms || end_ms > decoded.duration_ms {
        return Err(AudioEngineError::TrimOutOfRange(
            "Trim bounds out of range".to_string(),
        ));
    }
    let selected_ms = end_ms - start_ms;
    if selected_ms < MIN_TRIM_DURATION_MS || selected_ms > max_allowed_ms {
        return Err(AudioEngineError::TrimOutOfRange(format!(
            "Selected {selected_ms}ms outside {MIN_TRIM_DURATION_MS}..{max_allowed_ms}ms"
        )));
    }

    let start_sample = ((u64::from(start_ms) * u64::from(decoded.sample_rate)) / 1000) as usize;
    let end_sample = (((u64::from(end_ms) * u64::from(decoded.sample_rate)) / 1000) as usize)
        .min(decoded.mono_samples.len());
    let sliced = &decoded.mono_samples[start_sample.min(end_sample)..end_sample];
    let gain = volume.clamp(0.0, 1.0);
    let gained: Vec<f32> = sliced
        .iter()
        .map(|&s| (s * gain).clamp(-1.0, 1.0))
        .collect();
    let resampled = resample_linear(&gained, decoded.sample_rate, TARGET_SAMPLE_RATE);
    let out_duration_ms = ((resampled.len() as u64 * 1000) / u64::from(TARGET_SAMPLE_RATE)) as u32;

    Ok(PcmMono48k {
        sample_rate: TARGET_SAMPLE_RATE,
        channels: 1,
        samples: resampled,
        duration_ms: out_duration_ms,
    })
}

/// Encodes mono 48 kHz float samples to a standard PCM 16-bit WAV container.
pub fn encode_wav_pcm16(pcm: &PcmMono48k) -> Vec<u8> {
    let data_bytes = (pcm.samples.len() * 2) as u32;
    let riff_size = 36 + data_bytes;
    let mut out = Vec::with_capacity((44 + data_bytes) as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&riff_size.to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&pcm.channels.to_le_bytes());
    out.extend_from_slice(&pcm.sample_rate.to_le_bytes());
    let byte_rate = pcm.sample_rate * u32::from(pcm.channels) * 2;
    out.extend_from_slice(&byte_rate.to_le_bytes());
    let block_align = pcm.channels * 2;
    out.extend_from_slice(&block_align.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_bytes.to_le_bytes());
    for &s in &pcm.samples {
        let q = (s.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        out.extend_from_slice(&q.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sine_wav(duration_ms: u32, sample_rate: u32) -> Vec<u8> {
        let count = ((u64::from(duration_ms) * u64::from(sample_rate)) / 1000) as usize;
        let samples: Vec<f32> = (0..count)
            .map(|i| ((i as f32 * 440.0 * 2.0 * std::f32::consts::PI) / sample_rate as f32).sin() * 0.5)
            .collect();
        let pcm = PcmMono48k {
            sample_rate,
            channels: 1,
            samples,
            duration_ms,
        };
        encode_wav_pcm16(&pcm)
    }

    #[test]
    fn tc003_tc007_trims_to_10s_and_computes_1024_peaks() {
        let wav = make_sine_wav(12_000, 48000);
        let probe = probe_audio(&wav).unwrap();
        assert_eq!(probe.format, AudioContainerFormat::Wav);
        assert_eq!(probe.duration_ms, 12_000);

        let peaks = compute_waveform_peaks(&wav).unwrap();
        assert_eq!(peaks.buckets, 1024);
        assert_eq!(peaks.mins.len(), 1024);
        assert_eq!(peaks.maxs.len(), 1024);

        let trimmed_10s =
            render_trimmed_pcm_48k(&wav, 0, 10_000, 1.0, DEFAULT_VIDEO_MAX_MS).unwrap();
        assert!((i64::from(trimmed_10s.duration_ms) - 10_000).abs() <= 50);

        let trimmed_mid =
            render_trimmed_pcm_48k(&wav, 1500, 4200, 1.0, DEFAULT_VIDEO_MAX_MS).unwrap();
        assert!((i64::from(trimmed_mid.duration_ms) - 2700).abs() <= 50);
    }

    #[test]
    fn tc006_and_tc014_reject_corrupt_and_over_10s_trim() {
        let corrupt = b"not valid mp3 or wav bytes";
        assert!(matches!(
            probe_audio(corrupt),
            Err(AudioEngineError::UnsupportedAudio(_))
        ));

        let wav = make_sine_wav(12_000, 48000);
        let err = render_trimmed_pcm_48k(&wav, 0, 11_000, 1.0, DEFAULT_VIDEO_MAX_MS).unwrap_err();
        assert!(matches!(err, AudioEngineError::TrimOutOfRange(_)));
    }
}
