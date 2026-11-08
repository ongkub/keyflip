#!/bin/bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP="$ROOT/app/build/KeyFlipAuto.app"
mkdir -p "$ROOT/app/build" "$APP/Contents/MacOS" "$APP/Contents/Resources/detector/models"
swiftc -module-cache-path "$ROOT/app/build/module-cache" -O -swift-version 5 -target "$(uname -m)-apple-macos13.0" "$ROOT"/app/Sources/*.swift -framework AppKit -framework Carbon -framework ApplicationServices -o "$APP/Contents/MacOS/KeyFlipAuto"
cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict><key>CFBundleIdentifier</key><string>local.keyflip.auto</string><key>CFBundleName</key><string>KeyFlip Auto</string><key>CFBundleExecutable</key><string>KeyFlipAuto</string><key>CFBundlePackageType</key><string>APPL</string><key>LSUIElement</key><true/><key>LSMinimumSystemVersion</key><string>13.0</string><key>CFBundleShortVersionString</key><string>0.1</string><key>CFBundleVersion</key><string>0.1</string></dict></plist>
PLIST
cp "$ROOT/pairs.json" "$APP/Contents/Resources/pairs.json"
for f in keymap.py ngram.py detect.py serve.py; do if [ -f "$ROOT/detector/$f" ]; then cp "$ROOT/detector/$f" "$APP/Contents/Resources/detector/$f"; elif [ "$f" = serve.py ]; then echo "warning: detector/serve.py is missing" >&2; fi; done
cp "$ROOT/detector/config.json" "$APP/Contents/Resources/detector/config.json"
if [ -d "$ROOT/detector/models" ]; then cp "$ROOT"/detector/models/*.json.gz "$APP/Contents/Resources/detector/models/" 2>/dev/null || true; fi
codesign --force --sign - "$APP"
if [ "${1:-}" = "--install" ]; then mkdir -p "$HOME/Applications"; rm -rf "$HOME/Applications/KeyFlipAuto.app"; cp -R "$APP" "$HOME/Applications/KeyFlipAuto.app"; fi
