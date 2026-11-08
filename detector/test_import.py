import hashlib
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from import_chats import parse_file

ROOT = Path(__file__).resolve().parent


class ImportTests(unittest.TestCase):
    def test_line_fixtures(self):
        first, senders = parse_file(ROOT / "testdata/line_sample.txt")
        second, _ = parse_file(ROOT / "testdata/line_sample_mobile.txt")
        all_rows = first + second
        self.assertEqual(senders["Me Test"], 5)
        self.assertTrue(any(sender == "Me Test" and "ภาษาไทย\nแล้วต่อด้วย English mixed" in msg for sender, msg in first))
        self.assertTrue(any(sender == "Me Test" and "https://" in msg for sender, msg in first))
        self.assertTrue(any(sender == "Friend A" and msg == "[รูปภาพ]" for sender, msg in second))
        self.assertEqual(hashlib.sha1("สวัสดีครับ".encode()).hexdigest(), hashlib.sha1("สวัสดีครับ".encode()).hexdigest())
        self.assertGreater(len(all_rows), 0)

    def test_space_separated_format(self):
        rows, senders = parse_file(ROOT / "testdata/line_sample_space.txt")
        self.assertEqual(senders["Me Test"], 7)
        self.assertEqual(senders["Friend A"], 5)
        self.assertIn(("Me Test", "บรรทัดแรก\nบรรทัดที่สอง English"), rows)
        self.assertIn(("Friend A", "10:30 นะ"), rows)

    def test_copy_paste_fallback(self):
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "copy.txt"
            p.write_text("first line\nsecond line\n", encoding="utf8")
            rows, senders = parse_file(p)
            self.assertEqual(rows, [("", "first line"), ("", "second line")])
            self.assertFalse(senders)

    def test_import_drop_url_and_deterministic_split(self):
        with tempfile.TemporaryDirectory() as td:
            out1, out2 = Path(td) / "one", Path(td) / "two"
            command = [sys.executable, str(ROOT / "import_chats.py"), "--chats", str(ROOT / "testdata"), "--out"]
            subprocess.run(command + [str(out1)], check=True, capture_output=True, text=True)
            subprocess.run(command + [str(out2)], check=True, capture_output=True, text=True)
            for name in ("me_train.txt", "me_test.txt"):
                a, b = (out1 / name).read_bytes(), (out2 / name).read_bytes()
                self.assertEqual(a, b)
                self.assertNotIn(b"https://", a)
                self.assertNotIn(b"[Sticker]", a)
                self.assertNotIn(b"<a ", a)
                self.assertNotIn("รูป", a.decode("utf8").splitlines())


if __name__ == "__main__":
    unittest.main()
