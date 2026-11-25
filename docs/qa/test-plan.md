# Test plan

P0 cases gate a release candidate. Clipboard and installer cases are manual until a shell harness exists. Everything else should be automated against `packages/core`, the engines, and fixture files.

## Functional

| ID | Case | Expected | Auto |
| --- | --- | --- | --- |
| TC-001 | Paste or import a transparent PNG | Preview and WebP keep a transparent corner pixel | Auto for the engine, manual for paste |
| TC-002 | Import a static WebP | Dimensions and alpha match the probe | Auto |
| TC-003 | Import a 30 s MP3 and select 0–10000 ms | Exported Opus duration is within 50 ms of 10 s | Auto |
| TC-004 | Fixture HTML for a MyInstants page | Extractor returns the fixture media URL and title | Auto |
| TC-005 | URL on a non-allowlisted host | `url-rejected`, no bytes stored | Auto |
| TC-006 | Truncated PNG and random bytes renamed `.mp3` | Rejected, previous assets unchanged | Auto |
| TC-007 | Trim 1500–4200 ms | Exported duration within 50 ms of 2700 ms | Auto |
| TC-008 | Export sound bundle | ZIP entries are exactly `manifest.json`, `sticker.webp`, `sound.ogg`. Manifest validates. `whatsapp_native` is false | Auto |
| TC-009 | Save, reload the package | Association ids, trim, and source URLs match | Auto |
| TC-010 | Export a 5 MB PNG while the UI test watches the frame | Completion event arrives. The test process also handles a second ping during the job | Desktop integration |
| TC-011 | Run TC-008 on Windows and macOS | Both bundles validate and meet size and duration rules | CI |
| TC-012 | Export `standard-webp` | 512 x 512 and <= 100 KB | Auto |
| TC-013 | Lossless WebP would exceed 100 KB | Encoder steps quality down or returns `sticker-too-large` and publishes nothing | Auto |
| TC-014 | Selection of 12 s | Export control reports `trim-out-of-range` | Auto |
| TC-015 | Replace audio | Sticker id is kept, trim resets | Auto |
| TC-016 | Animated WebP before animated support | `unsupported-image` with the version message | Auto |
| TC-017 | Chrome opens a desktop bundle | Preview matches, and a new download reopens on desktop | Manual then extension integration |
| TC-018 | Chrome cannot encode a fixture | Message names the desktop app. Partial ZIP is absent | Auto at the engine boundary |

## Security

| ID | Case | Expected |
| --- | --- | --- |
| SEC-001 | `http://www.myinstants.com/...` | `url-rejected` |
| SEC-002 | `https://www.myinstants.com@evil.example/` | `url-rejected` |
| SEC-003 | Redirect from an allowlisted host to `http://127.0.0.1/` | `url-rejected`, no follow |
| SEC-004 | Redirect to a public non-allowlisted host | `url-rejected` |
| SEC-005 | Body larger than the audio limit | Aborted, `file-too-large` |
| SEC-006 | ZIP entry `../outside.txt` | Package rejected, nothing written outside the temp dir |
| SEC-007 | ZIP entry `assets/../../secret` | Rejected |
| SEC-008 | Manifest `file` value `/etc/passwd` or `assets/../x` | Schema rejection |
| SEC-009 | Image within the byte limit that decodes beyond 40 megapixels | Rejected before export |
| SEC-010 | Project name `../../escape` | Output file name is the sanitized fallback, ZIP entries unchanged |
| SEC-011 | Extension manifest permission list | Only the permissions in the platform specification |

## Manual release matrix

Run on a release candidate, not on every commit.

| Step | Windows | macOS | Chrome |
| --- | --- | --- | --- |
| Install or load | Installer or debug app | `.app` | Unpacked extension |
| Paste PNG | Ctrl+V | Cmd+V | Paste in the editor tab |
| Drop MP3 | Yes | Yes | Yes |
| Paste a live MyInstants URL | Yes, then disconnect the network and preview | Same | Same |
| Export bundle and reopen | Yes | Yes | Download, then open in desktop |
| Quit mid-export and reopen the previous save | Previous save intact | Same | IndexedDB project intact |

Record the OS version, the app commit, and the Chrome version in the release note.

## Fixtures to add in Phase 0

| Fixture | Purpose |
| --- | --- |
| `transparent-32.png` | Alpha |
| `opaque.jpg` | No alpha, contain-fit |
| `static.webp` | WebP round trip |
| `animated.webp` | Rejection until P2 |
| `tone-30s.mp3` | Trim |
| `truncated.png` | Corrupt image |
| `not-audio.mp3` | Corrupt audio |
| `instant-page.html` | Extractor success |
| `instant-page-missing.html` | Extractor failure |
| `zip-slip.zip` | Path traversal |
| `project-valid/` | Round trip |

Do not commit copyrighted MyInstants audio. HTML fixtures should be minimal pages written for the test, containing a realistic `/media/sounds/` URL, not a scraped production page if that page's terms forbid storing it. The live URL check stays on the manual matrix.
