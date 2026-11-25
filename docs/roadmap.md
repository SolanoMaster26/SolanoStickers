# Roadmap

The master TODO is the task list. This page explains the order.

## Phase 0 — Repository

A workspace that builds an empty desktop window and an empty extension tab, with CI running unit tests. No editor yet.

## Phase 1 — Domain

The project model, schema validation, and use cases with an in-memory media engine fake. This phase proves save, trim rules, and export validation before any codec work.

## Phase 2 — Desktop shell

A Tauri window loads the shared UI shell, opens a file dialog, and writes a project directory. Still using the fake engine if the real one is incomplete.

## Phase 3 — Image path

Paste and drop produce a 512 x 512 transparent WebP under 100 KB.

## Phase 4 — Audio path

Local MP3 becomes a waveform, a trim, and an Ogg Opus file.

## Phase 5 — MyInstants

Paste a URL, store the audio, continue offline. The provider is isolated and fixture-tested.

## Phase 6 — The editor

The single screen ties sticker, audio, preview, and save together. Acceptance AC-001 through AC-010 pass on desktop except live clipboard cases, which are manual.

## Phase 7 — Chrome

The same use cases on the web engine, package round trip, and the unsupported-file handoff.

## Phase 8 — Release candidates

Windows installer, macOS disk image, extension zip, manual matrix, license notices.

## Phase 9 — After v1

In product priority order, not a commitment to build them all next:

1. Recent projects, volume, Spanish.
2. More audio containers.
3. Sticker position and scale.
4. Verification spike for a native WhatsApp sound sticker. The spike writes the verification record or a document that says the path is still absent. It does not enable the profile on hope.
5. Batch associations and collections.
6. Animated stickers.
7. Fades and peak normalize.

Microphone recording stays out until a release explicitly adds an entitlement and a privacy note.
