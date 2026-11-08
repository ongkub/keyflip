"""Fixed-seed prefix simulation and development-only threshold tuning."""
import argparse
import json
import random
import statistics
import time
from pathlib import Path
from detect import Detector
from keymap import is_latin, is_thai, runs, to_english, to_thai

ROOT = Path(__file__).resolve().parent
SEED = 8152026


def read_lines(paths, limit):
    out = []
    for p in paths:
        try:
            out.extend((p.read_text(errors="ignore").splitlines())[:limit])
        except OSError:
            pass
    return out


def mapped(run, source):
    result = to_english(run) if source == "th" else to_thai(run)
    relevant = is_thai if source == "th" else is_latin
    if any(relevant(c) and ((source == "th" and c not in __import__('keymap').THAI_TO_ABC) or
                            (source == "en" and c not in __import__('keymap').ABC_TO_THAI)) for c in run):
        return None
    return result


def evaluate(det, lines, max_lines):
    by_lang = {x: {"runs": 0, "fp": 0, "sent_fp": set(), "sentences": 0, "detected": [], "misses": [], "fps": [], "times": []} for x in ("th", "en")}
    for line_no, line in enumerate(lines[:max_lines]):
        line_fp = {"th": False, "en": False}
        candidates = [(r, lang, prev) for r, lang, prev in runs(line) if lang in ("th", "en")]
        for run, lang, prev in candidates:
            row = by_lang[lang]; row["runs"] += 1
            fp = None
            for k in range(1, len(run) + 1):
                t = time.perf_counter_ns(); action, score = det.decide(run[:k], prev)
                row["times"].append((time.perf_counter_ns() - t) / 1e6)
                if action == "convert":
                    fp = (run, run[:k], prev, score)
                    break
            if fp:
                row["fp"] += 1; row["fps"].append(fp); line_fp[lang] = True
            garbled = mapped(run, lang)
            if garbled is not None:
                found = None
                for k in range(1, len(garbled) + 1):
                    action, score = det.decide(garbled[:k], prev)
                    if action == "convert":
                        found = k; break
                if found is None:
                    row["misses"].append((run, garbled, prev, score if garbled else 0))
                else:
                    row["detected"].append(found)
        for lang in ("th", "en"):
            if any(l == lang for _, l, _ in candidates):
                by_lang[lang]["sentences"] += 1
                if line_fp[lang]: by_lang[lang]["sent_fp"].add(line_no)
    return by_lang


def summarize(data):
    out = {}
    for lang, d in data.items():
        out[lang] = {"runs": d["runs"], "fp1000": 1000*d["fp"]/max(1,d["runs"]),
                     "sentence_fp": 100*len(d["sent_fp"])/max(1,d["sentences"]),
                     "recall": 100*len(d["detected"])/max(1,len(d["detected"])+len(d["misses"])),
                     "median": statistics.median(d["detected"]) if d["detected"] else None,
                     "p90": sorted(d["detected"])[int(.9*(len(d["detected"])-1))] if d["detected"] else None,
                     "within4": 100*sum(x<=4 for x in d["detected"])/max(1,len(d["detected"])+len(d["misses"])),
                     "ms": statistics.mean(d["times"]) if d["times"] else 0}
    return out


def tuning(lines, args):
    table=[]; best=None
    for th in (-2.0,-1.0,0.0,0.5,1.0,1.5,2.0,3.0,4.0):
        for ctx in (0.0,0.25):
            conf={"threshold":th,"ctx_delta":ctx,"min_len":2,"dictionary_bonus":0.0,
                  "orthography_convert":1.8,"orthography_keep":1.8}
            result=evaluate(Detector(conf),lines,args.max_lines)
            sm=summarize(result)
            fp=sum(x["fp1000"]*x["runs"] for x in sm.values())/max(1,sum(x["runs"] for x in sm.values()))
            recall=sum(len(result[x]["detected"]) for x in result)/max(1,sum(len(result[x]["detected"])+len(result[x]["misses"]) for x in result))
            table.append((th,ctx,fp,100*recall))
            valid=fp<=1.0
            key=(valid,100*recall if valid else -fp,-fp)
            if best is None or key>best[0]: best=(key,conf)
    return best[1],table


def main():
    ap=argparse.ArgumentParser(); ap.add_argument("--max-lines",type=int,default=3000); ap.add_argument("--extra")
    args=ap.parse_args(); random.seed(SEED)
    dev=read_lines([ROOT/"data/th_dev.txt",ROOT/"data/en_dev.txt"],args.max_lines)
    config,table=tuning(dev,args)
    (ROOT/"config.json").write_text(json.dumps(config,indent=2)+"\n")
    sets={"dev":dev,"test":read_lines([ROOT/"data/th_test.txt",ROOT/"data/en_test.txt"],args.max_lines),
          "chat":read_lines([ROOT/"testdata/chat_th_en.txt"],args.max_lines)}
    if args.extra: sets["extra"]=read_lines([Path(args.extra)],args.max_lines)
    report=["# Detector evaluation", "", f"Chosen config: `{json.dumps(config,sort_keys=True)}`", "",
            "## Dev threshold trade-off", "", "| threshold | ctx_delta | FP/1000 | recall % |", "|---:|---:|---:|---:|"]
    report += [f"| {t:g} | {c:g} | {fp:.3f} | {rec:.2f} |" for t,c,fp,rec in table]
    all_results={}
    for name,lines in sets.items():
        result=evaluate(Detector(config),lines,args.max_lines); all_results[name]=result
        sm=summarize(result)
        report += ["",f"## {name}","","| intended | runs | FP/1000 | sentences with FP % | recall % | median k | p90 k | within 4 % | avg ms |","|---|---:|---:|---:|---:|---:|---:|---:|---:|"]
        for lang,m in sm.items():
            report.append(f"| {lang} | {m['runs']} | {m['fp1000']:.3f} | {m['sentence_fp']:.2f} | {m['recall']:.2f} | {m['median']} | {m['p90']} | {m['within4']:.2f} | {m['ms']:.4f} |")
        for lang,d in result.items():
            report += ["",f"### {lang} worst false positives",""]
            report += [f"- run={r!r}; prefix={p!r}; prev={prev}; score={score:.3f}" for r,p,prev,score in sorted(d["fps"],key=lambda x:-x[3])[:25]] or ["- none"]
            report += ["",f"### {lang} misses",""]
            report += [f"- run={r!r}; garbled={g!r}; prev={prev}; score={score:.3f}" for r,g,prev,score in d["misses"][:25]] or ["- none"]
    report += ["", "## Known weaknesses", "", "Corpus is limited to strings exposed by local macOS localization tables; short prefixes and names absent from training remain ambiguous. Run boundaries are heuristic and dictionary coverage depends on `/usr/share/dict/words`. Chat data is an illustrative fixed sample, not a representative user study."]
    (ROOT/"RESULTS.md").write_text("\n".join(report)+"\n")
    print("Wrote detector/config.json and detector/RESULTS.md")
    for name,result in all_results.items(): print(name, json.dumps(summarize(result),ensure_ascii=False))


if __name__ == "__main__": main()
