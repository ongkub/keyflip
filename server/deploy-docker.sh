#!/usr/bin/env bash
# Deploy the KeyFlip report receiver as a Docker container behind the existing Traefik. Safe to run again.
# Run as root on the server (e.g. Hostinger Web console):
#   curl -fsSL https://raw.githubusercontent.com/ongkub/keyflip/main/server/deploy-docker.sh | bash
# It does not touch Traefik's configuration or any other stack: Traefik discovers the container by its labels.
# Remove later with:  cd /opt/keyflip-feedback && docker compose --env-file proxy.env down   (add -v to delete reports)
set -euo pipefail

DIR=/opt/keyflip-feedback
RAW=${KEYFLIP_RAW:-https://raw.githubusercontent.com/ongkub/keyflip/main/server}
PROXY_CONTAINER=${KEYFLIP_PROXY_CONTAINER:-spark-virtual-reverse-proxy-1}

fail() { echo "deploy: $*" >&2; exit 1; }
rand() { python3 -c 'import secrets; print(secrets.token_urlsafe(32))' 2>/dev/null || openssl rand -hex 32; }

[[ "$(id -u)" -eq 0 ]] || fail "run as root"
command -v docker >/dev/null || fail "docker not found"
docker compose version >/dev/null 2>&1 || fail "docker compose plugin not found"
docker inspect "$PROXY_CONTAINER" >/dev/null 2>&1 || fail "Traefik container '$PROXY_CONTAINER' not found (set KEYFLIP_PROXY_CONTAINER)"

NET="$(docker inspect "$PROXY_CONTAINER" --format '{{range $k, $v := .NetworkSettings.Networks}}{{$k}}{{"\n"}}{{end}}' | head -n 1)"
[[ -n "$NET" ]] || fail "could not find the Traefik container's network"
SUBNETS="$(docker network inspect "$NET" --format '{{range .IPAM.Config}}{{.Subnet}},{{end}}' | sed 's/,$//')"
[[ -n "$SUBNETS" ]] || fail "could not read the subnet of network '$NET'"

install -d -m 0755 "$DIR"
for f in Dockerfile docker-compose.yml feedback_server.py; do
  curl -fsSL "$RAW/$f" -o "$DIR/$f.new"
  mv "$DIR/$f.new" "$DIR/$f"
done

if [[ ! -f "$DIR/.env" ]]; then
  ( umask 077
    printf 'KEYFLIP_ADMIN_PASSWORD=%s\nKEYFLIP_IP_SALT=%s\n' "$(rand)" "$(rand)" > "$DIR/.env" )
  echo "Generated an admin password in $DIR/.env (root only). Read it with: grep ADMIN $DIR/.env"
else
  echo "Keeping existing $DIR/.env"
fi
chmod 600 "$DIR/.env"
# Not secret: which Docker network Traefik uses and which addresses may set X-Forwarded-For.
printf 'TRAEFIK_NETWORK=%s\nKEYFLIP_TRUSTED_PROXIES=%s\n' "$NET" "$SUBNETS" > "$DIR/proxy.env"

cd "$DIR"
docker compose --env-file proxy.env up -d --build

echo "Waiting for the container to report healthy..."
for _ in $(seq 1 30); do
  state="$(docker inspect --format '{{.State.Health.Status}}' keyflip-feedback-feedback-1 2>/dev/null || true)"
  [[ "$state" == "healthy" ]] && break
  sleep 2
done
[[ "${state:-}" == "healthy" ]] || fail "container is not healthy yet; check: docker compose --env-file proxy.env logs"

# Through Traefik on this machine (-k: the Let's Encrypt certificate can take a minute on first deploy).
if curl -fsSk --resolve keyflip.sparkth.io:443:127.0.0.1 https://keyflip.sparkth.io/healthz >/dev/null; then
  echo "Traefik routes keyflip.sparkth.io to the receiver."
else
  echo "Container is healthy but Traefik has not picked up the route yet; wait a few seconds and re-check."
fi
echo "Done. Public check: curl https://keyflip.sparkth.io/healthz   Admin: https://keyflip.sparkth.io/admin (user: admin)"
