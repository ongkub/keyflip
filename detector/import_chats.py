"""Privacy-first importer for LINE text exports and copied chat lines."""
import argparse
import hashlib
import re
from collections import Counter
from pathlib import Path

from keymap import is_latin, is_thai

TIME = re.compile(r"^\s*(?:\d{1,2}:\d{2}(?:\s*[AP]M)?|\d{1,2}:\d{2}น\.)\t([^\t]+)\t(.*)$", re.I)
DATE = re.compile(r"^(?:\d{4}[./-]\d{1,2}[./-]\d{1,2}.*|(?:Mon|Tue|Wed|Thu|Fri|Sat|Sun),.*|วัน.{2,},.*)$", re.I)
HEADER = re.compile(r"^(?:\[LINE\]|Chat history|ประวัติการแชท|Saved on:|บันทึกเมื่อ:)", re.I)
URL = re.compile(r"\b(?:https?://|www\.)\S+", re.I)
PLACEHOLDER = re.compile(r"^\[(?:Sticker|Photo|Video|File|Voice message|Album|Location|Contact|Note|สติกเกอร์|รูป|รูปภาพ|วิดีโอ|ไฟล์|ข้อความเสียง|อัลบั้ม|ตำแหน่ง|รายชื่อ|โน้ต)[^\]]*\]$", re.I)


def decode(path):
    raw = path.read_bytes()
    for enc in ("utf-8-sig", "utf-16"):
        try:
            return raw.decode(enc)
        except UnicodeError:
            pass
    return raw.decode("utf-8", errors="replace")


def emoji_only(text):
    return not re.search(r"[\w\u0e00-\u0e7f]", text, re.UNICODE)


def parse_file(path):
    lines = decode(path).splitlines()
    parsed, senders = [], Counter()
    current_sender, current = None, []
    def flush():
        if current:
            value = "\n".join(current).strip()
            if value.startswith('"') and value.endswith('"'):
                value = value[1:-1]
            parsed.append((current_sender, value.strip()))
    matched = False
    for line in lines:
        m = TIME.match(line)
        if m:
            flush(); matched = True
            current_sender, current = m.group(1).strip(), [m.group(2)]
            senders[current_sender] += 1
        elif DATE.match(line) or HEADER.match(line):
            flush(); current_sender, current = None, []
        elif current_sender is not None and line.strip():
            current.append(line.strip())
    flush()
    if not matched:
        return [("", line.strip()) for line in lines if line.strip()], Counter()
    return parsed, senders


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--chats", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--me")
    args = ap.parse_args()
    files = sorted(args.chats.glob("*.txt"))
    messages, sender_files, sender_msgs = [], Counter(), Counter()
    total_lines = 0
    for path in files:
        text_lines = decode(path).splitlines(); total_lines += len(text_lines)
        rows, counts = parse_file(path)
        messages.extend(rows)
        for name, count in counts.items():
            sender_msgs[name] += count
            sender_files[name] += 1
    chosen = args.me
    if chosen is None and sender_msgs:
        chosen = sorted(sender_msgs, key=lambda n: (-sender_files[n], -sender_msgs[n], n))[0]
    print(f"files={len(files)} total_lines={total_lines}")
    print(f"chosen_sender={chosen or '(copy-paste)'} files={sender_files.get(chosen, 0) if chosen else 0}")
    print("top_senders=" + ", ".join(f"{n}:{c}" for n, c in sender_msgs.most_common(5)))
    kept, dropped = [], Counter()
    for sender, msg in messages:
        if sender != (chosen or ""):
            dropped["other_sender"] += 1; continue
        msg = URL.sub("", msg).replace("\r", " ").replace("\n", " ")
        msg = re.sub(r"\s+", " ", msg).strip().strip('"').strip()
        if not msg: dropped["empty"] += 1; continue
        if PLACEHOLDER.fullmatch(msg): dropped["placeholder"] += 1; continue
        if "☎" in msg or re.search(r"call time|ระยะเวลาการโทร", msg, re.I): dropped["call"] += 1; continue
        if re.search(r"unsent a message|ยกเลิกการส่งข้อความ", msg, re.I): dropped["unsent"] += 1; continue
        if re.fullmatch(r"(?:https?://|www\.)\S+", msg, re.I): dropped["url"] += 1; continue
        if emoji_only(msg): dropped["emoji"] += 1; continue
        kept.append(msg)
    kept = list(dict.fromkeys(kept))
    train, test = [], []
    for msg in kept:
        bucket = int(hashlib.sha1(msg.encode("utf-8")).hexdigest(), 16) % 10
        (train if bucket < 7 else test).append(msg)
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "me_train.txt").write_text("\n".join(train) + ("\n" if train else ""), encoding="utf8")
    (args.out / "me_test.txt").write_text("\n".join(test) + ("\n" if test else ""), encoding="utf8")
    th = sum(any(is_thai(c) for c in m) for m in kept)
    en = sum(any(is_latin(c) for c in m) for m in kept)
    both = sum(any(is_thai(c) for c in m) and any(is_latin(c) for c in m) for m in kept)
    print(f"user_messages_kept={len(kept)} dropped=" + ", ".join(f"{k}:{v}" for k, v in sorted(dropped.items())))
    print(f"train={len(train)} test={len(test)} thai={th}/{len(kept)} latin={en}/{len(kept)} both={both}/{len(kept)}")


if __name__ == "__main__":
    main()
