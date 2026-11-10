#!/usr/bin/env python3
"""Small standard-library-only receiver for KeyFlip problem reports."""

from __future__ import annotations

import base64
import hashlib
import hmac
import html
import ipaddress
import json
import os
import secrets
import socket
import sqlite3
import sys
import threading
import time
from datetime import datetime, timedelta, timezone
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlsplit

MAX_BODY = 16 * 1024
MAX_REPORTS_PER_DAY = int(os.environ.get("KEYFLIP_MAX_REPORTS_PER_DAY", "1000"))
DB_PATH = os.environ.get("KEYFLIP_DB", "/var/lib/keyflip-feedback/reports.db")
BIND = os.environ.get("KEYFLIP_BIND", "127.0.0.1:8787")
ADMIN_PASSWORD = os.environ.get("KEYFLIP_ADMIN_PASSWORD", "")
IP_SALT = os.environ.get("KEYFLIP_IP_SALT", "")


def now_iso() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def parse_networks(text: str) -> list:
    nets = []
    for part in text.split(","):
        part = part.strip()
        if part:
            nets.append(ipaddress.ip_network(part, strict=False))
    return nets


# Proxies whose X-Forwarded-For we believe. Default: loopback (Caddy on the same host). In Docker behind
# Traefik, KEYFLIP_TRUSTED_PROXY_HOSTS names the Traefik container; it is resolved through Docker's DNS
# (cached for 60 s) so only that container is trusted, even if its IP changes after a restart.
TRUSTED_PROXIES = parse_networks(os.environ.get("KEYFLIP_TRUSTED_PROXIES", "127.0.0.1/32,::1/128"))
TRUSTED_PROXY_HOSTS = [h.strip() for h in os.environ.get("KEYFLIP_TRUSTED_PROXY_HOSTS", "").split(",") if h.strip()]
_proxy_cache = {"at": 0.0, "nets": []}
_proxy_lock = threading.Lock()


def trusted_networks() -> list:
    """Static networks plus the current addresses of the named proxy containers."""
    if not TRUSTED_PROXY_HOSTS:
        return TRUSTED_PROXIES
    with _proxy_lock:
        if time.monotonic() - _proxy_cache["at"] > 60:
            nets = []
            for host in TRUSTED_PROXY_HOSTS:
                try:
                    for info in socket.getaddrinfo(host, None):
                        nets.append(ipaddress.ip_network(info[4][0]))
                except OSError:
                    pass
            _proxy_cache.update(at=time.monotonic(), nets=nets)
        return TRUSTED_PROXIES + _proxy_cache["nets"]


def client_ip(peer_ip: str, forwarded_for: str | None, trusted=None) -> str:
    """When the TCP peer is a trusted proxy, use the hop it appended: the LAST X-Forwarded-For entry,
    and only if it is a valid IP address. Anything else falls back to the peer address."""
    nets = trusted_networks() if trusted is None else trusted
    try:
        peer = ipaddress.ip_address(peer_ip)
    except ValueError:
        return peer_ip
    if forwarded_for and any(peer in net for net in nets):
        last = forwarded_for.rsplit(",", 1)[-1].strip()
        try:
            return str(ipaddress.ip_address(last))
        except ValueError:
            return peer_ip
    return peer_ip


def hash_ip(ip: str, salt: str) -> str:
    return hashlib.sha256((salt + ip).encode("utf-8")).hexdigest()


