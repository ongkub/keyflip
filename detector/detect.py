import json
import re
import string
from pathlib import Path
from keymap import is_latin, is_thai, to_english, to_thai
from ngram import NgramLM

ROOT = Path(__file__).resolve().parent
TRIGGERS = set("ะ ั า ำ ิ ี ึ ื ุ ู ฺ ็ ่ ้ ๊ ๋ ์ ํ ๅ".split())
ABOVE_BELOW = set("ั ิ ี ึ ื ุ ู ฺ ็ ่ ้ ๊ ๋ ์ ํ")
TONE = set("่ ้ ๊ ๋")
NEUTRAL_CANDIDATE = set("[];',./\\=-")
NUMBER_LIKE = re.compile(
    r"^(?:(?:\$|€|£|฿)\s*)?"
    r"(?:\d+|\d{1,3}(?:,\d{3})+(?:\.\d+)?|\d+(?:\.\d+)?|"
    r"\d{1,2}:\d{2}|\d+(?:\.\d+)?%|\d{1,4}(?:[-/]\d{1,4}){1,2}|"
    r"\d{2,4}-\d{2,4}-\d{2,4})"
    r"(?:\s*(?:฿|บาท|\$|€|£))?$",
    re.IGNORECASE,
)


def neutral_candidate_ok(s):
    return (bool(s) and any(c in NEUTRAL_CANDIDATE or c.isdigit() for c in s)
            and not NUMBER_LIKE.fullmatch(s.strip())
            and thai_orthography_ok(to_thai(s)))


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
    _MODEL_CACHE = {}

    def __init__(self, config=None, models_dir=None):
        self.config = {"threshold": 0.65, "final_threshold": 0.65,
                       "ctx_delta": 0.15, "min_len": 2,
                       "max_check_len": None, "neutral_rule": False,
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
        models_dir = Path(models_dir) if models_dir is not None else ROOT / "models"
        for lang in ("th", "en"):
            path = models_dir / f"{lang}.json.gz"
            if path.exists():
                key = str(path.resolve())
                if key not in self._MODEL_CACHE:
                    self._MODEL_CACHE[key] = NgramLM.load(path)
                self.lms[lang] = self._MODEL_CACHE[key]
        self.words = set()
        try:
            self.words = {w.strip().lower() for w in Path("/usr/share/dict/words").read_text(errors="ignore").splitlines()}
        except OSError:
            pass

    def decide(self, run_so_far, prev_lang, complete=False):
        s = run_so_far
        has_thai = any(is_thai(c) for c in s)
        has_latin = any(is_latin(c) for c in s)
        source = "th" if has_thai else ("en" if has_latin else None)
        if source is None and self.config.get("neutral_rule"):
            if neutral_candidate_ok(s):
                source = "en"
        if source is None or source not in self.lms or ("th" not in self.lms or "en" not in self.lms):
            return "keep", 0.0
        max_check_len = self.config.get("max_check_len")
        if not complete and max_check_len is not None and len(s) > max_check_len:
            return "keep", 0.0
        target = "en" if source == "th" else "th"
        candidate = to_english(s) if source == "th" else to_thai(s)
        score = (self.lms[target].logprob(candidate.lower() if target == "en" else candidate, complete=complete) -
                 self.lms[source].logprob(s.lower() if source == "en" else s, complete=complete)) / max(1, len(s))
        threshold = self.config["final_threshold"] if complete else self.config["threshold"]
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
