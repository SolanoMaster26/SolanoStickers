use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const MAX_VIDEO_DURATION_MS: u32 = 10_000;
pub const MIN_VIDEO_DURATION_MS: u32 = 100;
pub const STICKER_PX: u32 = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VideoContainerProfile {
    VideoMp4,
    VideoWebmAlpha,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoMuxRequest {
    pub project_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub sticker_path: PathBuf,
    pub sticker_animated_gif: bool,
    pub audio_path: PathBuf,
    pub output_path: PathBuf,
    pub start_ms: u32,
    pub end_ms: u32,
    pub matte_color_hex: String,
    pub profile: VideoContainerProfile,
}

#[derive(Debug, Error)]
pub enum VideoEngineError {
    #[error("trim-out-of-range: {0}")]
    TrimOutOfRange(String),
    #[error("package-invalid: {0}")]
    UnsafePath(String),
    #[error("encode-failed: {0}")]
    EncodeFailed(String),
}

pub fn validate_matte_hex(raw: &str) -> Result<String, VideoEngineError> {
    let trimmed = raw.trim();
    if trimmed.len() == 7
        && trimmed.starts_with('#')
        && trimmed[1..].chars().all(|c| c.is_ascii_hexdigit())
    {
        return Ok(format!("0x{}", &trimmed[1..].to_uppercase()));
    }
    Err(VideoEngineError::EncodeFailed(format!(
        "Invalid matte hex color: {raw}"
    )))
}

fn ensure_confined_path(path: &Path, base_dir: &Path, label: &str) -> Result<PathBuf, VideoEngineError> {
    if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err(VideoEngineError::UnsafePath(format!(
            "Parent traversal '..' forbidden in {label}"
        )));
    }
    if !path.starts_with(base_dir) {
        return Err(VideoEngineError::UnsafePath(format!(
            "{label} must remain inside its confined directory"
        )));
    }
    Ok(path.to_path_buf())
}

pub fn preferred_h264_encoder() -> &'static str {
    if cfg!(target_os = "windows") {
        "h264_mf"
    } else if cfg!(target_os = "macos") {
        "h264_videotoolbox"
    } else {
        "libopenh264"
    }
}

