"""
NOTICE: Training data sources and licenses for KeyFlip open generic model
=========================================================================
- Wisesight Sentiment Corpus: CC0-1.0 Universal Public Domain Dedication
- Common Voice Thai sentences: CC0-1.0 (via PyThaiNLP/th-cv-sentences)
- /usr/share/dict/words: Public Domain
- Hand-written seed lists (detector/seed/casual_th.txt,
  detector/seed/loanwords_th.txt, detector/seed/work_en.txt): MIT License (this repository)
"""
import argparse
import gzip
import json
import re
import sys
from pathlib import Path

# Ensure detector package imports work regardless of working directory
SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from keymap import runs
from ngram import NgramLM

DICT_PATH = Path("/usr/share/dict/words")

DEFAULT_SOURCES = [
    "both:2:data-open/raw/wisesight-pos.txt",
    "both:2:data-open/raw/wisesight-neu.txt",
    "both:2:data-open/raw/wisesight-neg.txt",
    "both:2:data-open/raw/wisesight-q.txt",
    "th:1:data-open/raw/cv-th-approved.txt",
    "th:6:detector/seed/casual_th.txt",
    "th:6:detector/seed/loanwords_th.txt",
    "en:6:detector/seed/work_en.txt",
]


def clean_line(line: str) -> str:
    """Drop URL, @mention, and #hashtag tokens from the line."""
    tokens = []
    for token in line.split():
        stripped = token.strip("()[]{}<>\"'“”‘’")
        if stripped.startswith("@") or stripped.startswith("#"):
            continue
        if re.search(r"(?:https?://|www\.)\S+", token, re.IGNORECASE):
            continue
        tokens.append(token)
    return " ".join(tokens)


def resolve_path(p_str: str) -> Path:
    p = Path(p_str)
    if p.is_absolute():
        return p
    if p.exists():
        return p
    repo_rel = REPO_ROOT / p
    if repo_rel.exists():
        return repo_rel
    return p


def main():
    parser = argparse.ArgumentParser(description="Train KeyFlip open generic language models.")
    parser.add_argument(
        "--out",
        type=Path,
        default=Path("models/open-json"),
        help="Output directory for th.json.gz and en.json.gz (default: models/open-json)",
    )
    parser.add_argument(
        "--source",
        action="append",
        dest="sources",
        default=[],
        help="Source in LANG:WEIGHT:PATH format (can be specified multiple times)",
    )
    parser.add_argument(
        "--dict-weight",
        type=int,
        default=1,
        help="Weight for /usr/share/dict/words in English LM (0 to disable, default: 1)",
    )
    args = parser.parse_args()

    raw_sources = args.sources if args.sources else DEFAULT_SOURCES
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)

    thai = NgramLM(5)
    english = NgramLM(5)

    for spec in raw_sources:
        parts = spec.split(":", 2)
        if len(parts) != 3:
            print(f"Skipping invalid source spec: {spec}", file=sys.stderr)
            continue
        lang_spec, weight_str, path_str = parts
        lang_spec = lang_spec.lower().strip()
        try:
            weight = int(weight_str)
        except ValueError:
            print(f"Skipping invalid weight in source spec: {spec}", file=sys.stderr)
            continue

        resolved_path = resolve_path(path_str)
        line_count = 0
        th_runs = 0
        en_runs = 0

        if resolved_path.exists():
            text_data = resolved_path.read_text(encoding="utf-8", errors="ignore")
            lines = text_data.splitlines()
            line_count = len(lines)
            for line in lines:
                cleaned = clean_line(line)
                if not cleaned:
                    continue
                for text, run_lang, _ in runs(cleaned):
                    if len(text) > 30:
                        continue
                    if run_lang == "th" and lang_spec in ("th", "both"):
                        thai.add(text, weight)
                        th_runs += 1
                    elif run_lang == "en" and lang_spec in ("en", "both"):
                        english.add(text.lower(), weight)
                        en_runs += 1
        else:
            print(f"Note: source file not found: {resolved_path}", file=sys.stderr)

        print(f"{path_str} ({lang_spec}, weight={weight}): {line_count} lines, {th_runs} th runs, {en_runs} en runs")

    if args.dict_weight > 0 and DICT_PATH.exists():
        dict_words = 0
        for value in DICT_PATH.read_text(encoding="utf-8", errors="ignore").splitlines():
            value = value.strip().lower()
            if value and value.isalpha():
                english.add(value, args.dict_weight)
                dict_words += 1
        print(f"{DICT_PATH}: {dict_words} words (weight={args.dict_weight})")

    thai.finalize()
    english.finalize()

    for name, model in (("th", thai), ("en", english)):
        obj = {
            "order": model.order,
            "vocab": sorted(model.vocab),
            "counts": [{"".join(k): v for k, v in c.items()} for c in model.counts[1:]],
        }
        out_file = out_dir / f"{name}.json.gz"
        with gzip.open(out_file, "wt", encoding="utf8") as f:
            json.dump(obj, f, ensure_ascii=False, separators=(",", ":"))
        print(
            f"{name}: {len(model.vocab)} symbols, {sum(map(len, model.counts))} ngrams, {out_file.stat().st_size} bytes"
        )


if __name__ == "__main__":
    main()
