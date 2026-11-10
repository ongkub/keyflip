#!/usr/bin/env bash
# Owner-run deployment of the KeyFlip report receiver. Safe to run again (idempotent).
#
#   scp -r server root@HOST:/tmp/keyflip-feedback
#   ssh root@HOST 'bash /tmp/keyflip-feedback/deploy.sh'
#
# It never edits other Caddy sites: it appends one site block only if keyflip.sparkth.io is absent, validates the
# whole config, and restores the previous Caddyfile automatically if validation fails.
set -euo pipefail

SRC_DIR="${KEYFLIP_FEEDBACK_SOURCE:-$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)}"
INSTALL_DIR=/opt/keyflip-feedback
ENV_FILE=/etc/keyflip-feedback.env
UNIT_FILE=/etc/systemd/system/keyflip-feedback.service
CADDYFILE="${KEYFLIP_CADDYFILE:-/etc/caddy/Caddyfile}"
PORT=8787

fail() { echo "deploy: $*" >&2; exit 1; }

[[ "$(id -u)" -eq 0 ]] || fail "run as root"
[[ -f "$SRC_DIR/feedback_server.py" ]] || fail "server files not found in $SRC_DIR (copy the server/ folder first)"
[[ -f "$CADDYFILE" ]] || fail "Caddyfile not found at $CADDYFILE (set KEYFLIP_CADDYFILE)"
command -v caddy >/dev/null || fail "caddy command not found"
command -v python3 >/dev/null || fail "python3 not found"

# The port must be free unless it is already ours.
if ss -ltn "sport = :$PORT" | grep -q ":$PORT" && ! systemctl is-active --quiet keyflip-feedback.service; then
  fail "127.0.0.1:$PORT is used by another program; set KEYFLIP_BIND in $ENV_FILE and the Caddy snippet to a free port"
fi

install -d -m 0755 "$INSTALL_DIR"
install -m 0644 "$SRC_DIR/feedback_server.py" "$INSTALL_DIR/feedback_server.py"
install -m 0644 "$SRC_DIR/keyflip-feedback.service" "$UNIT_FILE"

if [[ ! -e "$ENV_FILE" ]]; then
  umask 077
  admin_password="$(python3 -c 'import secrets; print(secrets.token_urlsafe(32))')"
  ip_salt="$(python3 -c 'import secrets; print(secrets.token_urlsafe(32))')"
  printf 'KEYFLIP_ADMIN_PASSWORD=%s\nKEYFLIP_IP_SALT=%s\nKEYFLIP_DB=/var/lib/keyflip-feedback/reports.db\nKEYFLIP_BIND=127.0.0.1:%s\n' \
    "$admin_password" "$ip_salt" "$PORT" > "$ENV_FILE"
  unset admin_password ip_salt
  chmod 0600 "$ENV_FILE"
  echo "Generated the admin password in $ENV_FILE (root only). Read it with: sudo grep ADMIN $ENV_FILE"
else
  chmod 0600 "$ENV_FILE"
  echo "Keeping existing $ENV_FILE"
fi

systemctl daemon-reload
systemctl enable --now keyflip-feedback.service
systemctl restart keyflip-feedback.service

if ! grep -Fq 'keyflip.sparkth.io' "$CADDYFILE"; then
  backup="$CADDYFILE.bak-keyflip-$(date +%Y%m%d-%H%M%S)"
  cp -p "$CADDYFILE" "$backup"
  { printf '\n'; cat "$SRC_DIR/Caddyfile.snippet"; } >> "$CADDYFILE"
  if ! caddy validate --config "$CADDYFILE" --adapter caddyfile; then
    cp -p "$backup" "$CADDYFILE"
    fail "new Caddy config did not validate; restored $CADDYFILE from $backup (Caddy was not reloaded)"
  fi
  echo "Added keyflip.sparkth.io to $CADDYFILE (backup: $backup)"
else
  caddy validate --config "$CADDYFILE" --adapter caddyfile || fail "existing Caddy config does not validate; not reloading"
fi
systemctl reload caddy

for _ in 1 2 3 4 5; do
  if curl --fail --silent http://127.0.0.1:$PORT/healthz >/dev/null; then break; fi
  sleep 1
done
curl --fail --silent --show-error http://127.0.0.1:$PORT/healthz && echo
echo "Deployment complete. Public check: curl https://keyflip.sparkth.io/healthz"
