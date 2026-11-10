#!/usr/bin/env bash
set -euo pipefail

REF=${KEYFLIP_REF:-}
[[ "$REF" =~ ^[0-9a-f]{40}$ ]] || { echo "deploy-site: set KEYFLIP_REF to a full 40-character lowercase commit SHA (no unpinned deploys)" >&2; exit 1; }
TAG=${REF:0:12}
DIR=${KEYFLIP_DIR:-/opt/keyflip-site}
HOST=${KEYFLIP_HOST:-keyflip.sparkth.io}
HTTPS_PORT=${KEYFLIP_HTTPS_PORT:-443}
PROXY=${KEYFLIP_PROXY_CONTAINER:-spark-virtual-reverse-proxy-1}
TARBALL=${KEYFLIP_TARBALL_URL:-https://codeload.github.com/ongkub/keyflip/tar.gz/$REF}
CURL_EXTRA=${KEYFLIP_CURL_EXTRA:-}

fail() { echo "deploy-site: $*" >&2; exit 1; }
printf 'Deploy settings: REF=%s TAG=%s DIR=%s HOST=%s HTTPS_PORT=%s PROXY=%s TARBALL=%s\n' \
  "$REF" "$TAG" "$DIR" "$HOST" "$HTTPS_PORT" "$PROXY" "$TARBALL"
[[ "${KEYFLIP_ALLOW_NONROOT:-0}" == 1 || "$(id -u)" -eq 0 ]] || fail "run as root (or set KEYFLIP_ALLOW_NONROOT=1 for local testing)"
command -v docker >/dev/null || fail "docker not found"
docker compose version >/dev/null 2>&1 || fail "docker compose plugin not found"
command -v curl >/dev/null || fail "curl not found"
command -v tar >/dev/null || fail "tar not found"
docker inspect "$PROXY" >/dev/null 2>&1 || fail "Traefik container '$PROXY' not found (set KEYFLIP_PROXY_CONTAINER)"

NETS="$(docker inspect "$PROXY" --format '{{range $k, $v := .NetworkSettings.Networks}}{{$k}}{{"\n"}}{{end}}' | sed '/^$/d')"
if [[ -n "${KEYFLIP_TRAEFIK_NETWORK:-}" ]]; then
  NET="$KEYFLIP_TRAEFIK_NETWORK"
  grep -qxF "$NET" <<<"$NETS" || fail "Traefik is not on network '$NET' (it is on: $(echo "$NETS"))"
elif [[ -n "$NETS" && "$(wc -l <<<"$NETS")" -eq 1 ]]; then
  NET="$NETS"
else
  fail "Traefik is on several networks ($(echo "$NETS")); re-run with KEYFLIP_TRAEFIK_NETWORK=<the one it routes on>"
fi
printf 'Effective Traefik network: %s\n' "$NET"

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
curl -fsSL "$TARBALL" -o "$STAGE/source.tar.gz" || fail "source download failed; nothing was changed"
mkdir "$STAGE/extract"
tar -xzf "$STAGE/source.tar.gz" --no-same-owner --no-same-permissions -C "$STAGE/extract" || fail "could not extract the source tarball"
TOP_COUNT="$(find "$STAGE/extract" -mindepth 1 -maxdepth 1 -type d -print | wc -l | tr -d ' ')"
[[ "$TOP_COUNT" == 1 ]] || fail "expected exactly one top-level directory in source archive"
TOP="$(find "$STAGE/extract" -mindepth 1 -maxdepth 1 -type d -print -quit)"
SRC="$TOP/site"
[[ -d "$SRC" && -f "$SRC/index.html" ]] || fail "site/index.html missing from source archive"
if find "$SRC" -type l -print -quit | grep -q .; then fail "source site contains a symlink"; fi

CTX="$STAGE/context"
mkdir -p "$CTX/html/download" "$CTX/html/assets" "$CTX/html/fonts" "$CTX/html/design"
cp "$SRC/deploy/Dockerfile" "$SRC/deploy/nginx.conf" "$SRC/deploy/security-headers.conf" "$CTX/"
cp "$SRC/index.html" "$SRC/404.html" "$CTX/html/"
cp -R "$SRC/download/." "$CTX/html/download/"
cp -R "$SRC/assets/." "$CTX/html/assets/"
cp -R "$SRC/fonts/." "$CTX/html/fonts/"
cp "$SRC/design/tokens.css" "$CTX/html/design/tokens.css"


# Roll back only to what is really running: a healthy container whose image still exists.
PREV=
if [[ "$(docker inspect --format '{{.State.Health.Status}}' keyflip-site-site-1 2>/dev/null || true)" == healthy ]]; then
  img="$(docker inspect --format '{{.Config.Image}}' keyflip-site-site-1 2>/dev/null || true)"
  [[ "$img" == keyflip-site:* ]] && PREV="${img#keyflip-site:}"
  if [[ -n "$PREV" ]] && ! docker image inspect "keyflip-site:$PREV" >/dev/null 2>&1; then PREV=; fi
fi
if [[ -n "$PREV" ]]; then echo "Rollback target: keyflip-site:$PREV (running and healthy)"
else echo "No healthy site is running: if this deploy fails, the site is removed and the receiver alone serves $HOST"; fi

docker build -t "keyflip-site:$TAG" "$CTX" || fail "image build failed; running deployment was not changed"
install -d "$DIR"
if [[ -f "$DIR/docker-compose.yml" ]]; then cp "$DIR/docker-compose.yml" "$STAGE/compose.prev"; fi
cp "$SRC/deploy/docker-compose.yml" "$DIR/docker-compose.yml"
write_env() {
  local tag="$1" tmp="$DIR/.site.env.$$"
  (umask 077; printf 'SITE_TAG=%s\nTRAEFIK_NETWORK=%s\nKEYFLIP_HOST=%s\n' "$tag" "$NET" "$HOST" > "$tmp")
  mv "$tmp" "$DIR/site.env"
}
COMPOSE=(docker compose -p keyflip-site --env-file "$DIR/site.env" -f "$DIR/docker-compose.yml")

CURL_ARGS=(-sS --max-time 15 --resolve "$HOST:$HTTPS_PORT:127.0.0.1")   # no -f: 401/404 bodies and headers are checked too
EXTRA_ARGS=()
if [[ -n "${CURL_EXTRA// /}" ]]; then read -r -a EXTRA_ARGS <<<"$CURL_EXTRA"; fi
CURL_ARGS+=(${EXTRA_ARGS[@]+"${EXTRA_ARGS[@]}"})
BASE="https://$HOST:$HTTPS_PORT"
request() { curl "${CURL_ARGS[@]}" -D "$STAGE/headers" -o "$STAGE/body" -w '%{http_code}' "$@"; }
header_has() { tr -d '\r' < "$STAGE/headers" | grep -Eiq "^$1:[[:space:]]*$2"; }
content_type_has() { header_has 'content-type' "$1"; }
# Traefik only routes to a healthy container, so until then the receiver answers the site's paths.
wait_healthy() {
  HEALTH=
  for _ in $(seq 1 30); do
    HEALTH="$(docker inspect --format '{{.State.Health.Status}}' keyflip-site-site-1 2>/dev/null || true)"
    [[ "$HEALTH" == healthy ]] && return 0
    sleep 2
  done
  return 1
}
# Every failure after this point ends here: back to the previous healthy version, or no site at all (the receiver
# then answers every path on the host again). Never leaves an unverified version routed.
restore() {
  local reason="$1"
  trap - INT TERM
  if [[ -n "$PREV" ]]; then
    if [[ -f "$STAGE/compose.prev" ]]; then cp "$STAGE/compose.prev" "$DIR/docker-compose.yml"; fi
    write_env "$PREV"
    if "${COMPOSE[@]}" up -d && wait_healthy; then
      sleep 3   # give Traefik a moment to add the router back
      local health_code home_code
      health_code="$(request "$BASE/healthz" 2>/dev/null || true)"
      home_code="$(request "$BASE/" 2>/dev/null || true)"
      echo "deploy-site: $reason; rolled back to $PREV (checks: /healthz=$health_code /=$home_code)" >&2
      exit 1
    fi
    echo "deploy-site: the previous version $PREV did not come back healthy either" >&2
  fi
  "${COMPOSE[@]}" down || true
  rm -f "$DIR/site.env"
  echo "deploy-site: $reason; the site was removed, so the receiver alone serves $HOST (re-run with a good KEYFLIP_REF)" >&2
  exit 1
}

write_env "$TAG"
trap 'restore "interrupted"' INT TERM
"${COMPOSE[@]}" up -d || restore "compose start failed"
echo "Waiting up to 60 seconds for the site container to report healthy..."
wait_healthy || true

PASS_COUNT=0
FAIL_COUNT=0
check() {
  local name="$1" ok="$2"
  if [[ "$ok" == 1 ]]; then printf 'PASS: %s\n' "$name"; PASS_COUNT=$((PASS_COUNT + 1));
  else printf 'FAIL: %s\n' "$name"; FAIL_COUNT=$((FAIL_COUNT + 1)); fi
}
first_ok=0
for _ in $(seq 1 10); do
  if code="$(request "$BASE/")" && [[ "$code" == 200 ]] && content_type_has 'text/html' && grep -q 'KeyFlip' "$STAGE/body" && header_has 'content-security-policy' '.+'; then first_ok=1; break; fi
  sleep 2
done
check 'GET / returns KeyFlip HTML with CSP' "$first_ok"

code="$(request "$BASE/download" 2>/dev/null || true)"
if [[ "$code" == 301 ]] && header_has 'location' '/download/'; then check 'GET /download redirects to /download/' 1; else check 'GET /download redirects to /download/' 0; fi
code="$(request "$BASE/download/" 2>/dev/null || true)"
if [[ "$code" == 200 ]] && content_type_has 'text/html'; then check 'GET /download/ returns 200' 1; else check 'GET /download/ returns 200' 0; fi
code="$(request "$BASE/assets/site.css" 2>/dev/null || true)"
if [[ "$code" == 200 ]] && header_has 'content-security-policy' '.+'; then check 'GET /assets/site.css returns 200 with CSP' 1; else check 'GET /assets/site.css returns 200 with CSP' 0; fi
code="$(request "$BASE/assets/demo/keyflip-demo.th.vtt" 2>/dev/null || true)"
if [[ "$code" == 200 ]] && content_type_has 'text/vtt'; then check 'GET demo captions uses text/vtt' 1; else check 'GET demo captions uses text/vtt' 0; fi
code="$(request -H 'Range: bytes=0-99' "$BASE/assets/demo/keyflip-demo.mp4" 2>/dev/null || true)"
if [[ "$code" == 206 ]]; then check 'MP4 byte range returns 206' 1; else check 'MP4 byte range returns 206' 0; fi
code="$(request "$BASE/healthz" 2>/dev/null || true)"
if [[ "$code" == 200 && "$(cat "$STAGE/body" 2>/dev/null || true)" == ok ]]; then check 'receiver GET /healthz remains ok' 1; else check 'receiver GET /healthz remains ok' 0; fi
code="$(request "$BASE/admin" 2>/dev/null || true)"
if [[ "$code" == 401 ]]; then check 'receiver GET /admin remains 401' 1; else check 'receiver GET /admin remains 401' 0; fi
code="$(request "$BASE/v1/reports" 2>/dev/null || true)"
if [[ "$code" == 404 ]] && content_type_has 'application/json'; then check 'receiver GET /v1/reports remains JSON' 1; else check 'receiver GET /v1/reports remains JSON' 0; fi

[[ "$HEALTH" == healthy ]] || restore "site container did not become healthy"
[[ "$FAIL_COUNT" -eq 0 ]] || restore "$FAIL_COUNT public checks failed"
trap - INT TERM

echo "Deployed tag: $TAG"
echo "Public checks: curl https://$HOST/ ; curl -I https://$HOST/download ; curl https://$HOST/healthz ; curl -i https://$HOST/admin ; curl -i https://$HOST/v1/reports"
echo "Roll back by hand: set SITE_TAG=<older tag> in $DIR/site.env, then docker compose -p keyflip-site --env-file $DIR/site.env -f $DIR/docker-compose.yml up -d"
