"""Line-oriented helper process for the keyboard layout detector."""
import argparse
import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parent
# Isolated mode omits the script directory from sys.path.
sys.path.insert(0, str(ROOT))

from detect import Detector  # noqa: E402
from keymap import to_english, to_thai  # noqa: E402


def _parser():
    parser = argparse.ArgumentParser()
    parser.add_argument("--models-dir", type=Path, default=ROOT / "models")
    parser.add_argument("--config", type=Path)
    return parser


def _response(line, detector):
    fields = line.split("\t")
    if fields == ["P"]:
        return "pong", False
    if fields == ["Q"]:
        return "", True
    if len(fields) == 4 and fields[0] == "D":
        _, run, prev, complete = fields
        if prev not in ("th", "en", "-") or complete not in ("0", "1"):
            return "err\tbad D fields", False
        action, score = detector.decide(run, None if prev == "-" else prev,
                                        complete=complete == "1")
        if action == "convert":
            if any("\u0e01" <= c <= "\u0e5b" for c in run):
                candidate, target = to_english(run), "en"
            else:
                candidate, target = to_thai(run), "th"
            return f"convert\t{score:.3f}\t{candidate}\t{target}", False
        return f"keep\t{score:.3f}", False
    if len(fields) == 3 and fields[0] == "C":
        _, value, target = fields
        if target == "th":
            converted = to_thai(value)
        elif target == "en":
            converted = to_english(value)
        else:
            return "err\tbad C target", False
        return f"ok\t{converted}", False
    return "err\tmalformed request", False


def main():
    args = _parser().parse_args()
    try:
        config = json.loads(args.config.read_text(encoding="utf-8")) if args.config else None
        detector = Detector(config, args.models_dir)
    except Exception as exc:
        # Startup diagnostics may be sent to stderr; never include request data.
        print(f"startup error: {type(exc).__name__}: {exc}", file=sys.stderr, flush=True)
        return 1

    out = sys.stdout
    out.write("ready\n")
    out.flush()
    while True:
        raw = sys.stdin.buffer.readline()
        if not raw:
            return 0
        line = raw.decode("utf-8", errors="replace").rstrip("\r\n")
        try:
            result, quit_requested = _response(line, detector)
        except Exception:
            # Do not leak the request or exception contents: they may contain keystrokes.
            result, quit_requested = "err\trequest failed", False
        if quit_requested:
            return 0
        out.write(result + "\n")
        out.flush()


if __name__ == "__main__":
    raise SystemExit(main())
