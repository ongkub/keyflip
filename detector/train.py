import gzip
import json
import re
import sys
from pathlib import Path
from keymap import runs
from ngram import NgramLM

ROOT = Path(__file__).resolve().parent
DICT = Path("/usr/share/dict/words")


def load_runs(path, language):
    rows = []
    if path.exists():
        for line in path.read_text(errors="ignore").splitlines():
            rows.extend(text for text, lang, _ in runs(line) if lang == language)
    return rows


def main():
    data, models = ROOT / "data", ROOT / "models"
    models.mkdir(exist_ok=True)
    thai = NgramLM(5)
    for value in load_runs(data / "th_train.txt", "th"):
        thai.add(value)
    thai.finalize()
    english = NgramLM(5)
    en_words = load_runs(data / "en_train.txt", "en") + load_runs(data / "th_train.txt", "en")
    for value in en_words:
        english.add(value.lower())
    if DICT.exists():
        for value in DICT.read_text(errors="ignore").splitlines():
            value = value.strip().lower()
            if value and value.isalpha():
                english.add(value, 1)
    english.finalize()
    for name, model in (("th", thai), ("en", english)):
        obj = {"order": model.order, "vocab": sorted(model.vocab),
               "counts": [{"".join(k): v for k, v in c.items()} for c in model.counts[1:]]}
        # Context statistics are derived on load; gzip keeps models compact.
        with gzip.open(models / f"{name}.json.gz", "wt", encoding="utf8") as f:
            json.dump(obj, f, ensure_ascii=False, separators=(",", ":"))
        print(f"{name}: {len(model.vocab)} symbols, {sum(map(len, model.counts))} ngrams, {(models / f'{name}.json.gz').stat().st_size} bytes")


if __name__ == "__main__":
    main()
