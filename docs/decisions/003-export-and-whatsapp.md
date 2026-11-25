# ADR 003 — Export decoupling and WhatsApp

Date: 2026-11-24
Status: accepted

## Context

Users want a sticker that plays audio inside WhatsApp. Public documentation covers static and animated WebP stickers and mobile sticker packs. It does not cover a third-party sound sticker.

Reports in October and November 2026 describe a first-party authoring UI and, for some accounts, playback of stickers that already contain sound. Unofficial notes describe audio as a separate message field, with client-side Opus encoding. None of that is a file we can drop into the sticker tray.

Embedding an MP3 in a WebP would produce an image WhatsApp treats as a normal sticker, or an image it rejects.

## Decision

- The domain model stores a sticker asset, an audio asset, and an association.
- Export profiles are plugins over that model.
- v1 enables `standard-webp` and `sound-bundle`.
- v1 registers `native-sound-sticker` and keeps it disabled.
- The sound bundle sets `whatsapp_native` to false.
- SolanoStickers will not implement the private WhatsApp message protocol, will not automate WhatsApp Web, and will not claim native playback without the verification record in the WhatsApp specification.

The default audio encoding of the bundle is Ogg Opus, mono, 48 kHz, because it is a compact speech-and-effects codec and it matches the direction of the public client observations. That choice is an interchange decision. It is not protocol compatibility.

## Consequences

- The MVP is useful before WhatsApp documents anything: the user keeps a matched pair of files and can send them.
- A future adapter can be added without migrating projects.
- Product copy stays narrower than the long-term goal.

## Alternatives rejected

- Wait to start until Meta publishes a format. The bundle does not need that format.
- Reverse engineer the send path and emit protocol messages from the desktop app. Fragile, unofficial, and the wrong distribution model for a user tool.
- Put audio metadata only in a sidecar next to the WebP with no manifest schema. The project would be harder to reopen and easier to break.
