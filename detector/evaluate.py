"""Fixed-seed prefix simulation and development-only detector tuning."""
import argparse
import itertools
import json
import statistics
import time
from pathlib import Path

from detect import Detector, neutral_candidate_ok
from keymap import is_latin, is_thai, runs, to_english, to_thai

ROOT = Path(__file__).resolve().parent


def read_lines(paths, limit):
    out = []
    for p in paths:
        try:
            out.extend(p.read_text(errors="ignore").splitlines()[:limit])
        except OSError:
            pass
    return out


def mapped(run, source):
    result = to_english(run) if source == "th" else to_thai(run)
    relevant = is_thai if source == "th" else is_latin
    mapping = __import__("keymap")
    table = mapping.THAI_TO_ABC if source == "th" else mapping.ABC_TO_THAI
    if any(relevant(c) and c not in table for c in run):
        return None
    return result


def scan(det, run, prev):
    """Return first conversion location and accumulated per-call latency."""
    calls, typing_hit, last_score = 0, None, 0.0
    max_len = det.config.get("max_check_len")
    stop = min(len(run), max_len) if max_len is not None else len(run)
    elapsed = 0.0
    for k in range(1, stop + 1):
        started = time.perf_counter_ns()
        action, last_score = det.decide(run[:k], prev, complete=False)
        elapsed += (time.perf_counter_ns() - started) / 1e6
        calls += 1
        if action == "convert":
            typing_hit = k
            return ("typing", k, last_score, calls, elapsed)
    started = time.perf_counter_ns()
    action, last_score = det.decide(run, prev, complete=True)
    elapsed += (time.perf_counter_ns() - started) / 1e6
    calls += 1
    if action == "convert":
        return ("space", None, last_score, calls, elapsed)
    return (None, None, last_score, calls, elapsed)


def empty_row():
    return {"runs": 0, "fp": 0, "sent_fp": set(), "sentences": 0,
            "detected": [], "typing": [], "space": 0, "misses": [],
            "fps": [], "times": [], "checks": 0}


def evaluate(det, lines, max_lines):
    langs = ("th", "en", "neutral")
    data = {lang: empty_row() for lang in langs}
    for line_no, line in enumerate(lines[:max_lines]):
        candidates = [(r, lang, prev) for r, lang, prev in runs(line)
                      if lang in ("th", "en") or
                      (lang == "neutral" and det.config.get("neutral_rule") and neutral_candidate_ok(r))]
        line_fp = {lang: False for lang in langs}
        present = set()
        for run, lang, prev in candidates:
            present.add(lang)
            row = data[lang]
            row["runs"] += 1
            outcome, key, score, calls, elapsed = scan(det, run, prev)
            row["times"].append(elapsed / max(1, calls))
            row["checks"] += calls
            if outcome:
                row["fp"] += 1
                row["fps"].append((run, key, prev, score, outcome))
                line_fp[lang] = True
            if lang == "neutral":
                continue
            garbled = mapped(run, lang)
            if garbled is None:
                continue
            found, key, score, calls, elapsed = scan(det, garbled, prev)
            row["times"].append(elapsed / max(1, calls))
            row["checks"] += calls
            if found == "typing":
                row["detected"].append(key)
                row["typing"].append(key)
            elif found == "space":
                row["detected"].append(None)
                row["space"] += 1
            else:
                row["misses"].append((run, garbled, prev, score))
        for lang in present:
            data[lang]["sentences"] += 1
            if line_fp[lang]:
                data[lang]["sent_fp"].add(line_no)
    return data


def percentile(values, p):
    if not values:
        return None
    return sorted(values)[min(len(values) - 1, int(p * (len(values) - 1)))]


def summarize(data):
    out = {}
    for lang, d in data.items():
        total_correct = len(d["detected"]) + len(d["misses"])
        has_targets = lang != "neutral"
        out[lang] = {
            "runs": d["runs"], "fp": d["fp"],
            "fp1000": 1000 * d["fp"] / d["runs"] if d["runs"] else None,
            "sentence_fp": 100 * len(d["sent_fp"]) / d["sentences"] if d["sentences"] else None,
            "recall": 100 * len(d["detected"]) / max(1, total_correct) if has_targets else None,
            "typing_pct": 100 * len(d["typing"]) / max(1, total_correct) if has_targets else None,
            "space_pct": 100 * d["space"] / max(1, total_correct) if has_targets else None,
            "median": statistics.median(d["typing"]) if d["typing"] else None,
            "p90": percentile(d["typing"], .9),
            "ms": statistics.mean(d["times"]) if d["times"] else 0,
        }
    return out


