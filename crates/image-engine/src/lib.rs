use std::io::Cursor;
use std::num::NonZeroU64;

use gif::{DecodeOptions, DisposalMethod, MemoryLimit};
use image::imageops::FilterType;
use image::{DynamicImage, ImageDecoder, ImageFormat as ImgFmt, ImageReader, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const MAX_IMAGE_BYTES: usize = 20 * 1024 * 1024;
pub const MAX_IMAGE_DIMENSION: u32 = 8000;
pub const MAX_IMAGE_PIXELS: u64 = 40_000_000;
pub const MAX_GIF_FRAMES: usize = 300;
pub const MAX_GIF_CANVAS_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_GIF_FRAME_PIXELS: u64 = 16_777_216;
pub const MAX_GIF_DECODED_BYTES: u64 = 256 * 1024 * 1024;
pub const STICKER_SIZE_PX: u32 = 512;
pub const MAX_STATIC_WEBP_BYTES: usize = 102_400;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StickerImageFormat {
    Png,
    Jpeg,
    Webp,
    Gif,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageProbeInfo {
    pub format: StickerImageFormat,
    pub width: u32,
    pub height: u32,
    pub has_alpha: bool,
    pub animated: bool,
    pub frame_count: usize,
    pub duration_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GifFrame512 {
    pub delay_ms: u32,
    pub rgba: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum ImageEngineError {
    #[error("unsupported-image: {0}")]
    UnsupportedImage(String),
    #[error("file-too-large: {0}")]
    FileTooLarge(String),
    #[error("sticker-too-large: {0}")]
    StickerTooLarge(String),
}

pub fn sniff_image_format(bytes: &[u8]) -> Result<StickerImageFormat, ImageEngineError> {
    if bytes.len() < 12 {
        return Err(ImageEngineError::UnsupportedImage(
            "Image header too short".to_string(),
        ));
    }
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Ok(StickerImageFormat::Png);
    }
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Ok(StickerImageFormat::Jpeg);
    }
    if bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return Ok(StickerImageFormat::Webp);
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Ok(StickerImageFormat::Gif);
    }
    Err(ImageEngineError::UnsupportedImage(
        "Unrecognized image magic bytes".to_string(),
    ))
}

fn check_dimension_limits(width: u32, height: u32) -> Result<(), ImageEngineError> {
    if width == 0 || height == 0 {
        return Err(ImageEngineError::UnsupportedImage(
            "Image has zero dimension".to_string(),
        ));
    }
    if width > MAX_IMAGE_DIMENSION || height > MAX_IMAGE_DIMENSION {
        return Err(ImageEngineError::FileTooLarge(format!(
            "Image dimension {width}x{height} exceeds {MAX_IMAGE_DIMENSION}px limit"
        )));
    }
    let pixels = u64::from(width) * u64::from(height);
    if pixels > MAX_IMAGE_PIXELS {
        return Err(ImageEngineError::FileTooLarge(format!(
            "Image pixel count {pixels} exceeds {MAX_IMAGE_PIXELS} limit"
        )));
    }
    Ok(())
}

fn gif_decoder_options() -> DecodeOptions {
    let mut opts = DecodeOptions::new();
    opts.set_color_output(gif::ColorOutput::RGBA);
    opts.set_memory_limit(MemoryLimit::Bytes(
        NonZeroU64::new(MAX_GIF_CANVAS_BYTES).expect("GIF canvas limit is non-zero"),
    ));
    opts
}

fn gif_canvas_bytes(width: u32, height: u32) -> Result<u64, ImageEngineError> {
    check_dimension_limits(width, height)?;
    let pixels = u64::from(width) * u64::from(height);
    if pixels > MAX_GIF_FRAME_PIXELS {
        return Err(ImageEngineError::FileTooLarge(format!(
            "GIF canvas {width}x{height} exceeds {MAX_GIF_FRAME_PIXELS} pixel GIF limit"
        )));
    }
    let bytes = pixels
        .checked_mul(4)
        .ok_or_else(|| ImageEngineError::FileTooLarge("GIF canvas size overflow".to_string()))?;
    if bytes > MAX_GIF_CANVAS_BYTES {
        return Err(ImageEngineError::FileTooLarge(format!(
            "GIF canvas allocation of {bytes} bytes exceeds {MAX_GIF_CANVAS_BYTES} byte GIF limit"
        )));
    }
    Ok(bytes)
}

fn gif_frame_pixels(
    canvas_width: u32,
    canvas_height: u32,
    left: u32,
    top: u32,
    width: u32,
    height: u32,
) -> Result<u64, ImageEngineError> {
    if width == 0 || height == 0 {
        return Err(ImageEngineError::UnsupportedImage(
            "GIF frame has zero dimensions".to_string(),
        ));
    }
    if width > MAX_IMAGE_DIMENSION || height > MAX_IMAGE_DIMENSION {
        return Err(ImageEngineError::FileTooLarge(format!(
            "GIF frame dimension {width}x{height} exceeds {MAX_IMAGE_DIMENSION}px limit"
        )));
    }
    let pixels = u64::from(width) * u64::from(height);
    if pixels > MAX_GIF_FRAME_PIXELS {
        return Err(ImageEngineError::FileTooLarge(format!(
            "GIF frame {width}x{height} exceeds {MAX_GIF_FRAME_PIXELS} pixel GIF limit"
        )));
    }
    if u64::from(left) + u64::from(width) > u64::from(canvas_width)
        || u64::from(top) + u64::from(height) > u64::from(canvas_height)
    {
        return Err(ImageEngineError::UnsupportedImage(format!(
            "GIF frame bounds ({left},{top} {width}x{height}) exceed the {canvas_width}x{canvas_height} canvas"
        )));
    }
    Ok(pixels)
}

fn gif_rgba_bytes(pixels: u64) -> Result<u64, ImageEngineError> {
    pixels
        .checked_mul(4)
        .ok_or_else(|| ImageEngineError::FileTooLarge("GIF RGBA size overflow".to_string()))
}

fn check_gif_decoded_memory(
    retained_bytes: u64,
    next_retained_bytes: u64,
    working_bytes: u64,
) -> Result<(), ImageEngineError> {
    let total = retained_bytes
        .checked_add(next_retained_bytes)
        .and_then(|value| value.checked_add(working_bytes))
        .ok_or_else(|| {
            ImageEngineError::FileTooLarge("GIF decoded memory size overflow".to_string())
        })?;
    if total > MAX_GIF_DECODED_BYTES {
        return Err(ImageEngineError::FileTooLarge(format!(
            "GIF decoded RGBA memory {total} exceeds {MAX_GIF_DECODED_BYTES} byte limit"
        )));
    }
    Ok(())
}

fn alloc_zeroed_vec(len: usize) -> Result<Vec<u8>, ImageEngineError> {
    let mut buffer = Vec::new();
    buffer.try_reserve_exact(len).map_err(|_| {
        ImageEngineError::FileTooLarge(format!("GIF allocation of {len} bytes failed"))
    })?;
    buffer.resize(len, 0);
    Ok(buffer)
}

fn alloc_rgba_image(width: u32, height: u32) -> Result<RgbaImage, ImageEngineError> {
    let pixels = u64::from(width) * u64::from(height);
    let bytes = gif_rgba_bytes(pixels)?;
    let len = usize::try_from(bytes).map_err(|_| {
        ImageEngineError::FileTooLarge("GIF canvas size is not addressable".to_string())
    })?;
    let buffer = alloc_zeroed_vec(len)?;
    RgbaImage::from_raw(width, height, buffer)
        .ok_or_else(|| ImageEngineError::UnsupportedImage("Invalid RGBA canvas".to_string()))
}

fn snapshot_region(
    image: &RgbaImage,
    left: u32,
    top: u32,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, ImageEngineError> {
    let row_bytes = usize::try_from(gif_rgba_bytes(u64::from(width))?).map_err(|_| {
        ImageEngineError::FileTooLarge("GIF disposal row is not addressable".to_string())
    })?;
    let total = row_bytes
        .checked_mul(usize::try_from(height).map_err(|_| {
            ImageEngineError::FileTooLarge("GIF disposal region is not addressable".to_string())
        })?)
        .ok_or_else(|| {
            ImageEngineError::FileTooLarge("GIF disposal region size overflow".to_string())
        })?;
    let mut region = alloc_zeroed_vec(total)?;
    let raw = image.as_raw();
    let stride = usize::try_from(image.width()).map_err(|_| {
        ImageEngineError::FileTooLarge("GIF canvas stride is not addressable".to_string())
    })?;
    for y in 0..usize::try_from(height).map_err(|_| {
        ImageEngineError::FileTooLarge("GIF disposal region is not addressable".to_string())
    })? {
        let src = ((usize::try_from(top).map_err(|_| {
            ImageEngineError::FileTooLarge("GIF disposal offset is not addressable".to_string())
        })? + y)
            * stride
            + usize::try_from(left).map_err(|_| {
                ImageEngineError::FileTooLarge("GIF disposal offset is not addressable".to_string())
            })?)
            * 4;
        let dst = y * row_bytes;
        region[dst..dst + row_bytes].copy_from_slice(&raw[src..src + row_bytes]);
    }
    Ok(region)
}

fn restore_region(
    image: &mut RgbaImage,
    region: &[u8],
    left: u32,
    top: u32,
    width: u32,
    height: u32,
) {
    let row_bytes = width as usize * 4;
    let stride = image.width() as usize;
    for y in 0..height as usize {
        let dst = ((top as usize + y) * stride + left as usize) * 4;
        let src = y * row_bytes;
        image.as_mut()[dst..dst + row_bytes].copy_from_slice(&region[src..src + row_bytes]);
    }
}

fn webp_is_animated(bytes: &[u8]) -> bool {
    if bytes.len() >= 21 && &bytes[12..16] == b"VP8X" {
        let flags = bytes[20];
        return (flags & 0x02) != 0;
    }
    false
}

fn inspect_gif(bytes: &[u8]) -> Result<(u32, u32, bool, usize, u32), ImageEngineError> {
    let opts = gif_decoder_options();
    let mut decoder = opts
        .read_info(Cursor::new(bytes))
        .map_err(|e| ImageEngineError::UnsupportedImage(format!("Invalid GIF header: {e}")))?;

    let width = u32::from(decoder.width());
    let height = u32::from(decoder.height());
    let canvas_bytes = gif_canvas_bytes(width, height)?;

    let mut frame_count = 0usize;
    let mut total_ms = 0u32;
    let mut has_alpha = false;

    loop {
        let info = match decoder
            .next_frame_info()
            .map_err(|e| ImageEngineError::UnsupportedImage(format!("Corrupt GIF frame: {e}")))?
        {
            Some(frame) => frame,
            None => break,
        };
        if frame_count >= MAX_GIF_FRAMES {
            return Err(ImageEngineError::FileTooLarge(format!(
                "GIF exceeds the {MAX_GIF_FRAMES} frame limit"
            )));
        }
        let _ = gif_frame_pixels(
            width,
            height,
            u32::from(info.left),
            u32::from(info.top),
            u32::from(info.width),
            u32::from(info.height),
        )?;
        frame_count += 1;
        let delay_cs = if info.delay == 0 { 10 } else { info.delay };
        total_ms = total_ms.saturating_add(u32::from(delay_cs) * 10);
        if !has_alpha && info.transparent.is_some() {
            has_alpha = true;
        }
    }

    if frame_count == 0 {
        return Err(ImageEngineError::UnsupportedImage(
            "GIF contains zero frames".to_string(),
        ));
    }

    check_export_gif_decoded_memory(bytes, width, height, canvas_bytes)?;

    Ok((width, height, has_alpha, frame_count, total_ms))
}

/// Transient bytes the TypeScript export decoder (`decodeGifFramesRgba` in
/// `packages/media-web`) holds while it decodes one frame. Mirrors
/// `gifFrameWorkingBytes` in `packages/media-web/src/gif-limits.ts`: working
/// canvas, joined LZW data, LZW index output (doubled when interlaced) and the
/// saved region for `DisposalMethod::Previous`.
fn export_gif_frame_working_bytes(
    canvas_bytes: u64,
    lzw_data_bytes: u64,
    frame_pixels: u64,
    interlaced: bool,
    dispose: DisposalMethod,
) -> u64 {
    let index_bytes = frame_pixels * if interlaced { 2 } else { 1 };
    let previous_bytes = if dispose == DisposalMethod::Previous { frame_pixels * 4 } else { 0 };
    canvas_bytes + lzw_data_bytes + index_bytes + previous_bytes
}

/// The desktop app probes images with this crate but exports with the
/// TypeScript decoder, which retains one canvas-sized RGBA frame per frame.
/// Applying that budget here makes a GIF that passes the probe also pass
/// export. Call only after every frame descriptor has been validated, because
/// reading compressed frame data reserves a buffer sized from the descriptor.
fn check_export_gif_decoded_memory(
    bytes: &[u8],
    width: u32,
    height: u32,
    canvas_bytes: u64,
) -> Result<(), ImageEngineError> {
    let mut opts = gif_decoder_options();
    opts.skip_frame_decoding(true);
    let mut decoder = opts
        .read_info(Cursor::new(bytes))
        .map_err(|e| ImageEngineError::UnsupportedImage(format!("Invalid GIF header: {e}")))?;
    let mut retained_bytes = 0u64;
    while let Some(frame) = decoder
        .read_next_frame()
        .map_err(|e| ImageEngineError::UnsupportedImage(format!("Corrupt GIF frame: {e}")))?
    {
        let frame_pixels = gif_frame_pixels(
            width,
            height,
            u32::from(frame.left),
            u32::from(frame.top),
            u32::from(frame.width),
            u32::from(frame.height),
        )?;
        // With skip_frame_decoding the buffer holds the LZW minimum code size
        // byte followed by the compressed data.
        let lzw_data_bytes = frame.buffer.len().saturating_sub(1) as u64;
        let working_bytes = export_gif_frame_working_bytes(
            canvas_bytes,
            lzw_data_bytes,
            frame_pixels,
            frame.interlaced,
            frame.dispose,
        );
        check_gif_decoded_memory(retained_bytes, canvas_bytes, working_bytes)?;
        retained_bytes += canvas_bytes;
    }
    Ok(())
}

pub fn probe_image(bytes: &[u8]) -> Result<ImageProbeInfo, ImageEngineError> {
    if bytes.is_empty() {
        return Err(ImageEngineError::UnsupportedImage("Empty image".to_string()));
    }
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(ImageEngineError::FileTooLarge(
            "Image exceeds 20 MB limit".to_string(),
        ));
    }

    let format = sniff_image_format(bytes)?;
    if format == StickerImageFormat::Png && bytes.len() >= 24 && &bytes[12..16] == b"IHDR" {
        let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        check_dimension_limits(w, h)?;
    }
    if format == StickerImageFormat::Gif {
        let (width, height, has_alpha, frame_count, duration_ms) = inspect_gif(bytes)?;
        return Ok(ImageProbeInfo {
            format,
            width,
            height,
            has_alpha,
            animated: frame_count > 1,
            frame_count,
            duration_ms: if frame_count > 1 { duration_ms } else { 0 },
        });
    }

    let img_fmt = match format {
        StickerImageFormat::Png => ImgFmt::Png,
        StickerImageFormat::Jpeg => ImgFmt::Jpeg,
        StickerImageFormat::Webp => ImgFmt::WebP,
        StickerImageFormat::Gif => unreachable!(),
    };

    let reader = ImageReader::with_format(Cursor::new(bytes), img_fmt);
    let decoder = reader
        .into_decoder()
        .map_err(|e| ImageEngineError::UnsupportedImage(format!("Decoder init failed: {e}")))?;

    let (width, height) = decoder.dimensions();
    check_dimension_limits(width, height)?;

    let color_type = decoder.color_type();
    let img = DynamicImage::from_decoder(decoder)
        .map_err(|e| ImageEngineError::UnsupportedImage(format!("Decode failed: {e}")))?;
    let rgba = img.to_rgba8();
    let has_alpha = color_type.has_alpha() || rgba.pixels().any(|p| p[3] < 255);
    let animated = format == StickerImageFormat::Webp && webp_is_animated(bytes);

    Ok(ImageProbeInfo {
        format,
        width,
        height,
        has_alpha,
        animated,
        frame_count: 1,
        duration_ms: 0,
    })
}

/// Computes a 1:1 square crop anchored at normalized (crop_x, crop_y) in [0.0, 1.0].
/// Output resolution is determined solely by the input's smaller side, capped at 512x512.
pub fn compute_square_crop(src_w: u32, src_h: u32, crop_x: f32, crop_y: f32) -> (u32, u32, u32, u32) {
    let crop_side = src_w.min(src_h).max(1);
    let raw_out = crop_side.min(STICKER_SIZE_PX);
    let output_size = if raw_out <= 2 { raw_out } else { raw_out - (raw_out % 2) };
    let nx = if crop_x.is_finite() { crop_x.clamp(0.0, 1.0) } else { 0.5 };
    let ny = if crop_y.is_finite() { crop_y.clamp(0.0, 1.0) } else { 0.5 };
    let src_x = ((src_w.saturating_sub(crop_side)) as f32 * nx).round() as u32;
    let src_y = ((src_h.saturating_sub(crop_side)) as f32 * ny).round() as u32;
    (src_x, src_y, crop_side, output_size)
}

/// Crops a 1:1 square from `src` at `(crop_x, crop_y)` and scales to `output_size x output_size` (<= 512).
pub fn crop_square_rgba_1x1(src: &RgbaImage, crop_x: f32, crop_y: f32) -> RgbaImage {
    let (w, h) = src.dimensions();
    let (src_x, src_y, crop_side, output_size) = compute_square_crop(w, h, crop_x, crop_y);
    let cropped = image::imageops::crop_imm(src, src_x, src_y, crop_side, crop_side).to_image();
    if crop_side == output_size {
        cropped
    } else {
        image::imageops::resize(&cropped, output_size, output_size, FilterType::Lanczos3)
    }
}

/// Scales an RGBA image proportionally so its longest side is <= 512 and centers it
/// on a 512x512 transparent RGBA canvas without premultiplication fringing.
pub fn contain_fit_rgba_512(src: &RgbaImage) -> RgbaImage {
    let (w, h) = src.dimensions();
    let scale = (STICKER_SIZE_PX as f64 / w as f64).min(STICKER_SIZE_PX as f64 / h as f64);
    let target_w = ((w as f64 * scale).round() as u32).clamp(1, STICKER_SIZE_PX);
    let target_h = ((h as f64 * scale).round() as u32).clamp(1, STICKER_SIZE_PX);

    let resized = if target_w == w && target_h == h {
        src.clone()
    } else {
        image::imageops::resize(src, target_w, target_h, FilterType::Lanczos3)
    };

    let mut canvas = RgbaImage::from_pixel(STICKER_SIZE_PX, STICKER_SIZE_PX, Rgba([0, 0, 0, 0]));
    let offset_x = (STICKER_SIZE_PX - target_w) / 2;
    let offset_y = (STICKER_SIZE_PX - target_h) / 2;
    image::imageops::overlay(&mut canvas, &resized, i64::from(offset_x), i64::from(offset_y));
    canvas
}

pub fn render_sticker_512(bytes: &[u8]) -> Result<RgbaImage, ImageEngineError> {
    let info = probe_image(bytes)?;
    if info.format == StickerImageFormat::Gif {
        let frames = decode_gif_frames_512_inner(bytes, 1, false)?;
        let first = frames
            .into_iter()
            .next()
            .ok_or_else(|| ImageEngineError::UnsupportedImage("No GIF frames".to_string()))?;
        return RgbaImage::from_raw(STICKER_SIZE_PX, STICKER_SIZE_PX, first.rgba)
            .ok_or_else(|| ImageEngineError::UnsupportedImage("Invalid RGBA frame".to_string()));
    }

    let img_fmt = match info.format {
        StickerImageFormat::Png => ImgFmt::Png,
        StickerImageFormat::Jpeg => ImgFmt::Jpeg,
        StickerImageFormat::Webp => ImgFmt::WebP,
        StickerImageFormat::Gif => unreachable!(),
    };
    let decoded = ImageReader::with_format(Cursor::new(bytes), img_fmt)
        .decode()
        .map_err(|e| ImageEngineError::UnsupportedImage(format!("Decode failed: {e}")))?;
    Ok(contain_fit_rgba_512(&decoded.to_rgba8()))
}

/// Decodes all GIF frames with disposal methods applied so transparency is preserved
/// across frames, and contain-fits every frame onto 512x512 RGBA.
pub fn decode_gif_frames_512(bytes: &[u8]) -> Result<Vec<GifFrame512>, ImageEngineError> {
    decode_gif_frames_512_inner(bytes, MAX_GIF_FRAMES, true)
}

fn decode_gif_frames_512_inner(
    bytes: &[u8],
    max_frames: usize,
    reject_extra: bool,
) -> Result<Vec<GifFrame512>, ImageEngineError> {
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(ImageEngineError::FileTooLarge("GIF exceeds 20 MB limit".to_string()));
    }
    let opts = gif_decoder_options();
    let mut decoder = opts
        .read_info(Cursor::new(bytes))
        .map_err(|e| ImageEngineError::UnsupportedImage(format!("Invalid GIF: {e}")))?;
    let canvas_w = u32::from(decoder.width());
    let canvas_h = u32::from(decoder.height());
    let canvas_bytes = gif_canvas_bytes(canvas_w, canvas_h)?;
    let mut current_canvas = alloc_rgba_image(canvas_w, canvas_h)?;
    let output_bytes = u64::from(STICKER_SIZE_PX) * u64::from(STICKER_SIZE_PX) * 4;
    let mut retained_out_bytes = 0u64;
    let mut out = Vec::new();
    loop {
        let info = match decoder
            .next_frame_info()
            .map_err(|e| ImageEngineError::UnsupportedImage(format!("Corrupt GIF: {e}")))?
        {
            Some(frame) => frame,
            None => break,
        };
        if out.len() >= max_frames {
            if reject_extra {
                return Err(ImageEngineError::FileTooLarge(format!(
                    "GIF exceeds the {MAX_GIF_FRAMES} frame limit"
                )));
            }
            break;
        }
        let frame_width = u32::from(info.width);
        let frame_height = u32::from(info.height);
        let frame_left = u32::from(info.left);
        let frame_top = u32::from(info.top);
        let frame_pixels =
            gif_frame_pixels(canvas_w, canvas_h, frame_left, frame_top, frame_width, frame_height)?;
        let frame_bytes = gif_rgba_bytes(frame_pixels)?;
        let needs_previous = info.dispose == DisposalMethod::Previous;
        let frame_dispose = info.dispose;
        let delay_cs = if info.delay == 0 { 10 } else { info.delay };
        let working_bytes = canvas_bytes
            + canvas_bytes
            + frame_bytes
            + if needs_previous { frame_bytes } else { 0 }
            + output_bytes * 2;
        check_gif_decoded_memory(retained_out_bytes, output_bytes, working_bytes)?;
        let buffer_size = decoder.buffer_size();
        let expected_buffer_size = usize::try_from(frame_bytes).map_err(|_| {
            ImageEngineError::FileTooLarge("GIF frame buffer is not addressable".to_string())
        })?;
        if buffer_size != expected_buffer_size {
            return Err(ImageEngineError::UnsupportedImage(
                "GIF frame buffer size mismatch".to_string(),
            ));
        }
        let mut frame_rgba = alloc_zeroed_vec(buffer_size)?;
        decoder
            .read_into_buffer(&mut frame_rgba)
            .map_err(|e| ImageEngineError::UnsupportedImage(format!("Corrupt GIF: {e}")))?;
        let prev_region = if needs_previous {
            Some(snapshot_region(&current_canvas, frame_left, frame_top, frame_width, frame_height)?)
        } else {
            None
        };
        for y in 0..frame_height {
            for x in 0..frame_width {
                let idx = ((y * frame_width + x) * 4) as usize;
                let a = frame_rgba[idx + 3];
                if a > 0 {
                    let cx = frame_left + x;
                    let cy = frame_top + y;
                    current_canvas.put_pixel(
                        cx,
                        cy,
                        Rgba([frame_rgba[idx], frame_rgba[idx + 1], frame_rgba[idx + 2], a]),
                    );
                }
            }
        }
        let fitted = contain_fit_rgba_512(&current_canvas);
        out.push(GifFrame512 {
            delay_ms: u32::from(delay_cs) * 10,
            rgba: fitted.into_raw(),
        });
        retained_out_bytes += output_bytes;
        match frame_dispose {
            DisposalMethod::Background => {
                for y in 0..frame_height {
                    for x in 0..frame_width {
                        let cx = frame_left + x;
                        let cy = frame_top + y;
                        current_canvas.put_pixel(cx, cy, Rgba([0, 0, 0, 0]));
                    }
                }
            }
            DisposalMethod::Previous => {
                if let Some(region) = prev_region.as_deref() {
                    restore_region(&mut current_canvas, region, frame_left, frame_top, frame_width, frame_height);
                }
            }
            DisposalMethod::Any | DisposalMethod::Keep => {}
        }
    }
    if out.is_empty() {
        return Err(ImageEngineError::UnsupportedImage(
            "GIF contained zero frames".to_string(),
        ));
    }
    Ok(out)
}

