#!/bin/sh
set -eu
HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
python3 "$HERE/build_corpus.py"
python3 "$HERE/train.py"
python3 "$HERE/evaluate.py"
