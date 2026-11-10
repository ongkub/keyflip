"""Export NgramLM to KFM1; see core/src/model.rs for the binary format."""
import argparse
import math
from pathlib import Path
import struct

from ngram import BEGIN, NgramLM


def write_model(model, path):
    """Precompute interpolated probabilities; never serialize training text."""
    if not 1 <= model.order <= 8:
        raise ValueError("KFM1 requires an order between 1 and 8")
    alphabet = sorted(model.vocab | {BEGIN, "\x03"})
    if len(alphabet) > 256 or any(len(c) != 1 for c in alphabet):
        raise ValueError("KFM1 requires at most 256 single-character symbols")
    indices = {c: i for i, c in enumerate(alphabet)}

    def key(chars):
        value = 0
        for char in chars:
            value = (value << 8) | indices[char]
        return value

    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("wb") as out:
        out.write(b"KFM1")
        out.write(struct.pack("<III", 1, model.order, len(alphabet)))
        for char in alphabet:
            encoded = char.encode("utf-8")
            out.write(struct.pack("<I", len(encoded)))
            out.write(encoded)
        out.write(struct.pack("<d", -math.log(max(1, len(model.vocab)))))
        for n in range(1, model.order + 1):
            seen = sorted((key(g), math.log(max(model._prob(g[:-1], g[-1]), 1e-300)))
                          for g in model.counts[n])
            contexts = sorted((key(ctx), math.log(model.types[n - 1][ctx] /
                               (total + model.types[n - 1][ctx])))
                              for ctx, total in model.totals[n - 1].items() if total)
            for table in (seen, contexts):
                out.write(struct.pack("<Q", len(table)))
                for encoded_key, log_value in table:
                    out.write(struct.pack("<Qf", encoded_key, log_value))
    return path.stat().st_size


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--in", dest="input", type=Path, default=Path("detector/models"))
    parser.add_argument("--out", type=Path, default=Path("models/generic"))
    args = parser.parse_args()
    for lang in ("th", "en"):
        model = NgramLM.load(args.input / f"{lang}.json.gz")
        path = args.out / f"{lang}.kfm"
        print(f"{path}: {write_model(model, path):,} bytes")


if __name__ == "__main__":
    main()