/// Straight-alpha composite onto a solid RGB matte color (used when producing H.264 MP4).
pub fn composite_on_matte(rgba: &RgbaImage, matte_rgb: [u8; 3]) -> RgbaImage {
    let mut out = RgbaImage::new(rgba.width(), rgba.height());
    for (x, y, px) in rgba.enumerate_pixels() {
        let a = f32::from(px[3]) / 255.0;
        let inv = 1.0 - a;
        let r = (f32::from(px[0]) * a + f32::from(matte_rgb[0]) * inv).round() as u8;
        let g = (f32::from(px[1]) * a + f32::from(matte_rgb[1]) * inv).round() as u8;
        let b = (f32::from(px[2]) * a + f32::from(matte_rgb[2]) * inv).round() as u8;
        out.put_pixel(x, y, Rgba([r, g, b, 255]));
    }
    out
}

pub fn parse_hex_rgb(hex: &str) -> [u8; 3] {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&clean[0..2], 16),
            u8::from_str_radix(&clean[2..4], 16),
            u8::from_str_radix(&clean[4..6], 16),
        ) {
            return [r, g, b];
        }
    }
    [255, 255, 255]
}

fn quantize_rgba_step(img: &RgbaImage, step: u8) -> RgbaImage {
    if step <= 1 {
        return img.clone();
    }
    let mut out = img.clone();
    for px in out.pixels_mut() {
        if px[3] == 0 {
            px.0 = [0, 0, 0, 0];
            continue;
        }
        px[0] = (px[0] / step) * step;
        px[1] = (px[1] / step) * step;
        px[2] = (px[2] / step) * step;
        px[3] = (px[3] / step) * step;
    }
    out
}