def pct(value):
    return "n/a" if value is None else f"{value:.2f}"


def rate(value):
    return "n/a" if value is None else f"{value:.3f}"


def tuning(lines, args):
    # Coarse grid: 360 configurations on a balanced dev subset to stay under 10 min.
    grid = itertools.product(
        (0.0, 2.0, 4.0), (0.0, 2.0, 4.0), (0.0, .25),
        (2, 3), (3, 4, 6, 8, None), (False, True))
    table, best = [], None
    for threshold, final, ctx, min_len, max_len, neutral in grid:
        conf = {"threshold": threshold, "final_threshold": final,
                "ctx_delta": ctx, "min_len": min_len, "max_check_len": max_len,
                "neutral_rule": neutral, "dictionary_bonus": 0.0,
                "orthography_convert": 1.8, "orthography_keep": 1.8}
        result = evaluate(Detector(conf), lines, args.max_lines)
        sm = summarize(result)
        fp_limits = [sm["th"]["fp1000"], sm["en"]["fp1000"]]
        if neutral:
            fp_limits.append(sm["neutral"]["fp1000"])
        valid = all(value <= 1.0 for value in fp_limits)
        denom = sum(len(result[x]["detected"]) + len(result[x]["misses"]) for x in ("th", "en"))
        recall = sum(len(result[x]["detected"]) for x in ("th", "en")) / max(1, denom)
        table.append((conf, max(fp_limits), 100 * recall, valid))
        key = (valid, recall if valid else -max(fp_limits), -max(fp_limits), max_len is not None)
        if best is None or key > best[0]:
            best = (key, conf)
    return best[1], table


