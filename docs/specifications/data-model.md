# Data model

The domain is three resources plus a project. Export formats are projections of this model. WhatsApp is not part of the model.

## Aggregates

```text
Project
  id
  name
  created_at
  updated_at
  schema_version
  stickers[]
  audios[]
  associations[]
  active_association_id

StickerAsset
  id
  file            # path relative to the project root
  source_name
  format          # png | jpeg | webp
  width
  height
  has_alpha
  animated
  edit

AudioAsset
  id
  file
  source_name
  format
  duration_ms
  sample_rate
  channels
  source          # local | myinstants
  edit

Association
  id
  sticker_id
  audio_id
  created_at
```

Trim and volume live on `AudioAsset.edit` in v1 because one audio belongs to one association. When batch editing arrives, move the edit onto the association if two associations must trim the same asset differently. The schema already puts the edit numbers on the association as well, and v1 writes the same values in both places. Readers prefer the association values.

## Portable package

A portable project is a directory or a ZIP with this layout:

```text
<project>/
  manifest.json
  assets/
    stk_<id>.<ext>
    aud_<id>.<ext>
```

The desktop app may store that directory uncompressed and zip it on export. Chrome downloads it as a `.solanostickers.zip`.

`manifest.json` validates against [solanostickers-project-1.0.schema.json](schemas/solanostickers-project-1.0.schema.json).

Rules:

- Every `file` value is a relative POSIX path starting with `assets/` and containing no `..`.
- Every association references existing sticker and audio ids.
- v1 writers emit zero or one sticker, zero or one audio, and zero or one association.
- Readers accept any count the schema allows, and the v1 editor opens the active association. If several exist and `active_association_id` is missing, open the first and do not delete the others.

## Manifest example

```json
{
  "schema_version": "1.0",
  "project": {
    "id": "prj_01HF7K000000",
    "name": "Airhorn cat",
    "created_at": "2026-11-24T20:00:00Z",
    "updated_at": "2026-11-24T20:05:00Z"
  },
  "stickers": [
    {
      "id": "stk_a1b2c3d4e5f6",
      "file": "assets/stk_a1b2c3d4e5f6.png",
      "source_name": "cat.png",
      "format": "png",
      "width": 640,
      "height": 480,
      "has_alpha": true,
      "animated": false,
      "edit": {
        "fit": "contain",
        "offset_x": 0,
        "offset_y": 0,
        "scale": 1,
        "stroke": "none"
      }
    }
  ],
  "audios": [
    {
      "id": "aud_ff00aa11bb22",
      "file": "assets/aud_ff00aa11bb22.mp3",
      "source_name": "airhorn.mp3",
      "format": "mp3",
      "duration_ms": 30000,
      "sample_rate": 44100,
      "channels": 2,
      "source": {
        "type": "myinstants",
        "page_url": "https://www.myinstants.com/en/instant/airhorn/",
        "media_url": "https://www.myinstants.com/media/sounds/airhorn.mp3",
        "title": "Airhorn"
      }
    }
  ],
  "associations": [
    {
      "id": "asc_001122334455",
      "sticker_id": "stk_a1b2c3d4e5f6",
      "audio_id": "aud_ff00aa11bb22",
      "start_ms": 0,
      "end_ms": 8500,
      "volume": 1,
      "fade_in_ms": 0,
      "fade_out_ms": 0
    }
  ],
  "active_association_id": "asc_001122334455",
  "export": {
    "profile": "sound-bundle",
    "target": "whatsapp"
  }
}
```

`export` records the last requested profile. It does not mean the bundle has been verified by WhatsApp.

## Desktop index

SQLite table `projects`:

| Column | Type | Notes |
| --- | --- | --- |
| id | text primary key | project id |
| name | text | |
| package_path | text | absolute path of the saved package |
| updated_at | text | ISO-8601 UTC |
| schema_version | text | |

The index is rebuilt if a path is missing. Deleting an index row does not delete an exported sound bundle in a different folder.

## Times

All timestamps in the manifest are UTC with a `Z` suffix and second precision.

## Schema evolution

`schema_version` is a string `"major.minor"`.

- Minor versions may add optional fields. Old readers ignore unknown fields only if a future decision says so. v1 readers reject unknown required fields and reject a major version other than 1.
- A native WhatsApp profile does not require a new major version.
- Breaking changes bump the major version and ship a migrator. v1 has no migrator because there is no previous version.
