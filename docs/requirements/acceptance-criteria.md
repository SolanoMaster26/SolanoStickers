# Acceptance criteria

The MVP is accepted when every P0 criterion below passes on Windows and macOS, and the Chrome column passes for the Chrome build. A criterion that the extension cannot meet must fail with the desktop handoff in AC-014, not with a broken file.

Native WhatsApp playback is not an MVP criterion. AC-015 records that exclusion on purpose.

## Core path

| ID | Given | When | Then |
| --- | --- | --- | --- |
| AC-001 | The editor is open and the clipboard holds a transparent PNG | The user pastes | A preview appears, alpha is preserved, and the source file on disk is unchanged |
| AC-002 | The user has a static WebP sticker | They import it | The preview matches the source and the later export remains WebP at 512 x 512 |
| AC-003 | The user has an MP3 longer than 10 seconds and the duration preset is 10 seconds (the default) | They import it and keep a range of 10 seconds or less | Preview plays that range, and the exported audio duration matches the range within 50 ms |
| AC-004 | The clipboard holds a working MyInstants instant URL | The user pastes it | The app stores a local audio file, shows a title or the URL, and can play it with the network disconnected afterwards |
| AC-005 | The clipboard holds a URL outside the allowlist, or a MyInstants URL that 404s | The user pastes it | A recoverable error is shown and no partial asset is attached |
| AC-006 | The user picks a truncated or non-media file | Import runs | The file is rejected. Existing project assets stay as they were |
| AC-007 | A sticker and a trimmed clip are associated | The user exports a sound bundle to an empty folder | The folder contains the ZIP. The ZIP contains `manifest.json`, one `.webp`, and one `.ogg`. The manifest validates against schema 1.0. Paths in the manifest match the entries |
| AC-008 | A project was saved | The user quits, relaunches, and opens the package | Sticker, audio, trim, volume, and profile match the saved manifest |
| AC-009 | The user exports a standard sticker | Validation runs | The WebP is 512 x 512 and 100 KB or smaller |
| AC-010 | Encoding cannot get the WebP under 100 KB | Export runs | No destination file is published, and the UI explains the size failure |

## Platforms

| ID | Criterion |
| --- | --- |
| AC-011 | The same fixture project exports byte-identical manifest semantics on Windows and macOS. Binary media may differ only if both outputs still pass AC-003, AC-007, and AC-009 |
| AC-012 | Chrome imports that package, previews it, and writes a package the desktop app can reopen |
| AC-013 | Import, waveform, and export do not freeze the window. A long task shows progress or a busy state and can be left without killing the process |
| AC-014 | Chrome shows a specific message when it cannot decode or encode a fixture, and it does not write a partial bundle |

## Explicit non-criteria

| ID | Statement |
| --- | --- |
| AC-015 | MVP acceptance does not require WhatsApp to play the exported audio when the image is imported as a sticker. The native adapter stays disabled |
| AC-016 | MVP acceptance does not require animated stickers, microphone recording, Spanish copy, or a project list |

## Definition of done for a story

A story that implements a P0 requirement is done when:

- the requirement's acceptance rows are covered by an automated test or, where the OS clipboard or file dialog is involved, by a documented manual check
- the portable manifest still validates
- the change works in the desktop shell, and either works in Chrome or is listed in the extension capability matrix