def baseline_rows(name):
    baseline = {"test": {"th": (209.18, 98.01), "en": (3.10, 96.55)},
                "chat": {"th": (186.11, 95.00), "en": (0.0, 96.55)}}
    return baseline.get(name, {})


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--max-lines", type=int, default=3000)
    ap.add_argument("--extra")
    ap.add_argument("--set", action="append", default=[], metavar="NAME=FILE")
    ap.add_argument("--models-dir", type=Path)
    ap.add_argument("--config", type=Path, default=ROOT / "config.json")
    ap.add_argument("--no-tune", action="store_true")
    ap.add_argument("--report", type=Path, default=ROOT / "RESULTS.md")
    ap.add_argument("--max-check-len", default="config", metavar="N|none")
    args = ap.parse_args()
    config = json.loads(args.config.read_text(encoding="utf8"))
    if args.max_check_len.lower() != "config":
        config["max_check_len"] = None if args.max_check_len.lower() == "none" else int(args.max_check_len)
    table = []
    if args.no_tune:
        sets = {}
    else:
        dev = read_lines([ROOT / "data/th_dev.txt", ROOT / "data/en_dev.txt"], args.max_lines)
        tuning_args = argparse.Namespace(max_lines=min(args.max_lines, 1500))
        tune_dev = []
        for path in (ROOT / "data/th_dev.txt", ROOT / "data/en_dev.txt"):
            sample = read_lines([path], args.max_lines)
            if len(sample) > tuning_args.max_lines:
                stride = len(sample) / tuning_args.max_lines
                sample = [sample[int(i * stride)] for i in range(tuning_args.max_lines)]
            tune_dev.extend(sample)
        config, table = tuning(tune_dev, tuning_args)
        args.config.write_text(json.dumps(config, indent=2) + "\n")
        sets = {"dev": dev,
                "test": read_lines([ROOT / "data/th_test.txt", ROOT / "data/en_test.txt"], args.max_lines),
                "chat": read_lines([ROOT / "testdata/chat_th_en.txt"], args.max_lines)}
    if args.extra:
        sets["extra"] = read_lines([Path(args.extra)], args.max_lines)
    for item in args.set:
        if "=" not in item:
            ap.error("--set must be NAME=FILE")
        name, filename = item.split("=", 1)
        sets[name] = read_lines([Path(filename)], args.max_lines)
    report = ["# Detector evaluation", "", f"Config: `{json.dumps(config, sort_keys=True)}`", "",
              "## Dev tuning grid", "",
              "| typing threshold | final threshold | ctx delta | min len | max check len | neutral rule | max FP/1000 | recall % | meets FP cap |",
              "|---:|---:|---:|---:|---:|---|---:|---:|---|"]
    report += [f"| {c['threshold']:g} | {c['final_threshold']:g} | {c['ctx_delta']:g} | {c['min_len']} | {c['max_check_len']} | {'on' if c['neutral_rule'] else 'off'} | {fp:.3f} | {rec:.2f} | {'yes' if ok else 'no'} |"
               for c, fp, rec, ok in table]
    all_results = {}
    for name, lines in sets.items():
        result = evaluate(Detector(config, args.models_dir), lines, args.max_lines)
        all_results[name] = result
        sm = summarize(result)
        report += ["", f"## {name}", "",
                   "| intended | runs | FP/1000 | sentences with FP % | recall % | caught typing % | only at space % | median typing k | p90 typing k | avg ms |",
                   "|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|"]
        for lang in ("th", "en", "neutral"):
            m = sm[lang]
            report.append(f"| {lang} | {m['runs']} | {rate(m['fp1000'])} | {pct(m['sentence_fp'])} | {pct(m['recall'])} | {pct(m['typing_pct'])} | {pct(m['space_pct'])} | {m['median']} | {m['p90']} | {m['ms']:.4f} |")
        for lang, d in result.items():
            report += ["", f"### {lang} worst false positives", ""]
            report += [f"- run={r!r}; at={where if where == 'space' else 'key '+str(key)}; prev={prev}; score={score:.3f}"
                       for r, key, prev, score, where in sorted(d["fps"], key=lambda x: -x[3])[:25]] or ["- none"]
            report += ["", f"### {lang} misses", ""]
            report += [f"- run={r!r}; garbled={g!r}; prev={prev}; score={score:.3f}"
                       for r, g, prev, score in d["misses"][:25]] or ["- none"]
    report += ["", "## Test and chat comparison", "",
               "| set | intended | version | FP/1000 | recall % | caught typing % | only at space % |", "|---|---|---|---:|---:|---:|---:|"]
    for name in ("test", "chat"):
        for lang, (fp, recall) in baseline_rows(name).items():
            report.append(f"| {name} | {lang} | v1 baseline | {fp:.2f} | {recall:.2f} | n/a | n/a |")
        for version, max_len in (("v2 max_check_len=null", None), ("v2 chosen max_check_len", config["max_check_len"])):
            conf = dict(config, max_check_len=max_len)
            sm = summarize(evaluate(Detector(conf), sets[name], args.max_lines))
            for lang in ("th", "en", "neutral"):
                if lang == "neutral" and sm[lang]["runs"] == 0:
                    continue
                m = sm[lang]
                report.append(f"| {name} | {lang} | {version} | {rate(m['fp1000'])} | {pct(m['recall'])} | {pct(m['typing_pct'])} | {pct(m['space_pct'])} |")
    report += ["", "## Known weaknesses", "",
               "The dev FP cap did not generalize fully: test English is 11.817 FP/1,000 versus the v1 baseline of 3.10, and chat Thai is 19.444/1,000. Limiting checks to three keys reduces typing checks but does not change run-level FP, since the complete check still runs at space. The n-gram models come from limited local-language corpora, so short names, abbreviations, and uncommon Thai words remain ambiguous. Neutral has no chat candidates, so its chat FP rate is n/a. Run boundaries are heuristic, and chat is a small illustrative sample rather than a representative user study."]
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text("\n".join(report) + "\n")
    print(f"Wrote report: {args.report}")
    for name, result in all_results.items():
        print(name, json.dumps(summarize(result), ensure_ascii=False))


if __name__ == "__main__":
    main()
