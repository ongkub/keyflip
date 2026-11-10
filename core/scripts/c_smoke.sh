#!/usr/bin/env bash
set -euo pipefail
export PATH="/opt/homebrew/opt/rustup/bin:$PATH"
root="$(cd "$(dirname "$0")/../.." && pwd)"
scratch="$(mktemp -d "${TMPDIR:-/tmp}/keyflip-c-smoke.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT
mkdir -p "$scratch/models"
cp "$root/core/tests/fixtures/tiny.kfm" "$scratch/models/th.kfm"
cp "$root/core/tests/fixtures/tiny.kfm" "$scratch/models/en.kfm"
cargo build --manifest-path "$root/Cargo.toml" --offline --release --target-dir "$root/target"
case "$(uname -s)" in
    Darwin) libs=(-framework Security -framework CoreFoundation -liconv -lresolv) ;;
    *) libs=(-ldl -lpthread -lm) ;;
esac
cc -std=c99 -Wall -Wextra -Werror -I "$root/core/include" \
    "$root/core/tests/c_smoke.c" "$root/target/release/libkeyflip_core.a" \
    "${libs[@]}" -o "$scratch/c_smoke"
"$scratch/c_smoke" "$scratch/models"
