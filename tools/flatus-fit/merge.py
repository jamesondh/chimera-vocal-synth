"""Merge several fit files of the same model (different libraries or starts) into one, keeping
each event's best fit: they are all searches of the same model, so the best is its fit. With
course stages (fit.py courses=1), each stage's best over the searches is kept as well.

    python3 -I merge.py <out.json> <fit.json> <fit.json> …
"""
import json, sys


def main():
    runs = [json.load(open(p)) for p in sys.argv[2:]]
    best = {}
    for r in runs:
        for f in r["fits"]:
            if f["id"] not in best or f["d"] < best[f["id"]]["d"]:
                best[f["id"]] = f
    # each course stage's best over the searches too (a stage's best is never better than the next's)
    for k in ("d_const", "d_ramp"):
        for i, f in best.items():
            vals = [g[k] for r in runs for g in r["fits"] if g["id"] == i and g.get(k) is not None]
            if vals:
                best[i] = {**f, k: min(vals)}
    out = dict(runs[0])
    out["fits"] = [best[f["id"]] for f in runs[0]["fits"]]
    json.dump(out, open(sys.argv[1], "w"), indent=1)


if __name__ == "__main__":
    main()
