# ADR 002 — Technology stack

Date: 2026-11-24
Status: accepted

## Context

SolanoStickers needs an installed app on Windows and macOS and an installable Chrome extension. Media must be processed locally. The three clients must open one project format.

Separate WinUI and AppKit applications would duplicate the editor. Electron would ship a second Chromium with every desktop install. A website cannot meet the offline and privacy requirements.

## Decision

- Desktop shell: Tauri 2, using the system webview (WebView2 on Windows, WKWebView on macOS) and a Rust core.
- UI: React and TypeScript, shared by the desktop webview and the extension tab.
- Styling: a small component set. Tailwind CSS is allowed. A heavy component kit is not required for the first screen.
- Desktop media: Rust crates. ffmpeg may be bundled dynamically for formats the Rust decoders do not cover, with license notices.
- Extension media: a TypeScript engine on Web APIs plus a permissively licensed WebP encoder and Opus encoder in WebAssembly. ffmpeg is not shipped inside the extension.
- Desktop index: SQLite.
- Extension storage: IndexedDB.
- Contract: the JSON Schema in `docs/specifications/schemas`.
- Tests: unit tests for core and providers, integration tests for export, one smoke path per shell.
- CI: GitHub Actions, with Windows and macOS runners for desktop packaging.

The desktop apps are native in the sense that matters for installation and for file and clipboard access: they are OS binaries with a Rust process. The interface is rendered by the OS webview so the Chrome extension can share it.

## Consequences

- WebView2 must be present on Windows.
- macOS v1 is Apple silicon. Intel is a later build.
- Some audio files will work on desktop and fail in Chrome. That failure is specified, not accidental.
- Two engines need fixture tests so the manifest and the limits do not drift.

## Alternatives rejected

- Electron. Larger install and a private Chromium, for an app whose heavy work is already in native code.
- WinUI plus SwiftUI plus a separate extension UI. Three editors to keep in parity.
- A single Rust core compiled to WebAssembly for both shells. Attractive later. It is not required to start, and the Chrome allowlist and download APIs would still be separate. Revisit if the two engines diverge.
