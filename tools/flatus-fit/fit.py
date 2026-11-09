"""Fit the flatulence creature's physical controls to each recorded event.

    python3 -I fit.py <events.json> <library.json> <out.json> <tmpdir> [only=id,id] [scale=<scale.json>]

Coarse stage: the nearest renders of the library (library.py, push held until the gas is gone).
Fine stage: Nelder–Mead from the best library points over the controls (push, tone, gas, cheek
spread, body size) and the push's timing: one push of a fitted length, or, for events with
several bursts, a train of pushes (count from the event's bursts, fitted period and duty), as the
staccato call does. Nothing but the creature's documented controls is fitted.

The distance's feature scales come from the real events only (features.feature_scale), so they are
the same for every model version; pass `scale=` to reuse a stored one.
"""
import json, os, sys, time
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import numpy as np
from scipy.optimize import minimize
import features as F
from sim import Renderer, RANGES
from library import sim_features

LOG = {"tone", "gas"}
KEYS = ["push", "tone", "gas", "spread", "size"]  # extended by the library's own keys
STEP = {"push": 8.0, "tone": 0.5, "gas": 0.5, "spread": 0.3, "size": 0.15, "mucus": 0.8}
STARTS = 2
MAXFEV = 70


def real_nn(events, scale):
    """Each event's distance to the nearest event of another recording."""
    out = {}
    for e in events:
        ds = [F.distance(e["feat"], o["feat"], scale) for o in events if o["rec"] != e["rec"]]
        j = int(np.argmin(ds))
        out[e["id"]] = float(ds[j])
    return out


def to_x(p):
    return [np.log(p[k]) if k in LOG else p[k] for k in KEYS]


def bounds():
    return [(np.log(RANGES[k][0]), np.log(RANGES[k][1])) if k in LOG else RANGES[k] for k in KEYS]


def from_x(x):
    return {k: float(np.exp(v) if k in LOG else v) for k, v in zip(KEYS, x)}


def pushes_of(mode, x_t, n):
    if mode == "single":
        return [(0.0, float(np.exp(x_t[0])))]
    period, duty = float(np.exp(x_t[0])), float(x_t[1])
    return [(i * period, i * period + duty * period) for i in range(n)]


def fit_event(r, e, lib, scale):
    fe = e["feat"]
    secs = float(max(3.0, min(10.0, 2.5 * fe["dur"] + 1.0)))
    dl = np.array([F.distance(fe, l["feat"], scale) for l in lib])
    order = np.argsort(dl)
    best = {"d": float(dl[order[0]]), "params": lib[order[0]]["params"], "pushes": None, "mode": "library"}
    coarse = float(dl[order[0]])
    starts = []
    for j in order:
        p = lib[j]["params"]
        if all(any(abs(np.log((p[k] + 1) / (q["params"][k] + 1))) > 0.3 for k in KEYS) for q in starts):
            starts.append(lib[j])
        if len(starts) >= STARTS:
            break
    nb = int(round(2 ** fe["bursts"]))
    modes = [("single", s) for s in starts]
    if 2 <= nb <= 12:
        modes.append(("train", starts[0]))
    nev = 0
    for mode, s in modes:
        x0 = to_x(s["params"])
        if mode == "single":
            xt0, bt = [np.log(max(0.05, fe["dur"]))], [(np.log(0.03), np.log(8.0))]
        else:
            xt0, bt = [np.log(max(0.03, fe["dur"] / nb)), 0.5], [(np.log(0.02), np.log(1.5)), (0.15, 0.9)]
        nt = len(xt0)

        def cost(x):
            nonlocal nev
            nev += 1
            p = from_x(x[:-nt])
            pu = pushes_of(mode, x[-nt:], nb)
            sf = sim_features(r, p, pu, secs)
            if sf is None:
                return 10.0
            d = F.distance(fe, sf, scale)
            if d < best["d"]:
                best.update(d=d, params=p, pushes=pu, mode=mode)
            return d

        x0 = np.array(list(x0) + xt0)
        bb = bounds() + bt
        step = np.array([STEP[k] for k in KEYS] + ([0.5] if nt == 1 else [0.4, 0.2]))
        sim0 = [x0] + [np.clip(x0 + np.eye(len(x0))[i] * step, [b[0] for b in bb], [b[1] for b in bb]) for i in range(len(x0))]
        minimize(cost, x0, method="Nelder-Mead", bounds=bb,
                 options={"maxfev": MAXFEV, "initial_simplex": np.array(sim0), "xatol": 1e-3, "fatol": 1e-3})
    # the best fit's per-feature misses
    sf = sim_features(r, best["params"], best["pushes"], secs)
    d, terms = F.distance(fe, sf, scale, detail=True) if sf else (10.0, {})
    return {"id": e["id"], "rec": e["rec"], "coarse": coarse, "d": d, "params": best["params"], "pushes": best["pushes"],
            "mode": best["mode"], "terms": terms, "evals": nev, "sim": {k: sf[k] for k in F.NAMES + ["dur", "voiced"]} if sf else None}


def main():
    events = json.load(open(sys.argv[1]))
    lib = json.load(open(sys.argv[2]))
    for k in lib[0]["params"]:
        if k not in KEYS:
            KEYS.append(k)
    outp, tmp = sys.argv[3], sys.argv[4]
    opts = dict(a.split("=", 1) for a in sys.argv[5:])
    if "scale" in opts:
        scale = np.array(json.load(open(opts["scale"])))
    else:
        scale = F.feature_scale([e["feat"] for e in events])
    if "only" in opts:
        events_fit = [e for e in events if e["id"] in opts["only"].split(",")]
    else:
        events_fit = events
    nn = real_nn(events, scale)
    try:
        done = {f["id"]: f for f in json.load(open(outp))["fits"]}
    except Exception:
        done = {}
    r = Renderer(tmp)
    t0 = time.time()
    for i, e in enumerate(events_fit):
        if e["id"] in done:
            continue
        f = fit_event(r, e, lib, scale)
        f["nn"] = nn[e["id"]]
        done[e["id"]] = f
        print(f"{i + 1}/{len(events_fit)} {e['id']}: coarse {f['coarse']:.2f} fit {f['d']:.2f} (real NN {f['nn']:.2f}) {f['evals']} renders {time.time() - t0:.0f}s", file=sys.stderr, flush=True)
        json.dump({"scale": scale.tolist(), "names": F.NAMES, "nn": nn, "fits": list(done.values())}, open(outp, "w"), indent=1)
    r.close()


if __name__ == "__main__":
    main()