/// Builds a strictly structured FFmpeg argument list (zero shell string interpolation).
pub fn build_ffmpeg_args(req: &VideoMuxRequest) -> Result<Vec<String>, VideoEngineError> {
    if req.end_ms <= req.start_ms {
        return Err(VideoEngineError::TrimOutOfRange(
            "end_ms must exceed start_ms".to_string(),
        ));
    }
    let duration_ms = req.end_ms - req.start_ms;
    if duration_ms < MIN_VIDEO_DURATION_MS || duration_ms > MAX_VIDEO_DURATION_MS {
        return Err(VideoEngineError::TrimOutOfRange(format!(
            "Video duration {duration_ms}ms must be within {MIN_VIDEO_DURATION_MS}..{MAX_VIDEO_DURATION_MS}ms"
        )));
    }

    let sticker = ensure_confined_path(&req.sticker_path, &req.project_dir, "sticker_path")?;
    let audio = ensure_confined_path(&req.audio_path, &req.project_dir, "audio_path")?;
    let output = ensure_confined_path(&req.output_path, &req.temp_dir, "output_path")?;

    let start_sec = format!("{:.3}", f64::from(req.start_ms) / 1000.0);
    let dur_sec = format!("{:.3}", f64::from(duration_ms) / 1000.0);

    let mut args = vec!["-y".to_string()];

    if req.sticker_animated_gif {
        args.extend(["-stream_loop".to_string(), "-1".to_string()]);
    } else {
        args.extend([
            "-loop".to_string(),
            "1".to_string(),
            "-framerate".to_string(),
            "15".to_string(),
        ]);
    }
    args.extend(["-i".to_string(), sticker.to_string_lossy().into_owned()]);

    args.extend([
        "-ss".to_string(),
        start_sec,
        "-t".to_string(),
        dur_sec.clone(),
        "-i".to_string(),
        audio.to_string_lossy().into_owned(),
    ]);

    match req.profile {
        VideoContainerProfile::VideoMp4 => {
            let matte = validate_matte_hex(&req.matte_color_hex)?;
            let vf = if req.sticker_animated_gif {
                format!(
                    "scale={s}:{s}:force_original_aspect_ratio=decrease,pad={s}:{s}:(ow-iw)/2:(oh-ih)/2:color={matte},fps=24,format=yuv420p",
                    s = STICKER_PX
                )
            } else {
                format!(
                    "scale={s}:{s}:force_original_aspect_ratio=decrease,pad={s}:{s}:(ow-iw)/2:(oh-ih)/2:color={matte},format=yuv420p",
                    s = STICKER_PX
                )
            };
            args.extend([
                "-vf".to_string(),
                vf,
                "-c:v".to_string(),
                preferred_h264_encoder().to_string(),
                "-c:a".to_string(),
                "aac".to_string(),
                "-b:a".to_string(),
                "128k".to_string(),
                "-ar".to_string(),
                "48000".to_string(),
                "-ac".to_string(),
                "1".to_string(),
                "-movflags".to_string(),
                "+faststart".to_string(),
            ]);
        }
        VideoContainerProfile::VideoWebmAlpha => {
            let vf = format!(
                "scale={s}:{s}:force_original_aspect_ratio=decrease,pad={s}:{s}:(ow-iw)/2:(oh-ih)/2:color=0x00000000,format=yuva420p",
                s = STICKER_PX
            );
            args.extend([
                "-vf".to_string(),
                vf,
                "-c:v".to_string(),
                "libvpx-vp9".to_string(),
                "-pix_fmt".to_string(),
                "yuva420p".to_string(),
                "-auto-alt-ref".to_string(),
                "0".to_string(),
                "-c:a".to_string(),
                "libopus".to_string(),
                "-b:a".to_string(),
                "64k".to_string(),
                "-ar".to_string(),
                "48000".to_string(),
                "-ac".to_string(),
                "1".to_string(),
            ]);
        }
    }

    args.extend([
        "-t".to_string(),
        dur_sec,
        "-shortest".to_string(),
        output.to_string_lossy().into_owned(),
    ]);

    Ok(args)
}

pub fn run_ffmpeg_mux(ffmpeg_bin: &Path, req: &VideoMuxRequest) -> Result<(), VideoEngineError> {
    let args = build_ffmpeg_args(req)?;
    let status = Command::new(ffmpeg_bin)
        .args(&args)
        .status()
        .map_err(|e| VideoEngineError::EncodeFailed(format!("Failed to start ffmpeg: {e}")))?;
    if !status.success() {
        return Err(VideoEngineError::EncodeFailed(format!(
            "ffmpeg exited with status {status}"
        )));
    }
    Ok(())
}

