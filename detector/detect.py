import json
import re
import string
from pathlib import Path
from keymap import is_latin, is_thai, to_english, to_thai
from ngram import NgramLM

ROOT = Path(__file__).resolve().parent
TRIGGERS = set("ะ ั า ำ ิ ี ึ ื ุ ู ฺ ็ ่ ้ ๊ ๋ ์ ํ ๅ ฯ ๆ".split())
ABOVE_BELOW = set("ั ิ ี ึ ื ุ ู ฺ ็ ่ ้ ๊ ๋ ์ ํ")
TONE = set("่ ้ ๊ ๋")


def thai_orthography_ok(s):
    if not s:
        return True
    if s[0] in TRIGGERS:
        return False
    prev_tone = False
    for i, c in enumerate(s):
        if c in TONE and prev_tone:
            return False
        if c in ABOVE_BELOW and i and not is_thai(s[i - 1]) and s[i - 1] not in ABOVE_BELOW | TONE:
            return False
        prev_tone = c in TONE
    return True


class Detector:
    def __init__(self, config=None):
        self.config = {"threshold": 0.65, "ctx_delta": 0.15, "min_len": 2,
                       "orthography_convert": 1.8, "orthography_keep": 1.8,
                       "dictionary_bonus": 0.0}
        if config is None:
            try:
                self.config.update(json.loads((ROOT / "config.json").read_text()))
            except OSError:
                pass
        else:
            self.config.update(config)
        self.lms = {}
        for lang in ("th", "en"):
            path = ROOT / "models" / f"{lang}.json.gz"
            if path.exists():
                self.lms[lang] = NgramLM.load(path)
        self.words = set()
        try:
            self.words = {w.strip().lower() for w in Path("/usr/share/dict/words").read_text(errors="ignore").splitlines()}
        except OSError:
            pass

    def decide(self, run_so_far, prev_lang):
        s = run_so_far
        source = "th" if any(is_thai(c) for c in s) else ("en" if any(is_latin(c) for c in s) else None)
        if source is None or source not in self.lms or ("th" not in self.lms or "en" not in self.lms):
            return "keep", 0.0
        target = "en" if source == "th" else "th"
        candidate = to_english(s) if source == "th" else to_thai(s)
        score = (self.lms[target].logprob(candidate.lower() if target == "en" else candidate) -
                 self.lms[source].logprob(s.lower() if source == "en" else s)) / max(1, len(s))
        threshold = self.config["threshold"]
        if prev_lang == target:
            threshold -= self.config["ctx_delta"]
        elif prev_lang == source:
            threshold += self.config["ctx_delta"]
        thai_side = s if source == "th" else candidate
        if not thai_orthography_ok(thai_side):
            if source == "th":
                return "convert", score + self.config["orthography_convert"]
            score -= self.config["orthography_keep"]
        if source == "en" and s.lower().rstrip(string.punctuation) in self.words:
            score -= self.config["dictionary_bonus"]
        if len(s) < self.config["min_len"]:
            return "keep", score
        return ("convert" if score > threshold else "keep"), score


def main():
    print("Use Detector(config).decide(prefix, prev_lang)")


if __name__ == "__main__":
    main()
