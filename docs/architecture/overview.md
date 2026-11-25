# Architecture

## Summary

One domain, two media engines, three shells.

```text
Windows app ─┐
macOS app  ──┼─► use cases ─► Project model ─► export profiles
Chrome ext ──┘        │
                      ├─ DesktopMediaEngine (Rust)
                      └─ WebMediaEngine (browser)
```

The use cases do not know which shell called them. Each shell binds a media engine at startup.

## Layers

| Layer | Responsibility | Lives in |
| --- | --- | --- |
| Shell | Windows, menus, file dialogs, downloads, webview | `apps/desktop`, `apps/extension` |
| UI | Editor layout, waveform view, copy | `packages/ui` |
| Use cases | Import, associate, trim, save, export | `packages/core` |
| Domain | Types, invariants, schema validation | `packages/core` |
| Providers | MyInstants extraction and URL policy | `packages/providers` |
| Media engines | Decode, fit, peaks, encode, zip | `crates/*` and `packages/media-web` |
| Storage | Project directory, SQLite index, IndexedDB | shell adapters |

## Use cases

These are the only operations the UI may call:

| Use case | Effect |
| --- | --- |
| `createProject` | Empty project in memory |
| `importSticker` | Sniff, copy, set fit |
| `importAudioFile` | Sniff, copy, set default trim |
| `importAudioUrl` | Provider resolve, then `importAudioFile` rules |
| `updateAudioEdit` | Validate and store trim and volume |
| `updateStickerEdit` | Validate fit fields |
| `associate` | Bind active sticker and audio |
| `preview` | Return object URLs or paths plus the edit |
| `saveProject` | Write manifest and assets |
| `openProject` | Read and validate a package |
| `exportProject` | Run the selected profile |

Each use case returns a structured error code from the user-flow specification.

## Media engine interface

```text
probeImage(bytes) -> ImageInfo | Error
renderSticker(source, edit) -> RgbaFrame512
encodeWebp(frame) -> bytes
probeAudio(bytes) -> AudioInfo | Error
waveform(bytes) -> PeakBuckets
renderAudio(bytes, edit) -> PcmMono48k
encodeOpus(pcm) -> bytes
zip(entries) -> bytes
```

`DesktopMediaEngine` implements this in Rust and is called through Tauri commands. Commands take ids and edit structs, then resolve paths inside the project directory. They do not take arbitrary filesystem paths from the webview, except for the user-selected destination folder returned by the official dialog API.

`WebMediaEngine` implements the same interface in TypeScript with:

- browser decoders for images and for `decodeAudioData`
- a WebP encoder that preserves alpha
- an Opus encoder compiled to WebAssembly under a permissive license
- a ZIP writer in process

If a method cannot handle the bytes, it returns `unsupported-audio` or `encode-failed` mapped to the Chrome handoff. It does not call a remote conversion API.

## Why two engines

The desktop app can ship a dynamically linked ffmpeg and a Rust image pipeline. The Chrome extension cannot ship that stack cleanly, and it must not send the user's media to a server to compensate.

Parity tests run the same fixtures through both engines and compare the acceptance properties: dimensions, alpha presence, file size, and duration. They do not require identical bytes.

## Desktop process

```text
Webview (React)
  └─ Tauri commands
       └─ Rust application
            ├─ image crate
            ├─ audio crate
            ├─ project crate
            └─ sqlite + filesystem
```

ffmpeg is invoked with a fixed argument list built in Rust, or replaced by a Rust decoder if one covers the P0 formats before packaging. The UI never builds that argument list.

Long commands report progress events. The webview stays interactive.

## Chrome process

```text
Extension service worker
  └─ opens editor tab
       └─ React UI
            └─ packages/core + WebMediaEngine
                 └─ IndexedDB
```

The service worker does not decode media. Fetching MyInstants may live in the editor page so the user can see failures. The page still uses the provider module and the allowlist.

## Export pipeline

```text
project + profile
  -> validate
  -> renderSticker + encodeWebp
  -> renderAudio + encodeOpus   (sound-bundle only)
  -> manifest projection
  -> zip or single file
  -> publish
```

`native-sound-sticker` is a profile whose function returns `export-unsupported`.

## Internationalization

UI strings are keys. `packages/ui/locales/en.json` ships in v1. `es.json` is added for FR-010.1 without changing components.

## Testing seams

- Domain and MyInstants extractor: unit tests with fixtures, no network, no webview.
- Export publish rule: integration tests on a temp directory.
- Engines: fixture-based characterization tests.
- Shells: one desktop end-to-end smoke and one extension smoke, described in the test plan.

## What is intentionally absent

- A SolanoStickers server.
- A plugin loader that downloads code.
- A WhatsApp protocol client.
- A shared mutable store between the extension and the desktop app. They share files the user saves.
