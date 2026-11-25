# Functional requirements

Status: baseline for v1
Priority: P0 is required for the MVP. P1 is the next increment. P2 waits for a later phase.

Identifiers are stable. Do not renumber them when a requirement moves between phases. Record the phase in the master TODO instead.

## FR-001 Sticker import

The original file is never modified. Import copies bytes into the project working area.

| ID | Requirement | Priority |
| --- | --- | --- |
| FR-001.1 | Paste an image from the clipboard with Ctrl+V on Windows and Cmd+V on macOS. The Chrome editor accepts paste inside its page. | P0 |
| FR-001.2 | Drag and drop an image onto the editor. | P0 |
| FR-001.3 | Choose a local image with the system file picker. | P0 |
| FR-001.4 | Accept PNG, JPG, JPEG, and WebP. Detection uses file content, not the extension. | P0 |
| FR-001.5 | Accept animated WebP and preserve it as a source asset. | P1 |
| FR-001.6 | Preserve the alpha channel of the source. | P0 |
| FR-001.7 | Show a preview before any export. | P0 |
| FR-001.8 | Reject an invalid or unsupported image with a recoverable error. The editor stays usable. | P0 |
| FR-001.9 | Report width, height, whether alpha is present, and whether the image is animated. | P0 |

## FR-002 Audio import

| ID | Requirement | Priority |
| --- | --- | --- |
| FR-002.1 | Import an MP3 file. | P0 |
| FR-002.2 | Import WAV, OGG, M4A, and AAC. | P1 |
| FR-002.3 | Drag and drop an audio file. | P0 |
| FR-002.4 | Paste a URL. MyInstants page URLs and direct media URLs are handled by FR-003. Any other host is rejected in v1. | P0 |
| FR-002.5 | Detect the container, the codec when known, the duration, the channel count, and the sample rate. | P0 |
| FR-002.6 | Show file name or sound title, duration, and format before the user commits the association. | P0 |
| FR-002.7 | Play the imported audio before it is associated. | P0 |
| FR-002.8 | Replace the audio without discarding sticker crop, size, or export settings. | P0 |
| FR-002.9 | Reject a corrupt or undecodable file with a recoverable error. | P0 |

WAV is P1 as a required format and should be attempted in the desktop MVP if the decoder already supports it at no extra cost. The acceptance bar for P0 is MP3 plus the MyInstants payload, which is typically MP3.

## FR-003 MyInstants

MyInstants is an import provider, not a hardcoded parser scattered through the UI.

| ID | Requirement | Priority |
| --- | --- | --- |
| FR-003.1 | Accept a pasted MyInstants instant URL and a direct MyInstants media URL. | P0 |
| FR-003.2 | Decide whether the URL can be resolved before downloading the media. | P0 |
| FR-003.3 | Resolve the media URL of the sound. | P0 |
| FR-003.4 | Read the sound title when the page exposes one. | P1 |
| FR-003.5 | Play a preview. Preview may use the downloaded local copy. v1 does not stream from the site during editing. | P0 |
| FR-003.6 | Store the audio in the project working area and continue offline after that. | P0 |
| FR-003.7 | Fail clearly on an unknown host, a removed sound, a non-audio payload, an HTTP error, or a blocked redirect. | P0 |
| FR-003.8 | Keep the original page URL and the resolved media URL in project metadata. | P1 |
| FR-003.9 | Apply the allowlist, size limit, and redirect rules in the security specification. | P0 |

## FR-004 Audio editor

| ID | Requirement | Priority |
| --- | --- | --- |
| FR-004.1 | Show a waveform of the imported audio. | P0 |
| FR-004.2 | Set a start time and an end time on the waveform. | P0 |
| FR-004.3 | Export only the selected range. | P0 |
| FR-004.4 | Preview the selected range. | P0 |
| FR-004.5 | Set a linear volume multiplier from 0 to 1, default 1. | P1 |
| FR-004.6 | Apply fade-in and fade-out in milliseconds. | P2 |
| FR-004.7 | Normalize peak loudness of the selected range. | P2 |
| FR-004.8 | Reset trim, volume, and fades to the imported audio. | P0 |
| FR-004.9 | Show the selected duration and the limit of the active export profile. | P0 |
| FR-004.10 | Refuse to export a selection longer than the active duration preset. Presets are 5000 ms and 10000 ms. The default preset is 10000 ms, which matches the profile ceiling. | P0 |

The upper cap of 10 seconds is a product rule copied from public reports of WhatsApp's own authoring UI. It is not a published third-party specification. The editor also offers a 5 second preset. The profile still stores 10000 ms as the ceiling so the preset can change without a domain rewrite.

## FR-005 Sticker editor

