#!/bin/bash
set -euo pipefail
export PATH="/opt/homebrew/opt/rustup/bin:$PATH"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cargo build --release -p keyflip-core --manifest-path "$ROOT/Cargo.toml"
NATIVE_INFO="$(cargo rustc -p keyflip-core --release --manifest-path "$ROOT/Cargo.toml" -- --print native-static-libs 2>&1)"
NATIVE_LINE="$(printf '%s\n' "$NATIVE_INFO" | sed -n 's/.*native-static-libs: //p' | tail -1)"
NATIVE_FLAGS=()
if [ -n "$NATIVE_LINE" ]; then read -r -a NATIVE_FLAGS <<< "$NATIVE_LINE"; fi
mkdir -p "$ROOT/app/build/test-models" "$ROOT/app/build/module-cache"
cp "$ROOT/core/tests/fixtures/tiny.kfm" "$ROOT/app/build/test-models/th.kfm"
cp "$ROOT/core/tests/fixtures/tiny.kfm" "$ROOT/app/build/test-models/en.kfm"
cat > "$ROOT/app/build/test-pairs.json" <<'JSON'
[["a","ก"],["b","ข"]]
JSON
cp "$ROOT/app/Tests/CoreTests.swift" "$ROOT/app/build/main.swift"
swiftc -module-cache-path "$ROOT/app/build/module-cache" -swift-version 5 \
  -import-objc-header "$ROOT/core/include/keyflip.h" -I "$ROOT/core/include" \
  "$ROOT/app/Sources/Core.swift" "$ROOT/app/Sources/EngineBridge.swift" "$ROOT/app/build/main.swift" \
  -L "$ROOT/target/release" -lkeyflip_core "${NATIVE_FLAGS[@]}" -o "$ROOT/app/build/engine-smoke"
cd "$ROOT"
app/build/engine-smoke
