"""Keyboard mapping and run segmentation shared by the detector."""
import json
import re
from pathlib import Path

_PAIRS = json.loads((Path(__file__).resolve().parent.parent / "pairs.json").read_text())
ABC_TO_THAI = dict(_PAIRS)
THAI_TO_ABC = {v: k for k, v in _PAIRS}


def is_thai(c):
    return len(c) == 1 and "\u0e01" <= c <= "\u0e5b"


def is_latin(c):
    return len(c) == 1 and ("A" <= c <= "Z" or "a" <= c <= "z")


def to_thai(s):
    return "".join(ABC_TO_THAI.get(c, c) for c in s)


def to_english(s):
    return "".join(THAI_TO_ABC.get(c, c) for c in s)


def runs(line):
    """Return (text, lang, prev_lang) tuples for one line."""
    result, previous = [], None
    for word in line.split():
        chunks, start, last_thai = [], 0, is_thai(word[0])
        for i, char in enumerate(word[1:], 1):
            now = is_thai(char)
            if now != last_thai:
                chunks.append(word[start:i])
                start, last_thai = i, now
        chunks.append(word[start:])
        for chunk in chunks:
            lang = "th" if any(is_thai(c) for c in chunk) else ("en" if any(is_latin(c) for c in chunk) else "neutral")
            result.append((chunk, lang, previous))
            if lang != "neutral":
                previous = lang
    return result
