#!/bin/bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
APP="$ROOT/app/build/KeyFlipAuto.app"
BIN="$APP/Contents/MacOS/KeyFlipAuto"
fail() { echo "package: $*" >&2; exit 1; }

KEYFLIP_UNIVERSAL=1 bash app/build.sh || fail "universal build failed"
[[ -f "$APP/Contents/Info.plist" ]] || fail "built Info.plist is missing"
VERSION="$(/usr/libexec/PlistBuddy -c 'Print CFBundleShortVersionString' "$APP/Contents/Info.plist")"
ARCHS="$(lipo -archs "$BIN" 2>/dev/null)" || fail "cannot inspect built binary architectures"
[[ " $ARCHS " == *" arm64 "* && " $ARCHS " == *" x86_64 "* ]] || fail "binary must contain arm64 and x86_64 (found: $ARCHS)"
for model in th en; do [[ -s "$APP/Contents/Resources/models/$model.kfm" ]] || fail "model missing or empty: $model.kfm"; done
codesign --verify --deep --strict "$APP" || fail "codesign verification failed"
README="${KEYFLIP_README:-app/README-TH.txt}"
[[ -f "$README" ]] || fail "readme file not found: $README"
STAGING="$ROOT/dist/KeyFlip-macOS-beta"
ZIP="$ROOT/dist/KeyFlip-macOS-beta-$VERSION.zip"
rm -rf "$STAGING"
mkdir -p "$STAGING"
ditto "$APP" "$STAGING/KeyFlipAuto.app"
ditto "$README" "$STAGING/README-TH.txt"
mkdir -p "$ROOT/dist"
rm -f "$ZIP"
ditto -c -k --norsrc --noextattr --noqtn --keepParent "$STAGING" "$ZIP"
printf '%s\n' "$ZIP"
stat -f '%z' "$ZIP"
shasum -a 256 "$ZIP"
