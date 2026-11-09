"""Summarise one or more fit files (fit.py) as a markdown table.

    python3 -I report.py <fit.json> [<fit2.json> …]

The threshold for "close" is the median, over the real events, of each event's distance to the
nearest event of another recording: a fit counts as close when the model is at least as near
the recording as a typical other real recording is. A recording counts as reached when its
nearest-fitting event is close. With several fit files, the columns compare them.
"""
import json, sys
import numpy as np


def misses(f, n=3):
    t = sorted(f["terms"].items(), key=lambda kv: -kv[1][1] * kv[1][0] ** 2)[:n]
    return ", ".join(f"{k} {v[0]:+.1f}" for k, v in t if v[1] * v[0] ** 2 > 0.5)


def fmt_params(f):
    p = f["params"]
    s = f"push {p['push']:.0f}, tone {p['tone']:.0f}, gas {p['gas']:.0f}, spread {p['spread']:.2f}, size {p['size']:.2f}"
    for k in sorted(set(p) - {"push", "tone", "gas", "spread", "size"}):
        s += f", {k} {p[k]:.2f}"
    pu = f.get("pushes")
    if pu and len(pu) > 1:
        s += f"; {len(pu)} pushes of {pu[0][1] - pu[0][0]:.2f} s every {pu[1][0] - pu[0][0]:.2f} s"
    elif pu:
        s += f"; push {pu[0][1]:.2f} s"
    return s


def main():
    runs = [json.load(open(p)) for p in sys.argv[1:]]
    nn = runs[0]["nn"]
    thr = float(np.median(list(nn.values())))
    fits = [{f["id"]: f for f in r["fits"]} for r in runs]
    ids = [f["id"] for f in runs[0]["fits"]]
    print(f"Threshold (median nearest-other-recording distance): {thr:.2f}\n")
    head = "| Event | real NN | " + " | ".join(f"d ({i})" for i in range(len(runs))) + " | best fit (last) | misses (last) |"
    print(head)
    print("|" + "---|" * (head.count("|") - 1))
    for e in ids:
        ds = [fs[e]["d"] if e in fs else float("nan") for fs in fits]
        last = fits[-1].get(e)
        mark = ["**%.2f**" % d if d <= thr else "%.2f" % d for d in ds]
        print(f"| {e} | {nn[e]:.2f} | " + " | ".join(mark) + f" | {fmt_params(last) if last else ''} | {misses(last) if last else ''} |")
    print()
    for i, fs in enumerate(fits):
        ev = [fs[e] for e in ids if e in fs]
        close = sum(f["d"] <= thr for f in ev)
        own = sum(f["d"] <= nn[f["id"]] for f in ev)
        recs = {}
        for f in ev:
            recs[f["rec"]] = min(recs.get(f["rec"], 9e9), f["d"])
        rc = sum(d <= thr for d in recs.values())
        print(f"run {i}: median d {np.median([f['d'] for f in ev]):.2f}; events close {close}/{len(ev)}; "
              f"events at least as close as their own nearest real neighbour {own}/{len(ev)}; recordings reached {rc}/{len(recs)}")
        # which features miss most across events (weighted squared z, mean)
        names = runs[i]["names"]
        tot = {k: np.mean([f["terms"][k][1] * f["terms"][k][0] ** 2 for f in ev]) for k in names}
        sgn = {k: np.mean([f["terms"][k][0] for f in ev if f["terms"][k][1] > 0] or [0]) for k in names}
        top = sorted(tot.items(), key=lambda kv: -kv[1])[:8]
        print("   largest mean misses (weighted z², mean signed z real − model): " + "; ".join(f"{k} {v:.2f} ({sgn[k]:+.2f})" for k, v in top))


if __name__ == "__main__":
    main()
