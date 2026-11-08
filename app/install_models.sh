#!/bin/bash
set -euo pipefail
[ "$#" -eq 1 ] || { echo "usage: $0 DIR" >&2; exit 2; }
DEST="$HOME/Library/Application Support/KeyFlipAuto/models"
mkdir -p "$DEST"
cp "$1/th.json.gz" "$1/en.json.gz" "$DEST/"
