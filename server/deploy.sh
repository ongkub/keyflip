#!/usr/bin/env bash
set -euo pipefail

# Owner-run deployment. From the repo root, copy this directory to the host,
# then run: ssh root@HOST 'bash -s' < server/deploy.sh
SRC_DIR="${KEYFLIP_FEEDBACK_SOURCE:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)}"
INSTALL_DIR=/opt/keyflip-feedback
ENV_FILE=/etc/keyflip-feedback.env
UNIT_FILE=/etc/systemd/system/keyflip-feedback.service
CADDYFILE=/etc/caddy/Caddyfile

if [[ "$(id -u)" -ne 0 ]]; then
  echo "Run this script as root." >&2
  exit 1
fi

install -d -m 0755 "$INSTALL_DIR"
install -m 0644 "$SRC_DIR/feedback_server.py" "$INSTALL_DIR/feedback_server.py"
install -m 0644 "$SRC_DIR/keyflip-feedback.service" "$UNIT_FILE"

if [[ ! -e "$ENV_FILE" ]]; then
  install -d -m 0755 "$(dirname "$ENV_FILE")"
  admin_password="$(python3 -c 'import secrets; print(secrets.token_urlsafe(32))')"
  ip_salt="$(python3 -c 'import secrets; print(secrets.token_urlsafe(32))')"
  umask 077
  printf 'KEYFLIP_ADMIN_PASSWORD=%s\nKEYFLIP_IP_SALT=%s\nKEYFLIP_DB=/var/lib/keyflip-feedback/reports.db\nKEYFLIP_BIND=127.0.0.1:8787\n' \
    "$admin_password" "$ip_salt" > "$ENV_FILE"
  chmod 0600 "$ENV_FILE"
  unset admin_password ip_salt
  echo "Generated admin password is stored in $ENV_FILE; read it there over a secure root session."
else
  chmod 0600 "$ENV_FILE"
  echo "Keeping existing environment file: $ENV_FILE"
fi

if ! grep -Fq 'keyflip.sparkth.io' "$CADDYFILE"; then
  cat "$SRC_DIR/Caddyfile.snippet" >> "$CADDYFILE"
fi

systemctl daemon-reload
systemctl enable --now keyflip-feedback.service
caddy validate --config "$CADDYFILE"
systemctl reload caddy
curl --fail --silent --show-error http://127.0.0.1:8787/healthz
echo
echo "Deployment complete."