| ID | Requirement | Priority |
| --- | --- | --- |
| FR-005.1 | Show the source sticker on a checkerboard and on solid preview backgrounds. | P0 |
| FR-005.2 | Scale and position the artwork inside the 512 x 512 output frame. | P1 |
| FR-005.3 | Keep transparency through resize and WebP encoding. | P0 |
| FR-005.4 | Add an optional 8 px white outer stroke, off by default. | P2 |
| FR-005.5 | Let the user switch the preview background among transparent, white, black, and a neutral gray. | P1 |
| FR-005.6 | Encode the output image as WebP. | P0 |
| FR-005.7 | Validate output size of 512 x 512 and static file size of at most 100 KB before a standard or bundle export. | P0 |
| FR-005.8 | Edit animated stickers. | P2 |
| FR-005.9 | If the encoded WebP exceeds 100 KB, retry at lower quality and then report a clear failure instead of writing an invalid sticker. | P0 |

v1 may center and fit the image into 512 x 512 without a free-form transform (FR-005.2). Fit must not stretch. Empty space stays transparent.

## FR-006 Association

| ID | Requirement | Priority |
| --- | --- | --- |
| FR-006.1 | Bind one audio asset to one sticker asset. | P0 |
| FR-006.2 | Store that binding as data, independent of the export format. | P0 |
| FR-006.3 | Replace either side of the binding. | P0 |
| FR-006.4 | Play the trimmed audio from the sticker preview. | P0 |
| FR-006.5 | Hold more than one association in a project. | P1 |
| FR-006.6 | Reopen a saved project with the same binding, trim, and export profile. | P0 |
| FR-006.7 | Ask the native export adapter to emit a WhatsApp sound sticker when that adapter is enabled. | P0 interface, disabled implementation |

The v1 interface for FR-006.7 exists and returns "unsupported" until the adapter's verification checklist passes. The UI must not offer it as a working export.

## FR-007 Export

| ID | Requirement | Priority |
| --- | --- | --- |
| FR-007.1 | Export a static WebP that satisfies the published WhatsApp static sticker limits. | P0 |
| FR-007.2 | Export the processed audio as its own file. v1 container is Ogg Opus, mono, 48 kHz. | P0 |
| FR-007.3 | Export a ZIP sound bundle containing the WebP, the audio, and the manifest. | P0 |
| FR-007.4 | Write a manifest that validates against schema 1.0. | P0 |
| FR-007.5 | Run validation before any file is written to the destination. A failed validation leaves the destination unchanged. | P0 |
| FR-007.6 | Call the native sound-sticker adapter through the same export interface. | P0 interface, disabled implementation |
| FR-007.7 | Let the user pick the destination folder on desktop, and download the bundle from Chrome. | P0 |
| FR-007.8 | Name files from the project name plus a timestamp, with characters safe on Windows and macOS. | P0 |

## FR-008 Project library

| ID | Requirement | Priority |
| --- | --- | --- |
| FR-008.1 | Create a project, name it, and save it. | P0 |
| FR-008.2 | Reopen a project from the portable package. | P0 |
| FR-008.3 | List recent projects on desktop. | P1 |
| FR-008.4 | Delete a project from the local library without deleting an already exported bundle. | P1 |

Chrome v1 persists the open project in the extension and round-trips it through the portable package. A cross-device library is out of scope.

## FR-009 Preview

| ID | Requirement | Priority |
| --- | --- | --- |
| FR-009.1 | Show the sticker at roughly chat size and at 512 x 512. | P0 |
| FR-009.2 | Play and stop the trimmed audio while the sticker is visible. | P0 |
| FR-009.3 | Show a speaker mark on the preview so the user can see that the association has audio. The mark is a SolanoStickers preview affordance, not a claim that WhatsApp will render one. | P0 |

## FR-010 Settings

| ID | Requirement | Priority |
| --- | --- | --- |
| FR-010.1 | Switch the UI language between English and Spanish. | P1 |
| FR-010.2 | Show the active export profile and its limits. | P0 |
| FR-010.3 | Remember the last destination folder on desktop. | P1 |

v1 UI copy ships in English. Spanish is P1 and must not block the MVP.

## FR-011 Chrome extension

| ID | Requirement | Priority |
| --- | --- | --- |
| FR-011.1 | Install the editor as a Chrome extension and open it in a full tab. | P0 |
| FR-011.2 | Paste a sticker and a MyInstants URL inside that tab. | P0 |
| FR-011.3 | Import and export the same portable package the desktop app uses. | P0 |
| FR-011.4 | Download the WebP, the audio, and the ZIP. | P0 |
| FR-011.5 | Tell the user when a file needs the desktop app because the browser engine cannot decode or encode it. | P0 |
| FR-011.6 | Do not inject scripts into WhatsApp Web and do not send messages. | P0 |
