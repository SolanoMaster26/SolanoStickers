# Master TODO

Status: v1 core, engines, minimalist UI, desktop & extension shells implemented (verified via unit tests)
Updated: 2026-11-24

How to use this list: implement phases in order. A phase is done when its exit check passes. Do not enable `native-sound-sticker` from this list. That work is the spike in Phase 9 and stays disabled unless the WhatsApp verification record exists.

Note (2026-11-24 product alignment): Primary export profile is `video-mp4` (H.264 + AAC clip with audio selections of 5 or 10 seconds, on-screen switch, default 10, plus `video-webm-alpha` for real alpha), GIF animated stickers are supported alongside PNG/WebP/JPG, and the UI uses the 4-control minimalist canvas (`✕`, `✓`, center sticker, trim timeline bar).

Requirements live in `docs/requirements`. Behavior lives in `docs/specifications`. This file is only the work order.

## Locked decisions

- [x] Name: SolanoStickers
- [x] Experience: sound bundle now, native WhatsApp adapter only after verification
- [x] v1 width: one sticker and one audio in the UI, many allowed in the schema
- [x] Clients: Windows app, macOS app, Chrome extension
- [x] Stack: Tauri 2, Rust, React, TypeScript, shared project schema

## Phase 0 — Repository skeleton

Goal: an empty window on the desktop and an empty extension tab, with CI.

- [ ] T0.1 Create the pnpm workspace and the Cargo virtual manifest from `docs/architecture/repository-layout.md`
- [ ] T0.2 Add `packages/core` with a build and a single passing unit test
- [ ] T0.3 Add `apps/desktop` with Tauri 2, React, and TypeScript, opening a window titled SolanoStickers
- [ ] T0.4 Confirm `pnpm tauri dev` on Windows shows that window
- [ ] T0.5 Add `apps/extension` Manifest V3 with a service worker that opens `editor.html`
- [ ] T0.6 Load the unpacked extension in Chrome and confirm the tab opens
- [ ] T0.7 Add GitHub Actions to install dependencies and run unit tests
- [ ] T0.8 Add `.gitignore` for `node_modules`, `target`, and Tauri build output
- [ ] T0.9 Add `docs/legal/THIRD_PARTY.md` as an empty notice list with a line that licenses must be filled before a release build

Exit: CI is green and both shells launch. No media code yet.

## Phase 1 — Domain and schema

Goal: projects, validation, and use cases tested with a fake engine. Maps to FR-006, FR-007.4, FR-007.5, FR-008.

- [ ] T1.1 Copy the JSON Schema into the repo path the tests load, keeping `docs/specifications/schemas` as the source of truth
- [ ] T1.2 Implement project, sticker, audio, and association types in `packages/core`
- [ ] T1.3 Validate manifests against the schema. Reject path traversal in `file`
- [ ] T1.4 Enforce audio edit invariants from `docs/specifications/editors.md`
- [ ] T1.5 Implement `createProject`, `importSticker`, `importAudioFile`, `updateAudioEdit`, `associate`, `saveProject`, and `openProject` against an in-memory store
- [ ] T1.6 On sticker replace, keep the audio. On audio replace, reset the trim
- [ ] T1.7 Implement export planning for `standard-webp` and `sound-bundle` using a fake encoder that returns fixture bytes
- [ ] T1.8 Return `export-unsupported` for `native-sound-sticker` and write no file
- [ ] T1.9 Cover TC-006, TC-009, TC-014, TC-015, and SEC-008 with unit tests
- [ ] T1.10 Define the Rust DTO names that will mirror the schema. Do not generate code until Phase 2 if a hand-written struct is enough

Exit: `packages/core` tests pass without Tauri, Chrome, or codecs.

## Phase 2 — Desktop shell around the domain

Goal: the desktop app can create and reopen a project directory. Maps to NFR-001, NFR-007, FR-008.

- [ ] T2.1 Add Tauri commands for create, save, and open that call the Rust side
- [ ] T2.2 Port the save and open rules to `crates/project-store`, or call the TypeScript core from the webview and let Rust only touch the filesystem. Pick one and document it in the crate README. Do not split the rules across both
- [ ] T2.3 Restrict command paths to the project directory and the export temp directory
- [ ] T2.4 Implement temp-file publish from the export specification
- [ ] T2.5 Add a system file dialog for opening a package and choosing an export folder
- [ ] T2.6 Store working files in the per-user app data directory
- [ ] T2.7 Add SQLite only if a recent-project row is needed for development. The P1 library UI can wait
- [ ] T2.8 Capability file: no arbitrary shell, no global filesystem scope

