# Editor specification

## Audio timeline

Time is an integer number of milliseconds from the start of the stored audio.

```text
AudioEdit
  start_ms: integer >= 0
  end_ms: integer > start_ms
  volume: number from 0 to 1, default 1
  fade_in_ms: integer >= 0, default 0
  fade_out_ms: integer >= 0, default 0
```

Invariants:

- `end_ms` is at most the probed duration.
- `end_ms - start_ms` is at least 100 ms.
- `end_ms - start_ms` is at most the active profile `max_audio_ms` at export time. The editor may display a longer selection, and export stays disabled until the selection fits.
- `fade_in_ms + fade_out_ms` is at most the selected duration. v1 stores fades but does not apply them unless FR-004.6 is implemented. The v1 encoder ignores fade fields and still round-trips them.

Default selection after import:

- start at 0
- end at `min(duration_ms, active preset)`

The editor exposes two duration presets, 5000 ms and 10000 ms. The default preset is 10000 ms. The active preset is the export cap for that session and cannot exceed the profile `max_audio_ms`.

Reset restores that default and sets volume to 1 and fades to 0.

## Waveform

The waveform is a derived cache of peaks, not project data.

- Compute min and max peaks for a fixed bucket count (1024 buckets across the full clip).
- Render the selected range with a distinct color.
- Clicking the waveform or moving the slider repositions a window clamped to the active preset (5 or 10 seconds). The selection cannot exceed that preset.
- Play selection starts at `start_ms` and stops at `end_ms`.
- Play does not change the selection.

## Volume

P1. Until the control exists, the encoder uses 1.0. When the control exists, gain is applied to the selected samples before encode. Values outside 0..1 are clamped at the use-case boundary.

## Loudness and fades

P2. Normalization is peak normalization of the selected range to -1 dBFS, not loudness-unit normalization. Document that in the UI when the control appears so users do not expect broadcast loudness.

## Encoded audio result

The export encoder always produces:

| Property | Value |
| --- | --- |
| Container | Ogg |
| Codec | Opus |
| Channels | 1, downmixed if the source is stereo |
| Sample rate | 48000 Hz |
| Bitrate | 64 kbps |
| Range | `start_ms` inclusive to `end_ms` exclusive, after gain |

Duration tolerance against the requested range: 50 ms, matching AC-003.

The encoder does not embed artwork or the source URL in the audio tags in v1. Provenance lives in the manifest.

## Sticker frame

```text
StickerEdit
  fit: "contain"          # only value in v1
  offset_x: 0             # P1
  offset_y: 0             # P1
  scale: 1                # P1, relative to the contain fit
  stroke: "none" | "white-8px"
  preview_background: "checker" | "white" | "black" | "gray"
```

v1 uses `contain`, zero offsets, scale 1, stroke `none`, preview background `checker`. The checker is a UI concern and is not encoded into the WebP.

## WebP encode

| Property | Value |
| --- | --- |
| Dimensions | 512 x 512 |
| Alpha | Required. Fully transparent pixels stay transparent |
| Animation | Not produced in v1 |
| Lossless | Tried first |
| Lossy fallback | Quality steps 90, 80, 70, 60, 50 until the file is 100 KB or smaller |
| Failure | `sticker-too-large` if 50 still exceeds 100 KB |

The encoder strips metadata (EXIF, XMP, ICC). Color is sRGB.

## Preview playback

- One preview player exists. Starting playback stops the previous playback.
- The sticker view does not flash or resize when audio starts.
- A speaker icon is drawn by the UI over the preview corner when an association has audio. It is not burned into the exported WebP.
- Stopping is immediate.

## Disabled export

The export action is disabled, with the reason visible, when:

- there is no sticker
- the selected profile needs audio and there is no audio
- the selection breaks the audio invariants
- a previous validation error has not been cleared by a new edit

`standard-webp` does not require audio. `sound-bundle` requires both.
