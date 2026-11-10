# KeyFlip website (`keyflip.sparkth.io`)

Static, Thai-first, no build step and no third-party requests.

- `index.html` — landing page · `download/index.html` — download page
- `assets/` — page CSS/JS, `pairs.json` (Kedmanee keys for the in-page demo), `favicon.svg`, `og.png`, `demo/` (video)
- `DESIGN.md` + `design/` — the design kit every agent must read first (see `../AGENTS.md`); `design/bench.html`
  is an internal test page, `design/og.html` is the source of `assets/og.png`
- `fonts/` — self-hosted Thai + Latin subsets (SIL OFL 1.1)

## Preview locally
From the repo root: `python3 -m http.server 8765 --bind 127.0.0.1`, then open http://localhost:8765/site/

## When a new version is released
Update in `download/index.html`: version, file names, sizes, download URLs, both SHA-256 values, release-notes link;
in `index.html`: the version line under the hero, JSON-LD `softwareVersion`, footer. Check with
`grep -rn "0\.2\.0" site/ --include=*.html`.

## Share image
`"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --headless=new --hide-scrollbars --force-device-scale-factor=1 --window-size=1200,630 --virtual-time-budget=4000 --screenshot=site/assets/og.png http://localhost:8765/site/design/og.html`

## Demo video
Recorded on a real Mac with `app/tools/kfdemo.swift` driving each app with physical key codes while
`screencapture -v` records only the typing area; clips are cropped, labelled and joined with ffmpeg, captions are
`assets/demo/keyflip-demo.th.vtt`. Re-record when the app's behaviour changes visibly; never record windows that
show private content (crop to the typing area and check every clip before committing).

## Deploy (keyflip.sparkth.io)
Run as root on the VPS (Hostinger Web console), pinned to a pushed commit:
`REF=<40-char sha>; curl -fsSL https://raw.githubusercontent.com/ongkub/keyflip/$REF/site/deploy/deploy-site.sh | KEYFLIP_REF=$REF bash 2>&1 | tee /root/keyflip-site-deploy.log`
It builds `keyflip-site:<sha12>` (nginx, non-root, read-only), routes the host except `/v1`, `/admin`, `/healthz`
(those stay with the report receiver), runs 9 checks through Traefik and rolls back to the previous healthy tag on
any failure (or removes the site on a failed first deploy). Files live in `/opt/keyflip-site`.
