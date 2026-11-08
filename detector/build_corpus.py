"""Build deterministic local Thai/English training corpus from macOS localization."""
import hashlib
import plistlib
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SEARCH = [Path("/System/Library/Frameworks"), Path("/System/Library/PrivateFrameworks"),
          Path("/System/Applications"), Path("/System/Library/CoreServices")]
MAX_FILES = 30000
SPEC = re.compile(r"%(?:\d+\$)?[-+#0 ]*(?:\d+|\*)?(?:\.\d+)?(?:hh|h|ll|l|q|z|t|j|L)?[@diuoxXfFeEgGaAcCsSpn%]")


def leaves(value):
    if isinstance(value, str):
        yield value
    elif isinstance(value, dict):
        for v in value.values():
            yield from leaves(v)
    elif isinstance(value, (list, tuple)):
        for v in value:
            yield from leaves(v)


def clean(s):
    s = SPEC.sub(" ", s)
    s = re.sub(r"\$\{[^}]*\}|\{[^}]*\}", " ", s)
    s = re.sub(r"<[^>]*>", " ", s)
    s = s.replace(r"\n", " ").replace(r"\t", " ")
    s = re.sub(r"\s+", " ", s).strip()
    return s if re.search(r"[A-Za-z\u0e01-\u0e5b]", s) else ""


def main():
    files = sorted(p for base in SEARCH if base.exists() for p in base.rglob("*.loctable"))
    capped = len(files) > MAX_FILES
    files = files[:MAX_FILES]
    collected = {"th": {"train": set(), "dev": set(), "test": set()}, "en": {"train": set(), "dev": set(), "test": set()}}
    for path in files:
        try:
            data = plistlib.loads(path.read_bytes())
            bucket = int(hashlib.sha1(str(path).encode()).hexdigest(), 16) % 10
            split = "train" if bucket <= 6 else ("dev" if bucket == 7 else "test")
            for locale in ("th", "en"):
                for raw in leaves(data.get(locale, {})):
                    value = clean(raw)
                    if value:
                        collected[locale][split].add(value)
        except Exception:
            continue
    out = ROOT / "data"
    out.mkdir(parents=True, exist_ok=True)
    for locale in ("th", "en"):
        for split in ("train", "dev", "test"):
            rows = sorted(collected[locale][split])
            (out / f"{locale}_{split}.txt").write_text("\n".join(rows) + ("\n" if rows else ""))
            print(f"{locale}_{split}: {len(rows)}")
    print(f"files scanned: {len(files)}" + (f" (capped at {MAX_FILES} of {len(files)})" if capped else ""))


if __name__ == "__main__":
    main()
