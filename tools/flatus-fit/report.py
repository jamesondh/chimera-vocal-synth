"""Summarise one or more fit files (fit.py) as a markdown table.

    python3 -I report.py <fit.json> [<fit2.json> …] [--recordings]

`--recordings` prints one row per recording instead (its best-fitting event in the last run).
A fit file may hold several searches of the same model: `merge.py`.

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


ORDER = ["push", "tone", "eas", "ias", "gas", "spread", "size", "mucus"]
DIGITS = {"spread": 2, "size": 2, "mucus": 2}


def fmt_params(f):
    p = f["params"]
    keys = [k for k in ORDER if k in p] + sorted(set(p) - set(ORDER))
    s = ", ".join(f"{k} {p[k]:.{DIGITS.get(k, 0)}f}" for k in keys)
    pu = f.get("pushes")
    if pu and len(pu) > 1:
        s += f"; {len(pu)} pushes of {pu[0][1] - pu[0][0]:.2f} s every {pu[1][0] - pu[0][0]:.2f} s"
    elif pu:
        s += f"; push {pu[0][1]:.2f} s"
    c = f.get("course")
    if c:
        s += f"; eas {c['e0']:.0f} → {c['e1']:.0f} from {c['t0']:.2f} s over {c['dur']:.2f} s"
        if c.get("depth"):
            s += f", ±{c['depth']:.0f} at {c['rate']:.1f} Hz"
    return s


def by_recording(runs, thr):
    fits = [{f["id"]: f for f in r["fits"]} for r in runs]
    recs = {}
    for f in runs[-1]["fits"]:
        if f["rec"] not in recs or f["d"] < recs[f["rec"]]["d"]:
            recs[f["rec"]] = f
    head = "| Recording | " + " | ".join(f"best d ({i})" for i in range(len(runs))) + " | best event, fit (last) | misses (last) |"
    print(head)
    print("|" + "---|" * (head.count("|") - 1))
    for rec, last in recs.items():
        ds = [min(f["d"] for f in fs.values() if f["rec"] == rec) for fs in fits]
        mark = ["**%.2f**" % d if d <= thr else "%.2f" % d for d in ds]
        print(f"| {rec} | " + " | ".join(mark) + f" | {last['id']}: {fmt_params(last)} | {misses(last)} |")


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    runs = [json.load(open(p)) for p in args]
    if "--recordings" in sys.argv:
        by_recording(runs, float(np.median(list(runs[0]["nn"].values()))))
        return
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
        for stage in ("d_const", "d_ramp"):
            if all(f.get(stage) is not None for f in ev):
                sd = {f["id"]: f[stage] for f in ev}
                srec = {}
                for f in ev:
                    srec[f["rec"]] = min(srec.get(f["rec"], 9e9), sd[f["id"]])
                print(f"   best with the command {'constant' if stage == 'd_const' else 'up to a ramp'}: median d {np.median(list(sd.values())):.2f}; events close "
                      f"{sum(d <= thr for d in sd.values())}/{len(ev)}; recordings reached {sum(d <= thr for d in srec.values())}/{len(srec)}")
        # which features miss most across events (weighted squared z, mean)
        names = runs[i]["names"]
        tot = {k: np.mean([f["terms"][k][1] * f["terms"][k][0] ** 2 for f in ev]) for k in names}
        sgn = {k: np.mean([f["terms"][k][0] for f in ev if f["terms"][k][1] > 0] or [0]) for k in names}
        top = sorted(tot.items(), key=lambda kv: -kv[1])[:8]
        print("   largest mean misses (weighted z², mean signed z real − model): " + "; ".join(f"{k} {v:.2f} ({sgn[k]:+.2f})" for k, v in top))


if __name__ == "__main__":
    main()
