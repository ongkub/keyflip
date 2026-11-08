#!/bin/sh
set -eu
HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
CHATS=$(cd "$1" && pwd)
OUT=$2
mkdir -p "$OUT"
python3 "$HERE/import_chats.py" --chats "$CHATS" --out "$OUT"
python3 "$HERE/train.py" --personal "$OUT/me_train.txt" --models-out "$OUT/models"
GENERIC="$HERE/models"
CONFIG="$HERE/config.json"
CHAT="$HERE/testdata/chat_th_en.txt"
TEST="$HERE/data/th_test.txt"
if [ -f "$HERE/data/en_test.txt" ]; then
  # Combine generic test samples in a temporary file under the requested output.
  cat "$TEST" "$HERE/data/en_test.txt" | head -n 3000 > "$OUT/generic_test.txt"
else
  head -n 3000 "$TEST" > "$OUT/generic_test.txt"
fi
python3 "$HERE/evaluate.py" --no-tune --config "$CONFIG" --max-check-len none --models-dir "$GENERIC" \
  --set "me=$OUT/me_test.txt" --set "chat=$CHAT" --set "test=$OUT/generic_test.txt" \
  --report "$OUT/report_generic.md" --max-lines 3000
python3 "$HERE/evaluate.py" --no-tune --config "$CONFIG" --max-check-len none --models-dir "$OUT/models" \
  --set "me=$OUT/me_test.txt" --set "chat=$CHAT" --set "test=$OUT/generic_test.txt" \
  --report "$OUT/report_personal.md" --max-lines 3000
python3 - "$OUT/report_generic.md" "$OUT/report_personal.md" <<'PY'
import re, sys
def rows(path):
    text = open(path, encoding="utf8").read()
    found = {}
    active = None
    for line in text.splitlines():
        if line.startswith("## "):
            active = line[3:]
        if line.startswith("| ") and active and line.split("|")[1].strip() in ("th", "en"):
            cells = [c.strip() for c in line.strip("|").split("|")]
            if len(cells) > 5:
                found[(active, cells[0])] = (cells[2], cells[4])
    return found
generic, personal = rows(sys.argv[1]), rows(sys.argv[2])
print("| set | language | generic FP/1000 | generic recall % | personal FP/1000 | personal recall % |")
print("|---|---|---:|---:|---:|---:|")
for key in sorted(set(generic) | set(personal)):
    g, p = generic.get(key, ("n/a", "n/a")), personal.get(key, ("n/a", "n/a"))
    print(f"| {key[0]} | {key[1]} | {g[0]} | {g[1]} | {p[0]} | {p[1]} |")
PY
