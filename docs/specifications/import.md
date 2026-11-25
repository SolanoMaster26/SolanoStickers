# Import specification

Import turns an external image or audio into a project asset. The source path is metadata. Later edits read the project copy.

## Asset identity

Each imported asset gets an id with the prefix `stk_` or `aud_` plus 12 lowercase hex characters from a random source. Ids are unique inside the project.

The working copy file name is `{id}.{ext}` where `ext` matches the sniffed type, not the original extension.

## Image sniffing

Read at least the first 32 bytes and the format markers required by the decoder.

| Format | Accept in v1 | Notes |
| --- | --- | --- |
| PNG | Yes | Include tRNS and alpha |
| JPEG | Yes | No alpha. Output still has a transparent canvas around the fitted image |
| WebP | Yes | Static WebP is P0. Animated WebP is stored and previewed in P1, and cannot be the standard-webp export until the animated editor exists |

Reject SVG, GIF, BMP, HEIC, and unknown content in v1 with `unsupported-image`. Do not shell out to a general converter for those types in the MVP.

Animated input in v1: if the decoder reports animation, accept the file as a source only when FR-001.5 is implemented. Until then, reject it with a message that animated stickers are not in this version. Do not silently export the first frame.

## Image fit for the output frame

When producing the 512 x 512 master used for preview and export:

1. Decode the source.
2. Scale uniformly so the longest side is at most 512.
3. Center the result on a 512 x 512 transparent canvas.
4. Do not premultiply in a way that fringes the edge. Use a resampler that keeps alpha.

The stored source asset remains the original bytes. The fitted frame is a derived preview and may be rebuilt. Export encodes the fitted frame.

## Audio sniffing

Trust the decoder probe, not the extension.

| Container | v1 desktop | v1 Chrome |
| --- | --- | --- |
| MP3 | P0 | P0 if `decodeAudioData` accepts it |
| WAV PCM | P1, implement in the desktop MVP if the probe already supports it | Same, if the browser decodes it |
| OGG | P1 | Best effort |
| M4A / AAC | P1 | Best effort |
| Opus in Ogg | Accepted as an input when the P1 OGG row is implemented | Best effort |

Store the original bytes. Store probe results: duration in milliseconds, sample rate, channels, and a format label.

If the duration is over 3 minutes, reject with `file-too-large` before generating peaks.

## Clipboard

### Desktop

On the paste command, inspect the clipboard in this order:

1. Image bytes (PNG preferred).
2. A list of file paths. If the first file sniffs as audio, route it to audio import. If it sniffs as an image, route it to sticker import. Extra files are ignored in v1 with a note that only one file was used.
3. Plain text. Trim whitespace. If it parses as an absolute `http` or `https` URL, route it to the URL importer. Otherwise ignore the paste and keep the current assets.

A paste that contains both an image and text uses the image.

### Chrome

The editor page handles the `paste` event.

- If `clipboardData` contains an image file or a `image/*` item, import it as the sticker.
- Otherwise if it contains text that parses as an `http` or `https` URL, route it to the URL importer.
- The page also offers a manual "Read clipboard" button so the user can grant clipboard access with a click when the browser blocks the event.

## Drag and drop

Accept a single file per drop in v1. Route by sniffed content. If the type is ambiguous, ask with two actions: "Use as sticker" and "Use as audio". That prompt is required only when sniffing recognizes both, which should be rare. Unknown types use `unsupported-image` or `unsupported-audio` based on which zone received the drop. A drop on the window chooses by sniffing.

## File picker filters

The picker may filter by extension as a convenience. After selection, sniffing still decides. A `.png` that is not a PNG is rejected.

## Replacement rules

| Action | Sticker asset | Audio asset | Trim | Fit |
| --- | --- | --- | --- | --- |
| Replace sticker | New id | Kept | Kept | Reset to default fit |
| Replace audio | Kept | New id | Reset to the full clip, then clamped to the profile limit | Kept |
| Remove sticker | Cleared | Kept | Kept | Cleared |
| Remove audio | Kept | Cleared | Cleared | Kept |

## Temporary files

Desktop writes the copy into the project directory as soon as import succeeds. A failed decode deletes the partial copy.

Chrome holds bytes in IndexedDB and only writes files on export or package download.
