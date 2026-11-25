# Risks

| Risk | Impact | What we do |
| --- | --- | --- |
| WhatsApp never documents a third-party sound sticker | The native goal stays blocked | Ship the bundle. Keep the adapter disabled. Do not promise playback |
| A native format appears and differs from Ogg Opus | Re-encode step for one profile | Association stays the source. Only the adapter changes |
| Someone treats reverse-engineering notes as an API | Broken or unsafe send path | ADR 003 forbids a private protocol client |
| MyInstants changes HTML | URL import breaks | One extractor, HTML fixtures, no parser calls in the UI |
| MyInstants blocks datacenter or extension fetches | Import fails for some users | Show `remote-unavailable`. Local files still work. Do not proxy the download through a SolanoStickers server |
| Rights in downloaded sounds | Redistribution complaints | No catalog, store the source URL, say so in About. Do not ship sample sounds from the site |
| ffmpeg license | Cannot legally ship the desktop app | Decide dynamic linking and notices before the first release build. Prefer a Rust decoder for MP3 if it removes the dependency |
| Chrome cannot encode Opus | Extension happy path fails | Pick the WASM encoder in Phase 7 before building the UI around it. Desktop handoff is the specified fallback |
| WebView2 missing on Windows | App does not open | Installer checks and offers the evergreen runtime |
| macOS unsigned app | Gatekeeper blocks public users | v1 can be a local build. Public macOS distribution waits for notarization |
| Clipboard differences | Paste works in one OS only | Platform specification defines the byte order. Manual matrix covers both |
| Zip slip in imported projects | Writes outside the project | Security specification and SEC-006 |
| Scope grows into a sticker studio | v1 slips | Single screen, one association, no drawing tools |
| Two media engines drift | Desktop and Chrome disagree | Shared schema and fixture tests on dimensions, alpha, size, and duration |

## Open points that do not block Phase 0

| Point | When it must be decided |
| --- | --- |
| Which Rust MP3 decoder, and whether ffmpeg is still required | Before Phase 4 packaging |
| Which WASM Opus encoder and its license | Before Phase 7 |
| Code signing certificates | Before the first public installer |
| Spanish catalog | Before calling FR-010.1 done |
| Intel macOS build | Before claiming macOS coverage beyond Apple silicon |
