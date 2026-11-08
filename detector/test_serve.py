import subprocess
import sys
import time
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parent


class ServeTests(unittest.TestCase):
    def setUp(self):
        self.proc = subprocess.Popen(
            [sys.executable, "-I", str(ROOT / "serve.py")],
            cwd=ROOT, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.PIPE, text=True, encoding="utf-8", bufsize=1,
        )
        self.assertEqual(self.proc.stdout.readline().rstrip("\r\n"), "ready")

    def tearDown(self):
        if self.proc.poll() is None:
            self.proc.stdin.write("Q\n")
            self.proc.stdin.flush()
            self.proc.wait(timeout=10)
        self.proc.stdin.close()
        self.proc.stdout.close()
        self.proc.stderr.close()

    def ask(self, request):
        self.proc.stdin.write(request + "\n")
        self.proc.stdin.flush()
        return self.proc.stdout.readline().rstrip("\r\n")

    def test_protocol_and_latency(self):
        self.assertEqual(self.ask("P"), "pong")

        response = self.ask("D\tw,j\tth\t1").split("\t")
        self.assertEqual(response[0], "convert")
        self.assertEqual(response[2:], ["ไม่", "th"])
        response = self.ask("D\t้ำสสน\tth\t0").split("\t")
        self.assertEqual(response[0], "convert")
        self.assertEqual(response[2:], ["hello", "en"])
        response = self.ask("D\tl;ylfu\t-\t1").split("\t")
        self.assertEqual(response[0], "convert")
        self.assertEqual(response[2], "สวัสดี")

        for request in (
            "D\tสวัสดี\tth\t1", "D\thello\t-\t1",
            "D\tmeeting\tth\t1", "D\tประชุม\ten\t1",
        ):
            self.assertTrue(self.ask(request).startswith("keep\t"), request)
        self.assertEqual(self.ask("C\tvpkdwxdbo-hk;\tth"), "ok\tอยากไปกินข้าว")
        self.assertEqual(self.ask("C\t้ำสสน\ten"), "ok\thello")

        malformed = self.ask("D onlyonefield")
        self.assertTrue(malformed.startswith("err\t"))
        self.assertEqual(self.ask("P"), "pong")

        durations = []
        for _ in range(500):
            start = time.perf_counter()
            response = self.ask("D\tw,j\tth\t1")
            durations.append((time.perf_counter() - start) * 1000)
            self.assertTrue(response.startswith("convert\t"))
        p99 = sorted(durations)[int(0.99 * len(durations)) - 1]
        print(f"serve.py D request p99: {p99:.3f} ms (500 requests)")
        self.assertLess(p99, 5.0)

    def test_quit_exits_zero(self):
        self.proc.stdin.write("Q\n")
        self.proc.stdin.flush()
        self.assertEqual(self.proc.wait(timeout=10), 0)


if __name__ == "__main__":
    unittest.main()