/// Encodes a 512x512 RGBA image to WebP preserving alpha and staying <= 100 KB.
/// Tries lossless first, then steps down palette/channel quantization until <= 102,400 bytes.
pub fn encode_webp_under_100kb(frame: &RgbaImage) -> Result<Vec<u8>, ImageEngineError> {
    if frame.width() != STICKER_SIZE_PX || frame.height() != STICKER_SIZE_PX {
        return Err(ImageEngineError::UnsupportedImage(
            "WebP frame must be 512x512".to_string(),
        ));
    }

    for step in [1u8, 4, 8, 16, 32] {
        let candidate = quantize_rgba_step(frame, step);
        let mut buf = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(candidate)
            .write_to(&mut buf, ImgFmt::WebP)
            .map_err(|e| ImageEngineError::UnsupportedImage(format!("WebP encode failed: {e}")))?;
        let bytes = buf.into_inner();
        if bytes.len() <= MAX_STATIC_WEBP_BYTES {
            return Ok(bytes);
        }
    }

    Err(ImageEngineError::StickerTooLarge(
        "Could not compress WebP under 100 KB".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gif::{Encoder, Frame, Repeat};

    fn make_transparent_png(w: u32, h: u32) -> Vec<u8> {
        let mut img = RgbaImage::from_pixel(w, h, Rgba([0, 0, 0, 0]));
        for y in (h / 4)..(3 * h / 4) {
            for x in (w / 4)..(3 * w / 4) {
                img.put_pixel(x, y, Rgba([255, 64, 32, 255]));
            }
        }
        let mut buf = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(img)
            .write_to(&mut buf, ImgFmt::Png)
            .unwrap();
        buf.into_inner()
    }

    fn make_animated_transparent_gif() -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let palette = &[0, 0, 0, 255, 0, 0, 0, 255, 0];
            let mut encoder = Encoder::new(&mut buf, 64, 64, palette).unwrap();
            encoder.set_repeat(Repeat::Infinite).unwrap();
            for idx in 0..2 {
                let mut pixels = vec![0u8; 64 * 64];
                for y in 16..48 {
                    for x in 16..48 {
                        pixels[y * 64 + x] = (idx + 1) as u8;
                    }
                }
                let mut frame = Frame::from_indexed_pixels(64, 64, pixels, Some(0));
                frame.delay = 50; // 500ms
                frame.dispose = DisposalMethod::Background;
                encoder.write_frame(&frame).unwrap();
            }
        }
        buf
    }

    #[test]
    fn tc001_png_preserves_alpha_and_encodes_webp_under_100kb() {
        let png = make_transparent_png(640, 480);
        let probe = probe_image(&png).unwrap();
        assert_eq!(probe.format, StickerImageFormat::Png);
        assert!(probe.has_alpha);
        assert!(!probe.animated);

        let fitted = render_sticker_512(&png).unwrap();
        assert_eq!(fitted.dimensions(), (512, 512));
        // Corner pixel must remain fully transparent
        assert_eq!(fitted.get_pixel(0, 0)[3], 0);
        // Center pixel must remain opaque
        assert_eq!(fitted.get_pixel(256, 256)[3], 255);

        let webp = encode_webp_under_100kb(&fitted).unwrap();
        assert!(webp.len() <= MAX_STATIC_WEBP_BYTES);

        let re_probe = probe_image(&webp).unwrap();
        assert_eq!(re_probe.format, StickerImageFormat::Webp);
        assert_eq!((re_probe.width, re_probe.height), (512, 512));
        assert!(re_probe.has_alpha);
    }

    #[test]
    fn gif_animated_preserves_alpha_and_frame_durations() {
        let gif_bytes = make_animated_transparent_gif();
        let probe = probe_image(&gif_bytes).unwrap();
        assert_eq!(probe.format, StickerImageFormat::Gif);
        assert!(probe.animated);
        assert!(probe.has_alpha);
        assert_eq!(probe.frame_count, 2);
        assert_eq!(probe.duration_ms, 1000);

        let frames = decode_gif_frames_512(&gif_bytes).unwrap();
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].delay_ms, 500);
        // Corner alpha is 0 (transparent)
        assert_eq!(frames[0].rgba[3], 0);
    }

    #[test]
    fn matte_composition_replaces_transparent_pixels_cleanly() {
        let png = make_transparent_png(128, 128);
        let fitted = render_sticker_512(&png).unwrap();
        let on_white = composite_on_matte(&fitted, parse_hex_rgb("#FFFFFF"));
        assert_eq!(on_white.get_pixel(0, 0).0, [255, 255, 255, 255]);
        assert_eq!(on_white.get_pixel(256, 256).0, [255, 64, 32, 255]);
    }

    #[test]
    fn sec009_and_tc006_reject_bomb_and_corrupt_bytes() {
        // Fake PNG IHDR claiming 9000x9000 pixels
        let mut fake_png = vec![
            0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, // signature
            0x00, 0x00, 0x00, 0x0D, b'I', b'H', b'D', b'R', // IHDR len + tag
        ];
        fake_png.extend_from_slice(&9000u32.to_be_bytes());
        fake_png.extend_from_slice(&9000u32.to_be_bytes());
        fake_png.extend_from_slice(&[8, 6, 0, 0, 0, 0, 0, 0, 0]);
        let err = probe_image(&fake_png).unwrap_err();
        assert!(matches!(err, ImageEngineError::FileTooLarge(_)));

        let corrupt = b"not an image at all";
        let err2 = probe_image(corrupt).unwrap_err();
        assert!(matches!(err2, ImageEngineError::UnsupportedImage(_)));
    }

    #[test]
    fn crops_1x1_center_and_hover_pan_preserving_input_resolution() {
        let mut wide = RgbaImage::from_pixel(800, 400, Rgba([0, 0, 0, 0]));
        // Put blue pixel at center (400, 200) and red pixel at right edge (750, 200)
        wide.put_pixel(400, 200, Rgba([0, 0, 255, 255]));
        wide.put_pixel(750, 200, Rgba([255, 0, 0, 255]));

        let centered = crop_square_rgba_1x1(&wide, 0.5, 0.5);
        assert_eq!(centered.dimensions(), (400, 400));
        assert_eq!(centered.get_pixel(200, 200).0, [0, 0, 255, 255]);

        let panned_right = crop_square_rgba_1x1(&wide, 1.0, 0.5);
        assert_eq!(panned_right.dimensions(), (400, 400));
        assert_eq!(panned_right.get_pixel(350, 200).0, [255, 0, 0, 255]);
    }
    fn make_raw_gif(canvas_w: u32, canvas_h: u32, frame_w: u32, frame_h: u32, left: u32, top: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"GIF89a");
        bytes.extend_from_slice(&(canvas_w as u16).to_le_bytes());
        bytes.extend_from_slice(&(canvas_h as u16).to_le_bytes());
        bytes.push(0x91);
        bytes.push(0);
        bytes.push(0);
        bytes.extend_from_slice(&[0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255]);
        bytes.push(0x2c);
        bytes.extend_from_slice(&(left as u16).to_le_bytes());
        bytes.extend_from_slice(&(top as u16).to_le_bytes());
        bytes.extend_from_slice(&(frame_w as u16).to_le_bytes());
        bytes.extend_from_slice(&(frame_h as u16).to_le_bytes());
        bytes.push(0);
        bytes.push(2);
        bytes.push(3);
        bytes.extend_from_slice(&[0x04, 0x80, 0x02]);
        bytes.push(0);
        bytes.push(0x3b);
        bytes
    }

    fn make_many_frame_gif(frame_count: usize, width: u32, height: u32) -> Vec<u8> {
        make_many_frame_gif_with_dispose(frame_count, width, height, DisposalMethod::Keep)
    }

    fn make_many_frame_gif_with_dispose(
        frame_count: usize,
        width: u32,
        height: u32,
        dispose: DisposalMethod,
    ) -> Vec<u8> {
        let mut buf = Vec::new();
        let palette = &[0u8, 0, 0, 255, 0, 0, 0, 255, 0];
        {
            let mut encoder = Encoder::new(&mut buf, width as u16, height as u16, palette).unwrap();
            encoder.set_repeat(Repeat::Infinite).unwrap();
            for _ in 0..frame_count {
                let pixels = vec![0u8; (width * height) as usize];
                let mut frame = Frame::from_indexed_pixels(width as u16, height as u16, pixels, Some(0));
                frame.delay = 10;
                frame.dispose = dispose;
                encoder.write_frame(&frame).unwrap();
            }
        }
        buf
    }

    #[test]
    fn gif_rejects_excessive_frame_descriptor_on_inspection() {
        let gif = make_raw_gif(2, 2, 65535, 65535, 0, 0);
        let err = probe_image(&gif).unwrap_err();
        assert!(matches!(err, ImageEngineError::FileTooLarge(_)));
    }

    #[test]
    fn gif_rejects_excessive_frame_descriptor_on_decode() {
        let gif = make_raw_gif(2, 2, 65535, 65535, 0, 0);
        let err = decode_gif_frames_512(&gif).unwrap_err();
        assert!(matches!(err, ImageEngineError::FileTooLarge(_)));
    }

    #[test]
    fn gif_rejects_too_many_frames_on_inspection() {
        let gif = make_many_frame_gif(MAX_GIF_FRAMES + 1, 1, 1);
        let err = probe_image(&gif).unwrap_err();
        assert!(matches!(err, ImageEngineError::FileTooLarge(_)));
    }

    #[test]
    fn gif_rejects_too_many_frames_on_decode() {
        let gif = make_many_frame_gif(MAX_GIF_FRAMES + 1, 1, 1);
        let err = decode_gif_frames_512(&gif).unwrap_err();
        assert!(matches!(err, ImageEngineError::FileTooLarge(_)));
    }

    #[test]
    fn gif_rejects_frame_descriptor_beyond_canvas() {
        let gif = make_raw_gif(2, 2, 2, 2, 1, 1);
        let err = probe_image(&gif).unwrap_err();
        assert!(matches!(err, ImageEngineError::UnsupportedImage(_)));
        let err = decode_gif_frames_512(&gif).unwrap_err();
        assert!(matches!(err, ImageEngineError::UnsupportedImage(_)));
    }

    #[test]
    fn gif_rejects_aggregate_decoded_memory_exhaustion() {
        let gif = make_many_frame_gif(MAX_GIF_FRAMES, 512, 512);
        let err = decode_gif_frames_512(&gif).unwrap_err();
        assert!(matches!(err, ImageEngineError::FileTooLarge(_)));
        let err = probe_image(&gif).unwrap_err();
        assert!(matches!(err, ImageEngineError::FileTooLarge(_)));
    }

    // The desktop app probes with this crate but exports with the TypeScript
    // decoder, which retains one canvas-sized RGBA frame (1 MiB at 512x512) per
    // frame. The 256 MiB budget fits 254 plain frames, or 253 when each frame
    // keeps a disposal buffer. These counts match packages/media-web tests.
    #[test]
    fn gif_probe_applies_export_decoded_memory_budget() {
        let cases = [
            (254, DisposalMethod::Keep, true),
            (255, DisposalMethod::Keep, false),
            (253, DisposalMethod::Previous, true),
            (254, DisposalMethod::Previous, false),
        ];
        for (count, dispose, accepted) in cases {
            let gif = make_many_frame_gif_with_dispose(count, 512, 512, dispose);
            let result = probe_image(&gif);
            if accepted {
                let probe = result.unwrap_or_else(|e| panic!("{count} frames {dispose:?}: {e}"));
                assert_eq!(probe.frame_count, count);
            } else {
                assert!(
                    matches!(result, Err(ImageEngineError::FileTooLarge(_))),
                    "{count} frames {dispose:?} should be rejected, got {result:?}"
                );
            }
        }
    }

    #[test]
    fn gif_accepts_valid_animated_gif_within_limits() {
        let gif = make_many_frame_gif(2, 64, 64);
        let probe = probe_image(&gif).unwrap();
        assert!(probe.animated);
        assert_eq!(probe.frame_count, 2);
        let frames = decode_gif_frames_512(&gif).unwrap();
        assert_eq!(frames.len(), 2);
    }

}
