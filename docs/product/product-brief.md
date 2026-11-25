# Product brief

Status: approved for implementation planning
Date: 2026-11-24
Name: SolanoStickers

## Problem

People already have a sticker they like and a short sound they want to attach to it. Today those are two separate WhatsApp messages. Creating the pair means leaving the chat, finding a converter, trimming audio in another tool, and then hoping the files still belong together.

SolanoStickers keeps that work in one place. The user reuses a sticker. They do not design one from scratch.

## Product statement

SolanoStickers is a desktop and browser tool that associates an existing sticker with a short audio clip and exports that association in every format SolanoStickers can honestly deliver to WhatsApp.

Tagline: Create it. Sound it. Share it.

## Users

Primary user: someone who sends stickers in WhatsApp and wants a specific sound on a specific sticker.

They are not a designer and not an audio engineer. The first session must finish without a tutorial: paste, add sound, trim, export.

## Experience goal

The long-term experience is a single sticker that plays its sound inside WhatsApp when the recipient taps it.

That experience is not available to third-party desktop or browser apps today. See [whatsapp.md](../specifications/whatsapp.md). The product therefore commits to three outcomes, in this order:

1. A portable sound bundle the user can keep, reopen, and send as separate files.
2. A conventional WebP sticker that meets WhatsApp's published static-sticker rules.
3. A native sound-sticker adapter, shipped only after an import path has been verified on a real WhatsApp client.

The first release delivers 1 and 2. It reserves the interface for 3 and does not advertise native playback.

## In scope for the first release

- Paste, drop, or open one sticker image.
- Open one local audio file, or paste one MyInstants URL.
- Preview the sticker and the sound together.
- Trim the sound to the selected duration preset (5 or 10 seconds; 10 seconds by default).
- Save a project and reopen it.
- Export WebP, processed audio, a JSON manifest, and a ZIP bundle.
- Run the same project format on Windows, macOS, and Chrome.

## Explicitly out of the first release

- Drawing or generating a sticker from scratch.
- A built-in sound catalog or a MyInstants browser inside the app.
- Batch creation as a user-facing workflow. The data model allows many associations; the UI edits one.
- Animated stickers with audio.
- Recording from the microphone.
- Sending a message through WhatsApp Web or the WhatsApp desktop client.
- Registering a sticker pack with the official third-party pack API. That API is mobile and has no audio field.
- Accounts, cloud sync, or uploading the user's media to a SolanoStickers server.

## Platforms

| Client | Form | Role in v1 |
| --- | --- | --- |
| Windows 10 and 11 | Installed native app | Full editor and exporter |
| macOS | Installed native app | Full editor and exporter, same behavior as Windows |
| Google Chrome | Manifest V3 extension | Same editor flow and same project package, with the processing limits in [platforms.md](../specifications/platforms.md) |

"Native app" means an installed operating-system binary with a Rust core, not a website wrapped in Electron. The UI is shared web technology inside the OS webview. See [002-stack.md](../decisions/002-stack.md).

## Success for v1

A user on Windows and a user on macOS can each complete the acceptance scenarios in [acceptance-criteria.md](../requirements/acceptance-criteria.md) without an account and without a SolanoStickers server.

A Chrome user can complete the same scenarios for files the browser engine can decode, and can open the resulting package in the desktop app.

## Rights

A downloadable MyInstants file is not a license to redistribute it. SolanoStickers stores the source URL as metadata, shows the sound title when it can read one, and does not ship, cache, or republish a sound library. The user is responsible for the clips they export.
