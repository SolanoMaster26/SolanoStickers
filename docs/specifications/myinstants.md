# MyInstants provider specification

## Role

`MyInstantsProvider` is the only component that understands MyInstants URLs and pages. The rest of the app sees a resolved sound:

```text
ResolvedSound
  page_url: string | null
  media_url: string
  title: string | null
  bytes: stored by the import pipeline, not by the UI
```

Adding another sound site later means adding another provider. It does not mean editing the editor.

## Allowed inputs

| Input | Example shape | Action |
| --- | --- | --- |
| Instant page | `https://www.myinstants.com/<locale>/instant/<slug>/` | Fetch HTML, then the media URL |
| Instant page without locale | `https://www.myinstants.com/instant/<slug>/` | Same |
| Direct media | `https://www.myinstants.com/media/sounds/<file>.mp3` | Download the media. Title stays null unless a later metadata fetch is implemented |

Query strings on an instant page are ignored for identity and must not be forwarded to a different host.

## Allowlist

A request is sent only when every hop matches:

- scheme `https`
- host `www.myinstants.com` or `myinstants.com`
- port empty or 443
- no userinfo
- DNS results and redirect targets are public addresses. Reject loopback, link-local, private, and unique-local ranges

Maximum redirects: 2. A redirect to any other host fails with `url-rejected`.

Download timeout: 20 seconds. Maximum body: the NFR-014 audio limit.

The provider refuses `file:`, `data:`, `blob:`, raw IP hosts, and internationalized hosts that punycode to a non-allowlisted name.

Chrome uses the same rules before `fetch`. Host permission is limited to `https://www.myinstants.com/*` and `https://myinstants.com/*`.

## Page extraction

Extraction is a pure function from HTML text to `{ title, media_url }`.

v1 strategy, in order, first match wins:

1. A media URL on `myinstants.com` whose path starts with `/media/sounds/` and whose suffix is `.mp3`, `.wav`, `.ogg`, or `.m4a`.
2. If none is found, fail with `remote-unavailable` and the message that the sound could not be read. Do not guess.

Title, when present:

1. `og:title`
2. else the document `<title>`
3. else null

Strip a trailing site suffix such as " - Myinstants" from the title. Cap the stored title at 120 characters.

The extractor is unit-tested with saved HTML fixtures. Tests do not call the network.

If MyInstants changes its markup, only the extractor and its fixtures change.

## Download and verify

After extraction:

1. Download the media URL under the same allowlist.
2. Sniff the bytes with the audio importer.
3. If the payload is not audio, delete it and return `unsupported-audio`.
4. On success, store `source.type = "myinstants"`, `source.page_url`, and `source.media_url`.

Preview playback uses the stored bytes. The editor does not keep a live connection to MyInstants.

## Failures

| Situation | Code |
| --- | --- |
| Host not on the allowlist | `url-rejected` |
| Redirect leaves the allowlist or exceeds 2 hops | `url-rejected` |
| DNS or connection failure, HTTP 404, HTTP 410, HTTP 5xx, timeout | `remote-unavailable` |
| HTML has no media URL | `remote-unavailable` |
| Body exceeds the size limit | `file-too-large` |
| Body is not audio | `unsupported-audio` |

## Rights metadata

The manifest records the source URL so the user can see where the clip came from. The app does not add a license field that claims the clip is reusable.

The about screen states that sounds from MyInstants stay the property of their owners and that SolanoStickers does not grant a license to those sounds.

v1 does not embed a search box, a trending list, or a cache shared between projects. Each project downloads a sound only after the user pastes its URL.