/// Pure-Rust, zero-dependency self-contained AVI (DIB RGB24 + PCM 16-bit audio)
/// writer for environments without an external FFmpeg binary.
pub fn encode_self_contained_video(
    rgba_frames: &[(&[u8], u32)],
    pcm_samples: &[i16],
    sample_rate: u32,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, VideoEngineError> {
    if rgba_frames.is_empty() || width == 0 || height == 0 {
        return Err(VideoEngineError::EncodeFailed(
            "Video frames and dimensions must be non-empty".to_string(),
        ));
    }
    let audio_dur_ms = ((pcm_samples.len() as u64 * 1000) / u64::from(sample_rate.max(1))) as u32;
    if audio_dur_ms < MIN_VIDEO_DURATION_MS || audio_dur_ms > MAX_VIDEO_DURATION_MS {
        return Err(VideoEngineError::TrimOutOfRange(format!(
            "Audio duration {audio_dur_ms}ms must be within {MIN_VIDEO_DURATION_MS}..{MAX_VIDEO_DURATION_MS}ms"
        )));
    }

    let fps: u32 = 10;
    let total_video_frames = ((u64::from(audio_dur_ms) * u64::from(fps)) / 1000).max(1) as usize;
    let stride = (width as usize * 3 + 3) & !3;
    let frame_byte_len = stride * height as usize;

    // Convert RGBA frames (top-down) into bottom-up BGR24 DIB buffers
    let mut bgr_frames: Vec<Vec<u8>> = Vec::with_capacity(rgba_frames.len());
    for &(rgba, _) in rgba_frames {
        let mut dib = vec![0u8; frame_byte_len];
        for y in 0..height as usize {
            let src_y = height as usize - 1 - y;
            for x in 0..width as usize {
                let s_idx = (src_y * width as usize + x) * 4;
                let d_idx = y * stride + x * 3;
                if s_idx + 2 < rgba.len() {
                    dib[d_idx] = rgba[s_idx + 2];
                    dib[d_idx + 1] = rgba[s_idx + 1];
                    dib[d_idx + 2] = rgba[s_idx];
                }
            }
        }
        bgr_frames.push(dib);
    }

    let mut movi = Vec::new();
    movi.extend_from_slice(b"movi");
    let samples_per_frame = (sample_rate / fps).max(1) as usize;
    let mut audio_cursor = 0usize;

    for f_idx in 0..total_video_frames {
        let frame_data = &bgr_frames[f_idx % bgr_frames.len()];
        movi.extend_from_slice(b"00db");
        movi.extend_from_slice(&(frame_data.len() as u32).to_le_bytes());
        movi.extend_from_slice(frame_data);
        if frame_data.len() % 2 == 1 {
            movi.push(0);
        }

        if audio_cursor < pcm_samples.len() {
            let end = if f_idx + 1 == total_video_frames {
                pcm_samples.len()
            } else {
                (audio_cursor + samples_per_frame).min(pcm_samples.len())
            };
            let slice = &pcm_samples[audio_cursor..end];
            let byte_len = (slice.len() * 2) as u32;
            movi.extend_from_slice(b"01wb");
            movi.extend_from_slice(&byte_len.to_le_bytes());
            for &s in slice {
                movi.extend_from_slice(&s.to_le_bytes());
            }
            audio_cursor = end;
        }
    }

    fn list_chunk(tag: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let mut v = Vec::with_capacity(8 + payload.len());
        v.extend_from_slice(b"LIST");
        v.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        v.extend_from_slice(tag);
        v.extend_from_slice(&payload[4..]);
        v
    }
    let _ = list_chunk;

    // Build hdrl LIST
    let mut avih = Vec::with_capacity(56);
    let us_per_frame = 1_000_000 / fps;
    avih.extend_from_slice(&us_per_frame.to_le_bytes());
    avih.extend_from_slice(&(frame_byte_len as u32 * fps).to_le_bytes());
    avih.extend_from_slice(&0u32.to_le_bytes());
    avih.extend_from_slice(&0x10u32.to_le_bytes()); // AVIF_HASINDEX
    avih.extend_from_slice(&(total_video_frames as u32).to_le_bytes());
    avih.extend_from_slice(&0u32.to_le_bytes());
    avih.extend_from_slice(&2u32.to_le_bytes()); // 2 streams: video + audio
    avih.extend_from_slice(&(frame_byte_len as u32).to_le_bytes());
    avih.extend_from_slice(&width.to_le_bytes());
    avih.extend_from_slice(&height.to_le_bytes());
    avih.extend_from_slice(&[0u8; 16]);

    // Video stream header (strl)
    let mut v_strh = Vec::with_capacity(56);
    v_strh.extend_from_slice(b"vids");
    v_strh.extend_from_slice(b"DIB ");
    v_strh.extend_from_slice(&[0u8; 12]);
    v_strh.extend_from_slice(&1u32.to_le_bytes()); // scale
    v_strh.extend_from_slice(&fps.to_le_bytes()); // rate
    v_strh.extend_from_slice(&0u32.to_le_bytes());
    v_strh.extend_from_slice(&(total_video_frames as u32).to_le_bytes());
    v_strh.extend_from_slice(&(frame_byte_len as u32).to_le_bytes());
    v_strh.extend_from_slice(&0u32.to_le_bytes());
    v_strh.extend_from_slice(&0u32.to_le_bytes());
    v_strh.extend_from_slice(&(width as u16).to_le_bytes());
    v_strh.extend_from_slice(&(height as u16).to_le_bytes());
    v_strh.extend_from_slice(&[0u8; 4]);

    let mut v_strf = Vec::with_capacity(40);
    v_strf.extend_from_slice(&40u32.to_le_bytes());
    v_strf.extend_from_slice(&width.to_le_bytes());
    v_strf.extend_from_slice(&height.to_le_bytes());
    v_strf.extend_from_slice(&1u16.to_le_bytes()); // planes
    v_strf.extend_from_slice(&24u16.to_le_bytes()); // bpp
    v_strf.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB
    v_strf.extend_from_slice(&(frame_byte_len as u32).to_le_bytes());
    v_strf.extend_from_slice(&[0u8; 16]);

    let mut v_strl = Vec::new();
    v_strl.extend_from_slice(b"strl");
    v_strl.extend_from_slice(b"strh");
    v_strl.extend_from_slice(&(v_strh.len() as u32).to_le_bytes());
    v_strl.extend_from_slice(&v_strh);
    v_strl.extend_from_slice(b"strf");
    v_strl.extend_from_slice(&(v_strf.len() as u32).to_le_bytes());
    v_strl.extend_from_slice(&v_strf);

    // Audio stream header (strl)
    let mut a_strh = Vec::with_capacity(56);
    a_strh.extend_from_slice(b"auds");
    a_strh.extend_from_slice(&0u32.to_le_bytes());
    a_strh.extend_from_slice(&[0u8; 12]);
    a_strh.extend_from_slice(&1u32.to_le_bytes());
    a_strh.extend_from_slice(&sample_rate.to_le_bytes());
    a_strh.extend_from_slice(&0u32.to_le_bytes());
    a_strh.extend_from_slice(&(pcm_samples.len() as u32).to_le_bytes());
    a_strh.extend_from_slice(&((sample_rate * 2) as u32).to_le_bytes());
    a_strh.extend_from_slice(&0u32.to_le_bytes());
    a_strh.extend_from_slice(&2u32.to_le_bytes()); // sample size
    a_strh.extend_from_slice(&[0u8; 8]);

    let mut a_strf = Vec::with_capacity(18);
    a_strf.extend_from_slice(&1u16.to_le_bytes()); // PCM
    a_strf.extend_from_slice(&1u16.to_le_bytes()); // mono
    a_strf.extend_from_slice(&sample_rate.to_le_bytes());
    a_strf.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    a_strf.extend_from_slice(&2u16.to_le_bytes());
    a_strf.extend_from_slice(&16u16.to_le_bytes());
    a_strf.extend_from_slice(&0u16.to_le_bytes());

    let mut a_strl = Vec::new();
    a_strl.extend_from_slice(b"strl");
    a_strl.extend_from_slice(b"strh");
    a_strl.extend_from_slice(&(a_strh.len() as u32).to_le_bytes());
    a_strl.extend_from_slice(&a_strh);
    a_strl.extend_from_slice(b"strf");
    a_strl.extend_from_slice(&(a_strf.len() as u32).to_le_bytes());
    a_strl.extend_from_slice(&a_strf);

    let mut hdrl = Vec::new();
    hdrl.extend_from_slice(b"hdrl");
    hdrl.extend_from_slice(b"avih");
    hdrl.extend_from_slice(&(avih.len() as u32).to_le_bytes());
    hdrl.extend_from_slice(&avih);
    hdrl.extend_from_slice(b"LIST");
    hdrl.extend_from_slice(&(v_strl.len() as u32).to_le_bytes());
    hdrl.extend_from_slice(&v_strl);
    hdrl.extend_from_slice(b"LIST");
    hdrl.extend_from_slice(&(a_strl.len() as u32).to_le_bytes());
    hdrl.extend_from_slice(&a_strl);

    let mut riff_body = Vec::new();
    riff_body.extend_from_slice(b"AVI ");
    riff_body.extend_from_slice(b"LIST");
    riff_body.extend_from_slice(&(hdrl.len() as u32).to_le_bytes());
    riff_body.extend_from_slice(&hdrl);
    riff_body.extend_from_slice(b"LIST");
    riff_body.extend_from_slice(&(movi.len() as u32).to_le_bytes());
    riff_body.extend_from_slice(&movi);

    let mut out = Vec::with_capacity(8 + riff_body.len());
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(riff_body.len() as u32).to_le_bytes());
    out.extend_from_slice(&riff_body);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_req() -> VideoMuxRequest {
        let base = PathBuf::from("/tmp/solanostickers/prj_1");
        let tmp = PathBuf::from("/tmp/solanostickers/export_tmp");
        VideoMuxRequest {
            project_dir: base.clone(),
            temp_dir: tmp.clone(),
            sticker_path: base.join("assets/stk_a.png"),
            sticker_animated_gif: false,
            audio_path: base.join("assets/aud_b.mp3"),
            output_path: tmp.join("out.mp4"),
            start_ms: 500,
            end_ms: 10_500,
            matte_color_hex: "#FFFFFF".to_string(),
            profile: VideoContainerProfile::VideoMp4,
        }
    }

    #[test]
    fn builds_structured_mp4_args_for_10s_clip() {
        let req = sample_req();
        let args = build_ffmpeg_args(&req).unwrap();
        assert!(args.contains(&"-loop".to_string()));
        assert!(args.contains(&"0.500".to_string()));
        assert!(args.contains(&"10.000".to_string()));
        assert!(args.contains(&"+faststart".to_string()));
        assert!(args
            .iter()
            .any(|a| a.contains("yuv420p") && a.contains("0xFFFFFF")));
    }

    #[test]
    fn builds_looping_gif_and_webm_alpha_args() {
        let mut req = sample_req();
        req.sticker_animated_gif = true;
        req.profile = VideoContainerProfile::VideoWebmAlpha;
        req.output_path = req.temp_dir.join("out.webm");
        let args = build_ffmpeg_args(&req).unwrap();
        assert!(args.contains(&"-stream_loop".to_string()));
        assert!(args.contains(&"libvpx-vp9".to_string()));
        assert!(args.contains(&"yuva420p".to_string()));
        assert!(args.contains(&"libopus".to_string()));
    }

    #[test]
    fn rejects_over_10s_and_path_traversal() {
        let mut over = sample_req();
        over.end_ms = 11_500;
        assert!(matches!(
            build_ffmpeg_args(&over),
            Err(VideoEngineError::TrimOutOfRange(_))
        ));

        let mut traversal = sample_req();
        traversal.sticker_path = traversal.project_dir.join("../secret.png");
        assert!(matches!(
            build_ffmpeg_args(&traversal),
            Err(VideoEngineError::UnsafePath(_))
        ));
    }

    #[test]
    fn test_self_contained_video_10s_without_ffmpeg() {
        let frame = vec![255u8; 64 * 64 * 4];
        let pcm = vec![1000i16; 48000 * 10];
        let avi = encode_self_contained_video(&[(&frame, 500)], &pcm, 48000, 64, 64).unwrap();
        assert!(avi.starts_with(b"RIFF"));
        assert_eq!(&avi[8..12], b"AVI ");
        assert!(avi.windows(4).any(|w| w == b"vids"));
        assert!(avi.windows(4).any(|w| w == b"auds"));
    }
}
