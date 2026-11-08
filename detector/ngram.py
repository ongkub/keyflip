"""Character n-gram model with Witten-Bell backoff smoothing."""
from collections import Counter, defaultdict
from functools import lru_cache
import math

BEGIN = "\u0002"


class NgramLM:
    def __init__(self, order=5):
        self.order = order
        self.counts = [Counter() for _ in range(order + 1)]
        self.vocab = set()
        self.totals = [Counter() for _ in range(order)]
        self.types = [Counter() for _ in range(order)]

    def add(self, text, weight=1):
        seq = [BEGIN] * (self.order - 1) + list(text) + ["\u0003"]
        self.vocab.update(seq)
        for i in range(self.order - 1, len(seq)):
            for n in range(1, self.order + 1):
                gram = tuple(seq[i - n + 1:i + 1])
                self.counts[n][gram] += weight
    def finalize(self):
        # Build context totals once from n-gram counts.
        self.totals = [Counter() for _ in range(self.order)]
        self.types = [Counter() for _ in range(self.order)]
        for n in range(1, self.order + 1):
            for gram, count in self.counts[n].items():
                ctx = gram[:-1]
                self.totals[n - 1][ctx] += count
                self.types[n - 1][ctx] += 1

    @classmethod
    def load(cls, path):
        import gzip, json
        with gzip.open(path, "rt", encoding="utf8") as f:
            obj = json.load(f)
        model = cls(obj["order"])
        model.vocab = set(obj["vocab"])
        for n, table in enumerate(obj["counts"], 1):
            model.counts[n] = Counter({tuple(key): value for key, value in table.items()})
        model.finalize()
        return model

    def _prob(self, context, char):
        n = min(len(context) + 1, self.order)
        context = context[-(n - 1):] if n > 1 else ()
        count = self.counts[n].get(tuple(context) + (char,), 0)
        total = self.totals[n - 1].get(tuple(context), 0)
        types = self.types[n - 1].get(tuple(context), 0)
        if total == 0:
            if n == 1:
                return 1.0 / max(1, len(self.vocab))
            return self._prob(context[1:], char)
        # Witten-Bell: observed mass = total/(total+types); unseen mass backs off.
        lower = self._prob(context[1:], char) if n > 1 else 1.0 / max(1, len(self.vocab))
        # Interpolated Witten-Bell: the backed-off distribution contributes at
        # every context, so probabilities remain normalized over the alphabet.
        return count / (total + types) + (types / (total + types)) * lower

    @lru_cache(maxsize=500_000)
    def logprob(self, s, complete=True):
        seq = [BEGIN] * (self.order - 1) + list(s)
        if complete:
            seq.append("\u0003")
        ans = 0.0
        for i in range(self.order - 1, len(seq)):
            p = self._prob(seq[max(0, i - self.order + 1):i], seq[i])
            ans += math.log(max(p, 1e-300))
        return ans
