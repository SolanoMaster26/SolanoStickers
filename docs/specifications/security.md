# Security specification

SolanoStickers processes untrusted images, audio, archives, and web pages on the user's machine. The threat is a malicious file or URL crashing the app, reading unrelated files, or making the app call an internal network address.

## URL policy

Applies to every outbound request, including redirects.

- Schemes: `https` only.
- Hosts: the provider allowlist. v1 lists only the MyInstants hosts in the MyInstants specification.
- Reject userinfo, non-443 ports, raw IP literals, and localhost names.
- Resolve the host and reject private, loopback, link-local, and unique-local addresses before connecting.
- Re-check the policy on every redirect. Maximum 2 redirects.
- Timeout 20 seconds. Maximum response size from NFR-014. Abort and discard the body when the limit is crossed.
- Do not send cookies, authorization headers, or project metadata.

Chrome's host permission must match the same hosts. The extension must not call other origins.

## File policy

- Identify types by content. Ignore the extension when they disagree.
- Enforce size limits before full decode when the length is known.
- Decode in the engine process, off the UI thread. A decode error becomes `unsupported-image` or `unsupported-audio`.
- Do not execute downloaded bytes. Do not honor archive extra fields as paths.
- Image decoders must be configured to reject decompression bombs: cap decoded pixels at 40 megapixels and cap a single dimension at 8000 px before the fit step. Inputs inside the byte limit can still expand.
- Animated GIFs are further capped by GIF-specific limits: at most 300 frames, at most 64 MiB (16,777,216 pixels) for the logical canvas and any individual frame descriptor, and at most 256 MiB of retained decoded RGBA frame memory including the working canvas and disposal-method buffers. Frame descriptors are validated before any LZW output allocation. Exceeding these limits is rejected with a typed error instead of being truncated.
- Both the browser (`packages/media-web`) and native (`crates/image-engine`) GIF decode paths enforce the same limits before allocation, before retaining each frame, and before reporting a successful image probe.
- The image probe checks the decoded memory budget of the export decoder (`decodeGifFramesRgba` in `packages/media-web`), which retains one canvas-sized RGBA frame per GIF frame. The desktop app probes with `crates/image-engine` and exports with that decoder, so both probes use the same per-frame formula (`gifFrameWorkingBytes` and `export_gif_frame_working_bytes`). A GIF that passes the probe does not fail export with `file-too-large`.

## Package import

When opening a portable ZIP or directory:

- Reject absolute paths, drive letters, and any segment that is `..` or empty.
- Accept only `manifest.json` and paths under `assets/` with a single file name.
- Cap entry count at the NFR-014 file limit and uncompressed size at the package limit.
- Validate `manifest.json` against the schema before copying assets into the working area.
- Confirm every referenced file exists and no extra asset is required for v1. Extra asset files that are not referenced are ignored and are not executed.
- Overwrite only the destination project working copy, never an arbitrary path from the manifest.

## Zip export

Write the bundle with a library that encodes entry names as the exact relative strings in the export specification. Test that a crafted project name cannot change the entry name.

## Chrome

- Manifest V3, no remote code, no `eval` of fetched HTML.
- Content Security Policy uses the extension default. Do not relax `script-src`.
- HTML fetched from MyInstants is parsed as text by the extractor. It is never assigned to `innerHTML` and never opened in a webview.
- The extension does not register a content script.

## Desktop

- Tauri capabilities allow the commands the UI needs and deny arbitrary shell execution.
- The UI cannot pass a raw shell string. Encode arguments are structured: input path inside the working directory, output path inside the temp directory, and numeric trim values.
- Working paths are created by the app. A command refuses an input path outside the project directory and an output path outside the temp directory.

## Dependency and update risk

- Pin dependency versions.
- Record licenses before bundling ffmpeg or an Opus encoder.
- Do not auto-update in v1, so a compromised update channel is not in the first release.

## Logging

Logs may contain error codes, status codes, and durations. Logs must not contain clipboard contents, media bytes, or full MyInstants URLs in release builds. A debug build may log the host and path.

## Test obligations

The security cases in the test plan are P0. A build that fails them is not a release candidate.
