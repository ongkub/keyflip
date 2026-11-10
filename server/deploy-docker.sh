#!/usr/bin/env bash
# Deploy the KeyFlip report receiver as a Docker container behind the existing Traefik. Safe to run again.
# Run as root on the server (e.g. Hostinger Web console), pinned to one commit so nothing changes underneath you:
#   REF=<commit-sha>; curl -fsSL https://raw.githubusercontent.com/ongkub/keyflip/$REF/server/deploy-docker.sh | KEYFLIP_REF=$REF bash
# It does not touch Traefik's configuration or any other stack: Traefik discovers the container by its labels.
# Remove later with:  cd /opt/keyflip-feedback && docker compose --env-file proxy.env down   (add -v to delete reports)
set -euo pipefail

DIR=/opt/keyflip-feedback
REF=${KEYFLIP_REF:-}
[[ "$REF" =~ ^[0-9a-f]{40}$ ]] || { echo "deploy: set KEYFLIP_REF to a full 40-character commit SHA (no unpinned deploys)" >&2; exit 1; }
RAW=${KEYFLIP_RAW:-https://raw.githubusercontent.com/ongkub/keyflip/$REF/server}
PROXY_CONTAINER=${KEYFLIP_PROXY_CONTAINER:-spark-virtual-reverse-proxy-1}

fail() { echo "deploy: $*" >&2; exit 1; }
rand() { python3 -c 'import secrets; print(secrets.token_urlsafe(32))' 2>/dev/null || openssl rand -hex 32; }

[[ "$(id -u)" -eq 0 ]] || fail "run as root"
command -v docker >/dev/null || fail "docker not found"
docker compose version >/dev/null 2>&1 || fail "docker compose plugin not found"
docker inspect "$PROXY_CONTAINER" >/dev/null 2>&1 || fail "Traefik container '$PROXY_CONTAINER' not found (set KEYFLIP_PROXY_CONTAINER)"

# The network Traefik routes on. Refuse to guess when it is attached to more than one.
NETS="$(docker inspect "$PROXY_CONTAINER" --format '{{range $k, $v := .NetworkSettings.Networks}}{{$k}}{{"\n"}}{{end}}' | sed '/^$/d')"
if [[ -n "${KEYFLIP_TRAEFIK_NETWORK:-}" ]]; then
  NET="$KEYFLIP_TRAEFIK_NETWORK"
  grep -qxF "$NET" <<<"$NETS" || fail "Traefik is not on network '$NET' (it is on: $(echo $NETS))"
elif [[ -n "$NETS" && "$(wc -l <<<"$NETS")" -eq 1 ]]; then
  NET="$NETS"
else
  fail "Traefik is on several networks ($(echo $NETS)); re-run with KEYFLIP_TRAEFIK_NETWORK=<the one it routes on>"
fi

install -d -m 0755 "$DIR"
# Download into a temporary folder first: a failed download changes nothing.
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
for f in Dockerfile docker-compose.yml feedback_server.py; do
  curl -fsSL "$RAW/$f" -o "$STAGE/$f" || fail "download of $f failed; nothing was changed"
done
# Keep the running version only if it is healthy, so a failed update never overwrites the good copy.
if [[ "$(docker inspect --format '{{.State.Health.Status}}' keyflip-feedback-feedback-1 2>/dev/null || true)" == "healthy" ]]; then
  rm -rf "$DIR/previous"; mkdir -p "$DIR/previous"
  cp -p "$DIR/Dockerfile" "$DIR/docker-compose.yml" "$DIR/feedback_server.py" "$DIR/previous/"
fi
cp "$STAGE/"* "$DIR/"

if [[ ! -f "$DIR/.env" ]]; then
  ( umask 077
    printf 'KEYFLIP_ADMIN_PASSWORD=%s\nKEYFLIP_IP_SALT=%s\n' "$(rand)" "$(rand)" > "$DIR/.env" )
  echo "Generated an admin password in $DIR/.env (root only). Read it with: grep ADMIN $DIR/.env"
else
  echo "Keeping existing $DIR/.env"
fi
chmod 600 "$DIR/.env"
# Not secret: which Docker network Traefik uses, and that only the Traefik container (resolved by name through
# Docker DNS) may set X-Forwarded-For. 127.0.0.1 stays trusted for local checks inside the container only.
printf 'TRAEFIK_NETWORK=%s\nKEYFLIP_TRUSTED_PROXY_HOSTS=%s\nKEYFLIP_TRUSTED_PROXIES=127.0.0.1/32\n' "$NET" "$PROXY_CONTAINER" > "$DIR/proxy.env"

restore() {
  if [[ -d "$DIR/previous" ]]; then
    cp -p "$DIR/previous/"* "$DIR/"
    docker compose --env-file proxy.env up -d --build || true
    fail "$1; restored the previous version (logs: cd $DIR && docker compose --env-file proxy.env logs)"
  fi
  fail "$1 (logs: cd $DIR && docker compose --env-file proxy.env logs)"
}

cd "$DIR"
docker compose --env-file proxy.env up -d --build || restore "build or start failed"

echo "Waiting for the container to report healthy..."
for _ in $(seq 1 30); do
  state="$(docker inspect --format '{{.State.Health.Status}}' keyflip-feedback-feedback-1 2>/dev/null || true)"
  [[ "$state" == "healthy" ]] && break
  sleep 2
done
[[ "${state:-}" == "healthy" ]] || restore "the new version did not become healthy"

# Through Traefik on this machine (-k: the Let's Encrypt certificate can take a minute on first deploy).
if curl -fsSk --resolve keyflip.sparkth.io:443:127.0.0.1 https://keyflip.sparkth.io/healthz >/dev/null; then
  echo "Traefik routes keyflip.sparkth.io to the receiver."
else
  echo "Container is healthy but Traefik has not picked up the route yet; wait a few seconds and re-check."
fi
echo "Done. Public check: curl https://keyflip.sparkth.io/healthz   Admin: https://keyflip.sparkth.io/admin (user: admin)"