def clean_report(data: object) -> dict:
    if not isinstance(data, dict):
        raise ValueError("JSON body must be an object")
    if type(data.get("schema")) is not int or data["schema"] != 1:
        raise ValueError("schema must be 1")
    if data.get("app") != "keyflip":
        raise ValueError("app must be keyflip")
    platform = data.get("platform")
    if platform not in ("macos", "windows"):
        raise ValueError("invalid platform")
    model = data.get("model")
    if model not in ("personal", "open", "generic"):
        raise ValueError("invalid model")
    if type(data.get("enabled")) is not bool:
        raise ValueError("enabled must be a boolean")

    def string(key: str, limit: int, required: bool = True) -> str | None:
        value = data.get(key)
        if value is None and not required:
            return None
        if not isinstance(value, str):
            raise ValueError(f"{key} must be a string")
        if len(value) > limit or (required and not value):
            raise ValueError(f"invalid {key} length")
        return value

    message = string("message", 2000)
    if len(message) < 1:
        raise ValueError("message is required")
    app_version = string("app_version", 100)
    os_version = string("os_version", 200)
    locale = string("locale", 35)

    stats = data.get("stats_today")
    if not isinstance(stats, dict):
        raise ValueError("stats_today must be an object")
    cleaned_stats = {}
    for key in ("runs", "conversions", "undos"):
        value = stats.get(key)
        if type(value) is not int or value < 0:
            raise ValueError(f"stats_today.{key} must be a non-negative integer")
        cleaned_stats[key] = value

    report = {
        "schema": 1,
        "app": "keyflip",
        "platform": platform,
        "app_version": app_version,
        "os_version": os_version,
        "model": model,
        "enabled": data["enabled"],
        "stats_today": cleaned_stats,
        "message": message,
    }
    contact = string("contact", 200, required=False)
    if contact is not None:
        report["contact"] = contact
    words = data.get("recent_words")
    if words is not None:
        if not isinstance(words, list) or len(words) > 10:
            raise ValueError("recent_words must be an array of at most 10 items")
        cleaned_words = []
        for item in words:
            if not isinstance(item, dict):
                raise ValueError("recent_words items must be objects")
            word, result, kind = item.get("word"), item.get("result"), item.get("kind")
            if not isinstance(word, str) or len(word) > 40:
                raise ValueError("recent_words.word must be a string of at most 40 chars")
            if not isinstance(result, str) or len(result) > 40:
                raise ValueError("recent_words.result must be a string of at most 40 chars")
            if kind != "typing":
                raise ValueError("recent_words.kind must be typing")
            cleaned_words.append({"word": word, "result": result, "kind": "typing"})
        report["recent_words"] = cleaned_words
    report["locale"] = locale
    return report


def init_db(path: str) -> None:
    parent = os.path.dirname(os.path.abspath(path))
    os.makedirs(parent, exist_ok=True)
    with sqlite3.connect(path) as conn:
        conn.execute("PRAGMA journal_mode=WAL")
        conn.execute("""CREATE TABLE IF NOT EXISTS reports (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            received_at TEXT NOT NULL,
            ip_hash TEXT NOT NULL,
            report_json TEXT NOT NULL
        )""")
        conn.execute("CREATE INDEX IF NOT EXISTS reports_ip_time ON reports(ip_hash, received_at)")


