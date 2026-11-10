# KeyFlip problem report schema v1

Clients send `POST https://keyflip.sparkth.io/v1/reports` with
`Content-Type: application/json` and a UTF-8 JSON body no larger than 16 KiB.

```json
{
  "schema": 1,
  "app": "keyflip",
  "platform": "macos",
  "app_version": "0.2.0",
  "os_version": "macOS 27.2",
  "model": "personal",
  "enabled": true,
  "stats_today": {"runs": 120, "conversions": 4, "undos": 1},
  "message": "what happened (required, 1–2000 chars)",
  "contact": "optional, at most 200 chars",
  "recent_words": [{"word": "vpkd", "result": "อยาก", "kind": "typing"}],
  "locale": "th-TH"
}
```

`schema` must be `1`; `app` must be `keyflip`; `platform` is `macos` or
`windows`; `model` is `personal`, `open`, or `generic`; and `enabled` is a
boolean. `app_version`, `os_version`, and `locale` are strings of at most 100,
200, and 35 characters respectively. `stats_today` contains non-negative
integer `runs`, `conversions`, and `undos` counters. `message` is required and
contains 1–2000 characters. `contact` is optional and contains at most 200
characters.

`recent_words` is omitted unless the user explicitly opts in to attaching the
last converted words. It contains at most 10 objects, each with `word` and
`result` strings of at most 40 characters and `kind` equal to `typing`. Only
converted or undone words from the local events log may be included. Clients
must show the exact JSON to the user in a read-only preview before sending; no
other typed text is included.

Success returns HTTP `201` with `{"ok":true,"id":N}`. Invalid input returns
`400`, a body over 16 KiB returns `413`, and a rate limit returns `429`; error
bodies have the form `{"ok":false,"error":"..."}`.
