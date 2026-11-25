# Export specification

Export reads a project and writes a new artifact. It does not modify project assets.

## Profiles

| Profile id | v1 status | Output |
| --- | --- | --- |
| `video-mp4` | Enabled, default | Square MP4 (H.264 + AAC), solid background, at most the selected preset (5 or 10 seconds; default 10). A video file, not a WhatsApp sticker. |
| `video-webm-alpha` | Enabled | WebM with alpha, at most the selected preset (5 or 10 seconds; default 10). A video file, not a WhatsApp sticker. |
| `standard-webp` | Enabled | One 512 x 512 static WebP |
| `sound-bundle` | Enabled | A ZIP with the WebP, the Opus clip, and a manifest |
| `native-sound-sticker` | Registered, disabled | No file. Returns `export-unsupported` |

The UI lists the enabled profiles and hides `native-sound-sticker`. The adapter module still exists so a later build can enable it behind the verification checklist. Sharing or pasting the MP4 in a chat sends a video. WhatsApp does not add that file to the sticker tray. The published sticker file remains the static WebP without audio.

## Validation before write

Run in order. Stop on the first failure. Write nothing to the destination yet.

1. Schema-validate the in-memory project.
2. Confirm the active association exists for `sound-bundle`.
3. Confirm the sticker asset file exists and sniffs as an image.
4. Fit and encode the WebP in a temp directory. Check 512 x 512 and size <= 100 KB.
5. For `sound-bundle`, encode the audio in the temp directory and check the duration tolerance.
6. Build the manifest projection.
7. Publish.

## standard-webp

Destination file name: `<safe-name>-sticker.webp`.

`<safe-name>` is the project name in lower case, spaces turned into hyphens, disallowed path characters removed, truncated to 40 characters. If the result is empty, use `solanostickers`.

## sound-bundle

ZIP file name: `<safe-name>-solanostickers.zip`.

Entries, with forward slashes and no compression of the already-compressed WebP and Opus beyond the default deflate:

```text
manifest.json
sticker.webp
sound.ogg
```

`manifest.json` inside the bundle is a copy of the project manifest plus:

```json
{
  "bundle": {
    "kind": "sound-bundle",
    "created_at": "<utc>",
    "files": {
      "sticker": "sticker.webp",
      "audio": "sound.ogg"
    },
    "audio_encoding": {
      "container": "ogg",
      "codec": "opus",
      "channels": 1,
      "sample_rate": 48000,
      "bitrate_bps": 64000
    },
    "limits": {
      "sticker_px": 512,
      "max_static_bytes": 102400,
      "max_audio_ms": 10000
    },
    "whatsapp_native": false
  }
}
```

`max_audio_ms` is the active duration preset: 10000 in the example (the default) or 5000 when the user selects 5 seconds.

`whatsapp_native: false` is required so a later tool does not mistake the bundle for a WhatsApp sound sticker.

The bundle ZIP must not contain absolute paths, `..`, or extra entries.

## Publish rule

1. Write all outputs into a new temp directory.
2. Validate the ZIP by reopening it and checking entry names and the manifest schema.
3. Move the finished file into the destination.
4. If the destination file exists, replace it only after the temp file is complete. On Windows, replace by writing `<name>.tmp` beside it and then replacing.
5. Delete the temp directory.
6. If any step fails, delete the temp directory and leave any previous destination file in place.

## Destination

Desktop: the user picks a folder. The app remembers it when FR-010.3 is implemented.

Chrome: trigger three downloads only if the user asked for separate files. The default Chrome action downloads the single ZIP.

## Native adapter contract

```text
NativeSoundStickerAdapter.export(project) -> Result<Artifact, ExportError>
```

v1 implementation returns `export-unsupported` without reading the network and without writing a file.

A future implementation may be enabled only when all of these are true:

- the target WhatsApp version and platform are named in the test record
- a file produced by the adapter was imported by that client through a user-visible, supported action
- the recipient client played the audio from the sticker
- the adapter does not depend on an unofficial send protocol, a modified client, or injected WhatsApp Web scripts

Until that record exists, marketing copy and the UI must not say that SolanoStickers creates WhatsApp sound stickers.

## Separate files

From the same temp outputs, desktop can also copy `sticker.webp` and `sound.ogg` next to the ZIP when the user checks "Also save separate files". Default is off. Chrome offers the ZIP only in v1.

## Idempotence

Exporting twice from an unchanged project produces manifests with equal asset and association data. `bundle.created_at` may differ. Media bytes should be stable for a fixed encoder version. Tests compare duration, dimensions, and size limits rather than requiring byte-identical Opus across operating systems.
