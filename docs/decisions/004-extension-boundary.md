# ADR 004 — Chrome extension boundary

Date: 2026-11-24
Status: accepted

## Context

The extension is part of the requested product, not a later idea. Chrome can host the editor and can download files. It cannot register WhatsApp sticker packs, and a content script on WhatsApp Web would be an unofficial integration.

Browser decode and encode coverage is narrower than ffmpeg.

## Decision

- The extension is Manifest V3 and opens the editor in a tab.
- It has no content scripts and no permission for WhatsApp origins.
- It reads and writes the same portable package as the desktop apps.
- It uses the web media engine and the shared use cases.
- When the engine cannot finish a file, the user keeps their project and can download it for the desktop app.
- Native messaging to a companion desktop app is out of v1. The package file is the bridge.

## Consequences

- Install is a Chrome extension, which matches the request, without pretending the extension is the WhatsApp client.
- QA includes a fixture the extension must refuse cleanly.
- Store review, when it happens, can rely on a short permission list and a local-processing privacy story.

## Alternatives rejected

- Drive WhatsApp Web from a content script to auto-send the sticker. Breaks when WhatsApp changes the DOM, conflicts with the export decision, and is a poor fit for a creation tool.
- Make the extension a remote control that requires the desktop app. That would make Chrome unusable on its own. v1 Chrome must complete the happy path for PNG and MP3.
- Skip the extension until desktop is finished. The shared packages are cheaper to introduce in Phase 0 than to extract later, even if the extension UI lands after the desktop happy path.