class Handler(BaseHTTPRequestHandler):
    server_version = "KeyFlipFeedback/1"
    timeout = 10  # seconds per socket read, so slow or stalled clients cannot hold worker threads

    def log_message(self, fmt: str, *args) -> None:
        # Keep the standard one-line request log; never log request bodies.
        sys.stderr.write("%s - %s\n" % (self.address_string(), fmt % args))

    def send_json(self, status: int, payload: object, headers: dict | None = None) -> None:
        body = json.dumps(payload, ensure_ascii=False, separators=(",", ":")).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        for key, value in (headers or {}).items():
            self.send_header(key, value)
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self) -> None:
        path = urlsplit(self.path).path
        if path == "/healthz":
            body = b"ok"
            self.send_response(200)
            self.send_header("Content-Type", "text/plain; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
        elif path in ("/admin", "/admin/reports.json"):
            self.handle_admin(path)
        else:
            self.send_json(404, {"ok": False, "error": "not found"})

    def handle_admin(self, path: str) -> None:
        if not ADMIN_PASSWORD:
            self.send_json(503, {"ok": False, "error": "admin is not configured"}, {"Cache-Control": "no-store"})
            return
        auth = self.headers.get("Authorization", "")
        supplied = ""
        try:
            scheme, token = auth.split(" ", 1)
            if scheme.lower() == "basic":
                decoded = base64.b64decode(token, validate=True).decode("utf-8")
                username, supplied = decoded.split(":", 1)
            else:
                username = ""
        except (ValueError, UnicodeError):
            username, supplied = "", ""
        if username != "admin" or not hmac.compare_digest(supplied, ADMIN_PASSWORD):
            self.send_response(401)
            self.send_header("WWW-Authenticate", 'Basic realm="KeyFlip reports"')
            self.send_header("Cache-Control", "no-store")
            self.send_header("Content-Length", "0")
            self.end_headers()
            return
        with sqlite3.connect(DB_PATH) as conn:
            conn.row_factory = sqlite3.Row
            rows = conn.execute("SELECT id, received_at, report_json FROM reports ORDER BY id DESC LIMIT 200").fetchall()
        records = []
        for row in rows:
            report = json.loads(row["report_json"])
            records.append({"id": row["id"], "time": row["received_at"], **report})
        if path.endswith(".json"):
            self.send_json(200, records, {"Cache-Control": "no-store"})
            return
        labels = ["เวลา", "แพลตฟอร์ม", "เวอร์ชัน", "ระบบปฏิบัติการ", "โมเดล", "ข้อความ", "ช่องทางติดต่อ", "คำล่าสุด"]
        parts = ["<!doctype html><html lang=\"th\"><meta charset=\"utf-8\"><title>รายงาน KeyFlip</title><body><h1>รายงานปัญหา KeyFlip</h1><table><thead><tr>"]
        parts.extend(f"<th>{html.escape(label)}</th>" for label in labels)
        parts.append("</tr></thead><tbody>")
        for report in records:
            words = report.get("recent_words", [])
            words_text = ", ".join(f"{w['word']} → {w['result']}" for w in words)
            cells = [report.get("time", ""), report.get("platform", ""), report.get("app_version", ""), report.get("os_version", ""), report.get("model", ""), report.get("message", ""), report.get("contact", ""), words_text]
            parts.append("<tr>" + "".join(f"<td>{html.escape(str(cell))}</td>" for cell in cells) + "</tr>")
        parts.append("</tbody></table></body></html>")
        body = "".join(parts).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(body)

    def do_POST(self) -> None:
        if urlsplit(self.path).path != "/v1/reports":
            self.send_json(404, {"ok": False, "error": "not found"})
            return
        length = self.headers.get("Content-Length")
        try:
            size = int(length) if length is not None else -1
        except ValueError:
            size = -1
        if size < 0:
            self.send_json(400, {"ok": False, "error": "valid Content-Length required"})
            return
        if size > MAX_BODY:
            self.send_json(413, {"ok": False, "error": "request body too large"})
            return
        raw = self.rfile.read(size)
        if len(raw) != size:
            self.send_json(400, {"ok": False, "error": "incomplete request body"})
            return
        try:
            data = json.loads(raw.decode("utf-8"))
            report = clean_report(data)
        except (UnicodeDecodeError, json.JSONDecodeError, ValueError) as exc:
            self.send_json(400, {"ok": False, "error": str(exc)})
            return
        peer = self.client_address[0]
        ip = client_ip(peer, self.headers.get("X-Forwarded-For"))
        ip_digest = hash_ip(ip, IP_SALT)
        received = now_iso()
        with sqlite3.connect(DB_PATH, timeout=10) as conn:
            hour_ago = (datetime.now(timezone.utc) - timedelta(hours=1)).isoformat(timespec="seconds")
            day_ago = (datetime.now(timezone.utc) - timedelta(days=1)).isoformat(timespec="seconds")
            hour_count = conn.execute("SELECT COUNT(*) FROM reports WHERE ip_hash=? AND received_at>=?", (ip_digest, hour_ago)).fetchone()[0]
            day_count = conn.execute("SELECT COUNT(*) FROM reports WHERE ip_hash=? AND received_at>=?", (ip_digest, day_ago)).fetchone()[0]
            if hour_count >= 5 or day_count >= 20:
                self.send_json(429, {"ok": False, "error": "rate limit exceeded"})
                return
            # Global cap: many addresses together cannot fill the disk.
            total_day = conn.execute("SELECT COUNT(*) FROM reports WHERE received_at>=?", (day_ago,)).fetchone()[0]
            if total_day >= MAX_REPORTS_PER_DAY:
                self.send_json(429, {"ok": False, "error": "daily capacity reached"})
                return
            cursor = conn.execute("INSERT INTO reports(received_at, ip_hash, report_json) VALUES(?,?,?)", (received, ip_digest, json.dumps(report, ensure_ascii=False, separators=(",", ":"))))  # schema fields only, no user agent
            report_id = cursor.lastrowid
        self.send_json(201, {"ok": True, "id": report_id})


def main() -> None:
    if not IP_SALT:
        raise SystemExit("KEYFLIP_IP_SALT is empty; refusing to store unsalted IP hashes")
    init_db(DB_PATH)
    if ":" in BIND and not BIND.startswith("["):
        host, port_text = BIND.rsplit(":", 1)
        port = int(port_text)
    else:
        host, port = "127.0.0.1", 8787
    server = ThreadingHTTPServer((host, port), Handler)
    server.daemon_threads = True
    print(f"KeyFlip feedback listening on {host}:{port}", flush=True)
    server.serve_forever()


if __name__ == "__main__":
    main()