Exit: a project saved on disk reopens with the same manifest. A failed save leaves the previous directory intact.

## Phase 3 — Sticker pipeline

Goal: image in, valid WebP out. Maps to FR-001, FR-005, FR-007.1.

- [ ] T3.1 Implement image sniffing for PNG, JPEG, and static WebP
- [ ] T3.2 Reject animated WebP with the v1 message (TC-016)
- [ ] T3.3 Enforce byte, dimension, and decoded-pixel limits (SEC-009)
- [ ] T3.4 Implement contain-fit onto a 512 x 512 transparent canvas
- [ ] T3.5 Encode WebP with alpha, lossless first, then the quality ladder
- [ ] T3.6 Fail with `sticker-too-large` without publishing (TC-013)
- [ ] T3.7 Wire desktop paste, drop, and file picker for images (FR-001.1 to FR-001.3)
- [ ] T3.8 Show the preview on a checkerboard
- [ ] T3.9 Add fixtures and automate TC-001, TC-002, and TC-012

Exit: a transparent PNG becomes a 512 x 512 WebP at or under 100 KB, and the source file is unchanged.

## Phase 4 — Audio pipeline

Goal: local audio in, trimmed Opus out. Maps to FR-002, FR-004, FR-007.2.

- [ ] T4.1 Choose the MP3 decoder and record its license in `THIRD_PARTY.md`
- [ ] T4.2 Probe duration, sample rate, and channels
- [ ] T4.3 Reject undecodable and oversized files (TC-006)
- [ ] T4.4 Compute 1024 peak buckets off the UI thread
- [ ] T4.5 Render the waveform and the range handles in the shared UI
- [ ] T4.6 Preview the selected range
- [ ] T4.7 Encode mono 48 kHz Ogg Opus at 64 kbps for the selected range
- [ ] T4.8 Add a duration test within 50 ms (TC-003, TC-007)
- [ ] T4.9 Reset trim from the UI (FR-004.8)
- [ ] T4.10 Keep the encoder off the webview thread and show a busy state (TC-010)

Exit: a 30 second MP3 can be trimmed to 10 seconds and exported as `sound.ogg`.

## Phase 5 — MyInstants

Goal: a pasted URL becomes a local audio asset. Maps to FR-003 and the security specification.

- [ ] T5.1 Implement the URL allowlist, redirect cap, and private-address rejection
- [ ] T5.2 Implement the HTML extractor as a pure function
- [ ] T5.3 Add local HTML fixtures and tests TC-004 and TC-005
- [ ] T5.4 Add SEC-001 through SEC-005
- [ ] T5.5 Download through the desktop HTTP client into the project directory
- [ ] T5.6 Pass downloaded bytes through the audio importer
- [ ] T5.7 Store `source.type`, `page_url`, `media_url`, and title
- [ ] T5.8 Paste plain-text URLs from the clipboard into this provider
- [ ] T5.9 Disconnect the network after import and confirm preview still plays

Exit: a MyInstants instant URL and a direct media URL both work, and any other host is rejected.

## Phase 6 — Desktop editor complete

Goal: the primary flow on one screen. Maps to FR-006, FR-007, FR-009, AC-001 through AC-011.

- [ ] T6.1 Build the editor layout from `docs/specifications/user-flows.md`
- [ ] T6.2 Editable project name and sanitized export file name (SEC-010)
- [ ] T6.3 Associate the current sticker and audio and play them together
- [ ] T6.4 Draw the speaker mark in the UI only, never into the WebP
- [ ] T6.5 Export sheet with `sound-bundle` as the default and `standard-webp` as the other choice
- [ ] T6.6 Do not list the native profile
- [ ] T6.7 Write the ZIP with the three required entries and `whatsapp_native: false` (TC-008)
- [ ] T6.8 Reopen the exported package (TC-009)
- [ ] T6.9 Success copy states that the image and the sound are separate files
- [ ] T6.10 About screen with third-party notices and the MyInstants rights sentence
- [ ] T6.11 Map every error code to an English sentence with a next action
- [ ] T6.12 Run the manual Windows half of the release matrix and file gaps as tasks

Exit: AC-007, AC-008, AC-009, and AC-010 pass on Windows. Manual paste passes.

## Phase 7 — Chrome extension

Goal: the same project format in Chrome. Maps to FR-011 and ADR 004.

