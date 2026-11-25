# ADR 001 — Product scope

Date: 2026-11-24
Status: accepted

## Context

The product needs a name, a definition of "done" for the first release, and a rule for how ambitious the WhatsApp integration is. A sticker and an audio file are separate resources. WhatsApp's own sound-sticker feature, where it exists, is not a published file format for desktop tools.

## Decision

1. The product name is SolanoStickers.
2. The target experience is both a future native sound sticker and a compatible bundle that works without that feature.
3. The first release edits one sticker and one audio at a time.
4. The project schema stores lists, so a later batch editor does not invent a second file format.
5. v1 clients are the Windows app, the macOS app, and the Chrome extension.

Names that were considered and not used: StikPop, StikForge, WavSticker, SoniStik, StickerBeats, StikJockey, SoundStash, StikBite, WhaSound. SolanoStickers stays understandable if the tool later handles more than one export profile.

## Consequences

- The export UI defaults to the sound bundle.
- The native adapter is a stub.
- Copy and store listings must not promise in-chat audio playback.
- The editor can stay a single screen.

## Alternatives rejected

- Bundle only, with no adapter seam. That forces a rewrite when a real import path appears.
- Native only. There is no verified import path, so the product would not have a shippable output.
- A mobile sticker-pack app as the v1 shell. The requested clients are desktop and Chrome, and the pack API does not carry audio.
