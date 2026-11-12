# KeyFlip

แก้คำที่พิมพ์ผิดภาษาให้อัตโนมัติระหว่างพิมพ์ (แป้นไทยเกษมณี ⇄ อังกฤษ) — พิมพ์ `vpkdwxdbo-hk;` แล้วได้ "อยากไปกินข้าว"
พร้อมสลับแป้นให้เอง · Fixes words typed with the wrong keyboard layout (Thai Kedmanee ⇄ English) while you type.

**Status: public beta 0.3.0** for macOS 13+ (Apple Silicon and Intel) and Windows 64-bit.

## ดาวน์โหลด · Download
[หน้าดาวน์โหลด (Releases)](https://github.com/ongkub/keyflip/releases) — เลือก `KeyFlip-macOS-beta-…zip` หรือ
`KeyFlip-Windows-beta-…zip` ในแต่ละ zip มีคู่มือติดตั้งภาษาไทย `README-TH.txt`
รุ่นทดลองยังไม่ได้ลงทะเบียนกับ Apple/Microsoft จึงมีคำเตือนตอนเปิดครั้งแรก (วิธีผ่านอยู่ในคู่มือ)
เจอปัญหาแจ้งผ่านเมนู "แจ้งปัญหา…" ในแอป · Problems: use "แจ้งปัญหา…" (Report a problem) in the app menu.

## Privacy
- Everything runs on your computer. Nothing you type is sent anywhere.
- The local log keeps only words that KeyFlip converted or that you undid; everything else is counted, never stored.
- "Report a problem" sends only what you see in the report window, and only when you press send.

## How it decides
Two character 5-gram language models (Thai, English) score each word as you type; if the word reads much better in
the other layout, KeyFlip replaces it and switches the input source. Undo: Backspace right after a conversion, or tap
Shift twice.

You can also convert text after typing: select text in any app and press Control+Shift+Space (⌃⇧Space on Mac) or Ctrl+Alt+Space (Windows) to replace the selection with the converted text. It converts only words typed in the wrong layout (or flips the whole selection if every word already looks right); undo with Command+Z or Ctrl+Z in that app. This works even when auto-fix is off, never runs in password fields or excluded apps (Terminal, VS Code, etc.), and briefly uses the clipboard while restoring previous clipboard content and marking it so clipboard managers skip it. Enabled by default via the menu item "แปลงข้อความที่เลือก".

## Repository
- `core/` — shared Rust core (scorer, models, per-keystroke tracker, C ABI)
- `app/` — macOS menu-bar app (Swift); `app/package.sh` makes the universal download zip
- `win/` — Windows tray app (Rust)
- `server/` — problem-report receiver (Python, Docker)
- `detector/` — Python research code: training, evaluation, model export
- `convert.js`, `build.py` — the original "select text and press a shortcut" macOS Quick Action

License: MIT