- [ ] T7.1 Share `packages/ui` and `packages/core` with the extension tab
- [ ] T7.2 Set permissions to `downloads`, `storage`, and the two MyInstants hosts
- [ ] T7.3 Confirm there is no content script and no WhatsApp origin permission (SEC-011)
- [ ] T7.4 Implement `WebMediaEngine` probe, WebP encode, waveform, and Opus encode
- [ ] T7.5 Record the WASM encoder license before merging it
- [ ] T7.6 IndexedDB save of the open project
- [ ] T7.7 Import and export `.solanostickers.zip`
- [ ] T7.8 Download the bundle through `chrome.downloads`
- [ ] T7.9 Unsupported codec path shows the desktop handoff and writes nothing (TC-018)
- [ ] T7.10 Paste image and paste URL inside the editor tab
- [ ] T7.11 Collapse the layout for a narrow window without removing an action
- [ ] T7.12 Round-trip a package from Chrome to the Windows app (TC-017)

Exit: a PNG plus MP3 completes in Chrome, and the resulting ZIP opens in the desktop app.

## Phase 8 — macOS and release artifacts

Goal: the second desktop OS and installable artifacts. Maps to NFR-001 and NFR-011.

- [ ] T8.1 Build the Apple silicon `.app` and run the editor smoke path
- [ ] T8.2 Match Windows on paste shortcut Cmd+V, save, trim, and bundle export (AC-011)
- [ ] T8.3 Produce a `.dmg` for local install
- [ ] T8.4 Produce a Windows installer that detects a missing WebView2 runtime
- [ ] T8.5 Zip the extension build
- [ ] T8.6 Run the full manual matrix on Windows, macOS, and Chrome
- [ ] T8.7 Write release notes that repeat AC-015: native WhatsApp playback is not included
- [ ] T8.8 If a signing certificate exists, sign the Windows build. If not, label the artifact unsigned
- [ ] T8.9 Leave macOS notarization as a follow-up unless an Apple account is already ready

Exit: a tester can install each client and finish the primary flow. The acceptance criteria for the MVP are checked off in `docs/requirements/acceptance-criteria.md`.

## Phase 9 — After the MVP

Do these only after Phase 8. Each item is optional relative to the MVP and ordered.

### Editor depth

- [ ] T9.1 Volume control (FR-004.5)
- [ ] T9.2 WAV, OGG, M4A, and AAC on desktop (FR-002.2)
- [ ] T9.3 Sticker scale and position (FR-005.2)
- [ ] T9.4 Preview backgrounds (FR-005.5)
- [ ] T9.5 Recent projects and delete from the index (FR-008.3, FR-008.4)
- [ ] T9.6 Spanish catalog (FR-010.1)
- [ ] T9.7 Optional 8 px white stroke (FR-005.4)
- [ ] T9.8 Fade-in, fade-out, and peak normalize (FR-004.6, FR-004.7)

### Native WhatsApp spike

- [ ] T9.9 Read the current public WhatsApp sticker docs again and note the date
- [ ] T9.10 Try the official user actions on a current Android client and a current iOS client. Record what import methods exist
- [ ] T9.11 If no supported import exists, write that result in `docs/specifications/whatsapp-native-verification.md` and stop
- [ ] T9.12 If a supported import exists, implement the adapter and enable the profile only after playback is confirmed on a second device
- [ ] T9.13 Do not add a WhatsApp Web content script as a shortcut

### Collections

- [ ] T9.14 UI for several associations in one project (FR-006.5)
- [ ] T9.15 Batch export of sound bundles
- [ ] T9.16 Animated sticker source and animated WebP export within the 500 KB and 10 second rules (FR-001.5, FR-005.8)

## Traceability

| Phase | Requirements |
| --- | --- |
| 0 | NFR-005, NFR-010 |
| 1 | FR-006, FR-007.4, FR-007.5, FR-008.1 |
| 2 | FR-007.7, FR-008.2, NFR-006, NFR-007 |
| 3 | FR-001, FR-005.1, FR-005.3, FR-005.6, FR-005.7, FR-007.1 |
| 4 | FR-002, FR-004, FR-007.2 |
| 5 | FR-003 |
| 6 | FR-006, FR-007, FR-009, FR-010.2 |
| 7 | FR-011, NFR-003 |
| 8 | NFR-001, NFR-011, AC-011 |
| 9 | Remaining P1 and P2 ids |

## Not in any phase

- SolanoStickers accounts or cloud storage
- A built-in MyInstants catalog
- Microphone recording
- Drawing tools
- Sending a WhatsApp message from the app
- Linux support
- Intel macOS, until added under T8
