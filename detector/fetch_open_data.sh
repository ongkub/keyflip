#!/bin/sh
# Download the openly licensed text used to train KeyFlip's shareable generic model into data-open/raw/.
#   Wisesight Sentiment corpus  - CC0-1.0 - https://github.com/PyThaiNLP/wisesight-sentiment
#   Common Voice Thai sentences - CC0-1.0 (sentences), repo MIT - https://github.com/PyThaiNLP/th-cv-sentences
set -eu
cd "$(dirname "$0")/.."
mkdir -p data-open/raw
for f in pos neu neg q; do
  curl -fsSL "https://raw.githubusercontent.com/PyThaiNLP/wisesight-sentiment/master/$f.txt" -o "data-open/raw/wisesight-$f.txt"
done
curl -fsSL "https://raw.githubusercontent.com/PyThaiNLP/th-cv-sentences/main/approved.txt" -o data-open/raw/cv-th-approved.txt
ls -l data-open/raw
