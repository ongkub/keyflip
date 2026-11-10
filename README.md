# KeyFlip

แก้คำที่พิมพ์ผิดภาษาให้อัตโนมัติระหว่างพิมพ์ (แป้นไทยเกษมณี ⇄ อังกฤษ) — พิมพ์ `vpkdwxdbo-hk;` แล้วได้ "อยากไปกินข้าว"
พร้อมสลับแป้นให้เอง · Fixes words typed with the wrong keyboard layout (Thai Kedmanee ⇄ English) while you type.

**Status: early beta, not released yet.** macOS prototype works; Windows is in progress.

## Privacy
- Everything runs on your computer. Nothing you type is sent anywhere.
- The local log keeps only words that KeyFlip converted or that you undid; everything else is counted, never stored.
- "Report a problem" sends only what you see in the report window, and only when you press send.

## How it decides
Two character 5-gram language models (Thai, English) score each word as you type; if the word reads much better in
the other layout, KeyFlip replaces it and switches the input source. Undo: Backspace right after a conversion, or tap
Shift twice.

## Repository
- `core/` — shared Rust core (scorer, models, per-keystroke tracker, C ABI)
- `app/` — macOS menu-bar app (Swift)
- `detector/` — Python research code: training, evaluation, model export
- `convert.js`, `build.py` — the original "select text and press a shortcut" macOS Quick Action

License: MIT
