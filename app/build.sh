#!/bin/bash
set -euo pipefail
export PATH="/opt/homebrew/opt/rustup/bin:$PATH"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP="$ROOT/app/build/KeyFlipAuto.app"
mkdir -p "$ROOT/app/build" "$APP/Contents/MacOS"
rm -rf "$APP/Contents/Resources"
mkdir -p "$APP/Contents/Resources/models"
if [ "${KEYFLIP_UNIVERSAL:-0}" = "1" ]; then
  build_arch() {
    local triple="$1" arch="$2" native_info native_line
    cargo build --release -p keyflip-core --manifest-path "$ROOT/Cargo.toml" --target "$triple"
    native_info="$(CARGO_TERM_COLOR=never cargo rustc -p keyflip-core --release --manifest-path "$ROOT/Cargo.toml" --target "$triple" -- --print native-static-libs 2>&1)"
    native_line="$(printf '%s\n' "$native_info" | sed -n 's/.*native-static-libs: //p' | tail -1)"
    local native_flags=()
    if [ -n "$native_line" ]; then read -r -a native_flags <<< "$native_line"; fi
    swiftc -module-cache-path "$ROOT/app/build/module-cache-$arch" -O -swift-version 5 -target "$arch-apple-macos13.0" \
      -import-objc-header "$ROOT/core/include/keyflip.h" -I "$ROOT/core/include" -L "$ROOT/target/$triple/release" -lkeyflip_core "${native_flags[@]}" \
      "$ROOT"/app/Sources/*.swift -framework AppKit -framework Carbon -framework ApplicationServices -o "$ROOT/app/build/KeyFlipAuto-$arch"
  }
  build_arch aarch64-apple-darwin arm64
  build_arch x86_64-apple-darwin x86_64
  lipo -create "$ROOT/app/build/KeyFlipAuto-arm64" "$ROOT/app/build/KeyFlipAuto-x86_64" -output "$APP/Contents/MacOS/KeyFlipAuto"
else
  cargo build --release -p keyflip-core --manifest-path "$ROOT/Cargo.toml"
  NATIVE_INFO="$(CARGO_TERM_COLOR=never cargo rustc -p keyflip-core --release --manifest-path "$ROOT/Cargo.toml" -- --print native-static-libs 2>&1)"
  NATIVE_LINE="$(printf '%s\n' "$NATIVE_INFO" | sed -n 's/.*native-static-libs: //p' | tail -1)"
  NATIVE_FLAGS=()
  if [ -n "$NATIVE_LINE" ]; then read -r -a NATIVE_FLAGS <<< "$NATIVE_LINE"; fi
  swiftc -module-cache-path "$ROOT/app/build/module-cache" -O -swift-version 5 -target "$(uname -m)-apple-macos13.0" \
    -import-objc-header "$ROOT/core/include/keyflip.h" -I "$ROOT/core/include" -L "$ROOT/target/release" -lkeyflip_core "${NATIVE_FLAGS[@]}" \
    "$ROOT"/app/Sources/*.swift -framework AppKit -framework Carbon -framework ApplicationServices -o "$APP/Contents/MacOS/KeyFlipAuto"
fi
cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict><key>CFBundleIdentifier</key><string>local.keyflip.auto</string><key>CFBundleName</key><string>KeyFlip Auto</string><key>CFBundleExecutable</key><string>KeyFlipAuto</string><key>CFBundleIconFile</key><string>AppIcon</string><key>CFBundlePackageType</key><string>APPL</string><key>LSUIElement</key><true/><key>LSMinimumSystemVersion</key><string>13.0</string><key>CFBundleShortVersionString</key><string>0.3.0</string><key>CFBundleVersion</key><string>0.3.0</string></dict></plist>
PLIST
cp "$ROOT/app/AppIcon.icns" "$APP/Contents/Resources/AppIcon.icns"   # source: brand/app-icon.html
for lang in th en; do
  # Ship the open-data model (models/open, see detector/train_open.py); KEYFLIP_MODELS overrides.
  model="$ROOT/${KEYFLIP_MODELS:-models/open}/$lang.kfm"
  if [ -f "$model" ]; then cp "$model" "$APP/Contents/Resources/models/$lang.kfm"; else echo "warning: model missing: $model" >&2; fi
done
if [ -n "${KEYFLIP_SIGN_ID:-}" ]; then codesign --force --sign "$KEYFLIP_SIGN_ID" "$APP"; else codesign --force --sign - "$APP"; fi
if [ "${1:-}" = "--install" ]; then mkdir -p "$HOME/Applications"; rm -rf "$HOME/Applications/KeyFlipAuto.app"; cp -R "$APP" "$HOME/Applications/KeyFlipAuto.app"; fi
