# Non-functional requirements

## NFR-001 Platforms

| Client | Minimum |
| --- | --- |
| Windows | Windows 10 version 1809 or later, and Windows 11, 64-bit, with the system WebView2 runtime |
| macOS | The current macOS and the previous major version at the time of each release. Apple silicon is required for v1. An Intel build is P1 |
| Chrome | The current stable Chrome on Windows and macOS. Manifest V3 |

Linux is not a v1 target. The Tauri shell may build there later without a support commitment.

The Windows and macOS apps are installed binaries. They are not a hosted website and they do not ship a private copy of Chromium.

## NFR-002 Offline first

After the assets are inside the project, image processing, audio processing, preview, save, and export work with no network connection.

The only required network use is resolving and downloading a URL the user just pasted. A failed network leaves previously imported assets usable.

## NFR-003 Privacy

SolanoStickers has no account and no first-party backend in v1.

The app must not upload stickers, audio, project files, or clipboard contents to a SolanoStickers-operated server. MyInstants receives only the request required to fetch the URL the user pasted.

Crash reporting and analytics are off in v1.

## NFR-004 Performance

Measured on a reference machine: 4 CPU cores, 8 GB RAM, integrated graphics.

| Action | Limit |
| --- | --- |
| Show a pasted PNG or WebP preview | under 300 ms for a source up to 5 MB |
| Show metadata for a local audio file up to 20 MB | under 1 s |
| Build waveform peaks for up to 60 s of audio | under 2 s |
| Export one static sticker and one trimmed clip | under 5 s |
| UI thread | stays responsive during import, decode, and export. Work runs off the UI thread |

These are engineering targets for the test plan, not a warranty.

## NFR-005 Modular architecture

Image processing, audio processing, import providers, and export providers are replaceable behind interfaces.

The UI depends on use cases. It does not call ffmpeg, a WebP encoder, or the MyInstants HTML parser directly.

The domain model does not mention WhatsApp field numbers.

## NFR-006 Security

Follow [security.md](../specifications/security.md).

Untrusted input is: clipboard bytes, dropped files, picked files, pasted URLs, HTTP responses, and imported project packages.

## NFR-007 Persistence

A saved project round-trips through the portable package on Windows, macOS, and Chrome.

Desktop may additionally index projects in a local SQLite database. The database is a cache. The package remains the source of interchange.

Closing the app during an export must not corrupt an existing saved project. Write to a temporary directory and publish by replace.

## NFR-008 Usability

A new user can finish paste, sound, trim, and export from one screen without opening settings.

Errors say what failed and what the user can do next. They do not expose stack traces in the main UI.

Keyboard paste works in the desktop apps. The extension paste works when the editor tab is focused, because Chrome only releases clipboard images on a user gesture.

## NFR-009 Internationalization

All user-visible strings live in message catalogs.

v1 catalog: English.
P1 catalog: Spanish.

Dates and file timestamps use UTC in the manifest and local time in the UI.

## NFR-010 Replaceable edges

A new sound provider or a new export profile is a new module plus a registration entry.

Shipping the native WhatsApp adapter must not require a change to the project schema beyond a new `export.profile` value already allowed by the schema.

## NFR-011 Packaging and updates

| Client | Package |
| --- | --- |
| Windows | Signed installer when a certificate is available. v1 may ship an unsigned installer for local use, labeled as such |
| macOS | `.dmg` containing a `.app`. Notarization is required before public distribution and is P1 if no Apple account is ready |
| Chrome | An unpacked extension for development and a `.zip` ready for the Chrome Web Store. Store submission is P1 |

In-app auto-update is P2. v1 can be replaced by installing a newer build.

## NFR-012 Licensing of dependencies

Before a dependency is bundled, record its license in `docs/legal/THIRD_PARTY.md` (created when the first dependency is added).

ffmpeg, if bundled, stays dynamically linked and its license notices ship with the app. The Chrome extension does not bundle ffmpeg. See [002-stack.md](../decisions/002-stack.md).

## NFR-013 Accessibility

P1, but the v1 layout must not block it:

- Preview and export actions have visible text, not only icons.
- Playback is reachable from the keyboard on desktop.
- The sticker preview has a text alternative that includes the project name and whether audio is attached.

## NFR-014 Data limits

Defaults, enforced before decode where possible:

| Input | Limit |
| --- | --- |
| Image bytes | 20 MB |
| Audio bytes | 25 MB |
| Audio duration considered for editing | 3 minutes. Longer files are rejected with a message to trim externally |
| Downloaded URL body | 25 MB |
| Project package uncompressed | 100 MB |
| Files inside an imported package | 32 |

Limits are constants in one module so tests and both clients share them.
