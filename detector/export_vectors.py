"""Generate parity vectors from synthetic testdata only, never detector/data."""
import json
from pathlib import Path
import random

from detect import Detector
from keymap import is_thai, runs, to_english, to_thai
from ngram import NgramLM
from export_model import write_model

ROOT = Path(__file__).resolve().parent.parent
DEST = ROOT / "core/tests/vectors"


def decision(detector, run, prev, complete):
    action, score = detector.decide(run, prev, complete=complete)
    # serve.py only transmits candidate/target for convert, but the Rust API
    # always populates them. Apply precisely its direction rule for both.
    candidate, target = (to_english(run), "en") if any(map(is_thai, run)) else (to_thai(run), "th")
    return dict(run=run, prev=prev, complete=complete, convert=action == "convert",
                score=round(score, 6), candidate=candidate, target=target)


def dump(path, rows):
    count = 0
    with path.open("w", encoding="utf-8") as out:
        for row in rows:
            out.write(json.dumps(row, ensure_ascii=False, separators=(",", ":")) + "\n")
            count += 1
    print(f"{path.relative_to(ROOT)}: {count:,} vectors")


def decide_vectors(detector, lines):
    for line in lines:
        for run, lang, prev in runs(line):
            wrong = to_english(run) if lang == "th" else to_thai(run)
            for text in (run, wrong):
                for end in range(1, len(text) + 1):
                    yield decision(detector, text[:end], prev, False)
                yield decision(detector, text, prev, True)
    # Additional neutral/Unicode/boundary cases, with every context and mode.
    for text in ("", "[]", "123", "๑๒๓", "²", "½", "🙂", "🦄ก", "123.45", "$ 1,234.50",
                 "12:30", "50%", "2026-11-10", "12 บาท", "้", "ก้่", "HELLO", "AΣ", "ก  "):
        for prev in (None, "th", "en"):
            for complete in (False, True):
                yield decision(detector, text, prev, complete)


def logprob_strings(lines):
    values = {"", "\x02", "\x03", "🙂", "🦄", "𐍈", "\0", "hello🙂world", "ไทย🙂", "1234567890",
              "[];',./\\=-", "HELLO", " ", "\t", "a" * 40, "ก" * 40}
    for line in lines:
        for word in line.split():
            for text in (word, to_thai(word), to_english(word)):
                values.add(text[:40])
                values.update(text[:i] for i in range(1, min(len(text), 40) + 1))
    rng = random.Random(104729)
    alphabet = "abcxyzABC019.,;[]กขคา่้ิๆๅ๑๒🙂🦄𐍈\x02\x03\0 "
    # Add random mixtures even when the corpus already supplies 2,000 strings.
    # These exercise unseen symbols in both the predicted symbol and context.
    random_values = set()
    while len(random_values) < 2500:
        random_values.add("".join(rng.choice(alphabet) for _ in range(rng.randrange(1, 41))))
    values.update(random_values)
    return sorted(values)


def tiny_fixture():
    model = NgramLM(order=3)
    for text in ("ab", "aba", "ac", "กข", "กก", "hello", "ไทย"):
        model.add(text)
    model.finalize()
    fixture = ROOT / "core/tests/fixtures"
    fixture.mkdir(parents=True, exist_ok=True)
    write_model(model, fixture / "tiny.kfm")
    cases = []
    for text in ("", "a", "ab", "aba", "ax", "🙂", "a🙂b", "กข", "\x02", "hello", "x" * 40):
        for complete in (False, True):
            cases.append(dict(run=text, complete=complete, score=model.logprob(text, complete)))
    (fixture / "tiny.json").write_text(json.dumps(cases, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    # Exercise every configurable decision branch using portable, synthetic
    # models, so CI still checks decide when the private generic models are absent.
    detector = Detector({}, models_dir=fixture)
    detector.lms = {"th": model, "en": model}
    detector.words = set()  # Rust intentionally does not load a system dictionary.
    defaults = detector.config.copy()
    variants = ({}, {"orthography_auto": False}, {"neutral_rule": True},
                {"threshold": -2, "final_threshold": 3, "ctx_delta": 2},
                {"min_len": 8, "min_len_final": 1, "threshold": -100, "final_threshold": -100},
                {"max_check_len": 2, "threshold": -100, "final_threshold": -100},
                {"orthography_auto": False, "orthography_convert": 9, "orthography_keep": 9},
                {"neutral_rule": True, "dictionary_bonus": 100},
                {"min_len": 0, "threshold": 0, "final_threshold": 0, "ctx_delta": 0})
    decisions = []
    for override in variants:
        detector.config = {**defaults, **override}
        for text in ("", "a", "ab", "hello", "HELLO", "กข", "้", "ก้่", "a้", "[]", "123",
                     "²", "๑๒", "🙂", "abc🙂", "ก  ", "กabc", "a" * 40):
            for prev in (None, "th", "en"):
                for complete in (False, True):
                    decisions.append(dict(config=override, **decision(detector, text, prev, complete)))
    (fixture / "decide.json").write_text(json.dumps(decisions, ensure_ascii=False, separators=(",", ":")) + "\n", encoding="utf-8")


def main():
    DEST.mkdir(parents=True, exist_ok=True)
    config = json.loads((ROOT / "detector/config.json").read_text())
    detector = Detector(config, ROOT / "detector/models")
    if set(detector.lms) != {"th", "en"}:
        raise SystemExit("Both detector/models/{th,en}.json.gz are required")
    lines = (ROOT / "detector/testdata/chat_th_en.txt").read_text().splitlines()
    dump(DEST / "decide.jsonl", decide_vectors(detector, lines))
    texts = logprob_strings(lines)
    dump(DEST / "logprob.jsonl", (
        dict(lang=lang, run=text, complete=complete, score=model.logprob(text, complete))
        for lang, model in detector.lms.items() for text in texts for complete in (False, True)))
    tiny_fixture()


if __name__ == "__main__":
    main()
