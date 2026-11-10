import base64
import json
import os
import tempfile
import threading
import unittest
from http.server import ThreadingHTTPServer
from urllib.error import HTTPError
from urllib.request import Request, urlopen

from server import feedback_server as app


class FeedbackServerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.db = os.path.join(cls.temp.name, "reports.db")
        app.DB_PATH = cls.db
        app.IP_SALT = "test-salt"
        app.ADMIN_PASSWORD = "test-password"
        app.init_db(cls.db)
        cls.server = ThreadingHTTPServer(("127.0.0.1", 0), app.Handler)
        cls.thread = threading.Thread(target=cls.server.serve_forever, daemon=True)
        cls.thread.start()
        cls.base = f"http://127.0.0.1:{cls.server.server_port}"

    @classmethod
    def tearDownClass(cls):
        cls.server.shutdown()
        cls.server.server_close()
        cls.thread.join()
        cls.temp.cleanup()

    def setUp(self):
        # Every test starts from an empty table so rate limits and admin listings do not depend on test order.
        import sqlite3
        with sqlite3.connect(self.db) as conn:
            conn.execute("DELETE FROM reports")

    def post(self, payload, headers=None, raw=None):
        body = raw if raw is not None else json.dumps(payload, ensure_ascii=False).encode()
        request = Request(self.base + "/v1/reports", data=body, headers={"Content-Type": "application/json", **(headers or {})})
        try:
            response = urlopen(request)
        except HTTPError as exc:
            return exc.code, json.loads(exc.read())
        return response.status, json.loads(response.read())

    def valid(self, **updates):
        data = {"schema": 1, "app": "keyflip", "platform": "macos", "app_version": "0.2.0", "os_version": "macOS 27.2", "model": "personal", "enabled": True,
                "stats_today": {"runs": 120, "conversions": 4, "undos": 1}, "message": "<script>alert(1)</script>", "contact": "", "recent_words": [{"word": "vpkd", "result": "อยาก", "kind": "typing"}], "locale": "th-TH"}
        data.update(updates)
        return data

    def test_valid_report_stored_with_hashed_ip_and_unknown_keys_dropped(self):
        status, response = self.post(self.valid(extra_secret="drop me"))
        self.assertEqual(status, 201)
        self.assertEqual(response["ok"], True)
        import sqlite3
        with sqlite3.connect(self.db) as conn:
            ip_hash, stored = conn.execute("SELECT ip_hash, report_json FROM reports WHERE id=?", (response["id"],)).fetchone()
        self.assertEqual(ip_hash, app.hash_ip("127.0.0.1", "test-salt"))
        self.assertNotEqual(ip_hash, "127.0.0.1")
        self.assertNotIn("extra_secret", json.loads(stored))

    def test_invalid_reports_and_body_limit(self):
        status, _ = self.post(self.valid(message=None))
        self.assertEqual(status, 400)
        status, _ = self.post(self.valid(enabled=1))
        self.assertEqual(status, 400)
        status, _ = self.post({}, raw=b"not json")
        self.assertEqual(status, 400)
        status, _ = self.post({}, raw=b" " * (16 * 1024 + 1))
        self.assertEqual(status, 413)

    def test_rate_limit_sixth_hourly_request(self):
        import sqlite3
        with sqlite3.connect(self.db) as conn:
            conn.execute("DELETE FROM reports")
        for _ in range(5):
            self.assertEqual(self.post(self.valid())[0], 201)
        self.assertEqual(self.post(self.valid())[0], 429)

    def test_forwarded_for_trust_helper(self):
        self.assertEqual(app.client_ip("192.0.2.4", "198.51.100.1, 10.0.0.2"), "192.0.2.4")
        self.assertEqual(app.client_ip("127.0.0.1", "198.51.100.1, 10.0.0.2"), "198.51.100.1")
        docker = app.parse_networks("172.16.0.0/12")
        self.assertEqual(app.client_ip("172.18.0.5", "198.51.100.7", docker), "198.51.100.7")  # Traefik in Docker
        self.assertEqual(app.client_ip("203.0.113.9", "198.51.100.7", docker), "203.0.113.9")  # not a proxy
        self.assertEqual(app.client_ip("not-an-ip", "198.51.100.7", docker), "not-an-ip")

    def test_admin_auth_and_html_escaping(self):
        self.assertEqual(self.post(self.valid())[0], 201)  # message contains <script>
        request = Request(self.base + "/admin")
        with self.assertRaises(HTTPError) as error:
            urlopen(request)
        self.assertEqual(error.exception.code, 401)
        token = base64.b64encode(b"admin:test-password").decode()
        response = urlopen(Request(self.base + "/admin", headers={"Authorization": "Basic " + token}))
        body = response.read().decode()
        self.assertEqual(response.status, 200)
        self.assertIn("&lt;script&gt;", body)
        self.assertNotIn("<script>", body)
        self.assertEqual(response.headers["Cache-Control"], "no-store")

    def test_admin_json_and_unconfigured_password(self):
        token = base64.b64encode(b"admin:test-password").decode()
        response = urlopen(Request(self.base + "/admin/reports.json", headers={"Authorization": "Basic " + token}))
        self.assertIsInstance(json.loads(response.read()), list)
        old = app.ADMIN_PASSWORD
        app.ADMIN_PASSWORD = ""
        try:
            with self.assertRaises(HTTPError) as error:
                urlopen(Request(self.base + "/admin"))
            self.assertEqual(error.exception.code, 503)
        finally:
            app.ADMIN_PASSWORD = old


if __name__ == "__main__":
    unittest.main()
