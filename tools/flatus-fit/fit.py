"""Fit the flatulence creature's physical controls to each recorded event.

    python3 -I fit.py <events.json> <library.json> <out.json> <tmpdir> [only=id,id] [scale=<scale.json>] [courses=1]

Coarse stage: the nearest renders of the library (library.py, push held until the gas is gone).
Fine stage: Nelder–Mead from the best library points over the controls (the library's: push, the
external sphincter's command and the internal sphincter's tone, gas, cheek spread, body size,
mucus) and the push's timing: one push of a fitted length, or, for events with several bursts, a
train of pushes (count from the event's bursts, fitted period and duty), as the staccato call
does. Nothing but the creature's documented controls is fitted.

Course stages (with `courses=1`, for a model with the external sphincter's command "eas"): from
the best constant fit, the command becomes a course. First a ramp: the command held at e0, then
moved linearly to e1 over `dur` s from t0 (four parameters: clench to cut off, relax to let go).
Then, from the best ramp, a sinusoidal modulation of the command around it, rate 0.5–8 Hz and
depth up to 60 cmH2O (two more). The creature passes the command through the muscle's
activation dynamics (10 ms up, 40 ms down), so the sphincter follows no faster than muscle can;
nothing shapes the sound itself. Each stage keeps its own best distance (d_const, d_ramp, d_mod),
so what the course adds can be read apart from the constant fit.

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
LOG1P = {"eas", "ias"}  # may be 0: log(1 + x)
KEYS = []  # the library's keys
STEP = {"push": 8.0, "tone": 0.5, "eas": 0.7, "ias": 0.7, "gas": 0.5, "spread": 0.3, "size": 0.15, "mucus": 0.8}
RAMP_FEV = 40
MOD_FEV = 25
MOD_RATE = (0.5, 8.0)
MOD_DEPTH = 60.0
STARTS = 2
COURSES = False
MAXFEV = 70


def real_nn(events, scale):
    """Each event's distance to the nearest event of another recording."""
    out = {}
    for e in events:
        ds = [F.distance(e["feat"], o["feat"], scale) for o in events if o["rec"] != e["rec"]]
        j = int(np.argmin(ds))
        out[e["id"]] = float(ds[j])
    return out


def fwd(k, v):
    return np.log(v) if k in LOG else np.log1p(v) if k in LOG1P else v


def inv(k, v):
    return float(np.exp(v) if k in LOG else np.expm1(v) if k in LOG1P else v)


def to_x(p):
    return [fwd(k, p[k]) for k in KEYS]


def bounds():
    return [(fwd(k, RANGES[k][0]), fwd(k, RANGES[k][1])) for k in KEYS]


def from_x(x):
    return {k: inv(k, v) for k, v in zip(KEYS, x)}


def course_of(c, until):
    """The external sphincter's command over time: e0 until t0, linear to e1 by t0 + dur, plus
    an optional modulation (rate, depth) around it, sampled every 10 ms up to `until` s."""
    keys = [(0.0, c["e0"]), (c["t0"], c["e0"]), (c["t0"] + c["dur"], c["e1"])]
    if not c.get("depth"):
        return keys
    ramp = lambda t: float(np.interp(t, [k[0] for k in keys], [k[1] for k in keys]))
    ts = np.arange(0.0, until, 0.01)
    return [(float(t), max(0.0, ramp(t) + c["depth"] * np.sin(2 * np.pi * c["rate"] * t))) for t in ts]


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
    best = {"d": float(dl[order[0]]), "params": lib[order[0]]["params"], "pushes": None, "mode": "library", "course": None}
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
        nm(cost, x0, step, bb, MAXFEV)
    d_const = best["d"]
    d_ramp = d_mod = None
    if COURSES and "eas" in KEYS:
        # the command's course: a ramp from the best constant fit, then a modulation around it
        until = float(min(secs, 1.5 * fe["dur"] + 0.5))
        e_fit = best["params"]["eas"]
        base = dict(best)

        def course_cost(c):
            nonlocal nev
            nev += 1
            sf = sim_features(r, base["params"], base["pushes"], secs, {"eas": course_of(c, until)})
            if sf is None:
                return 10.0
            d = F.distance(fe, sf, scale)
            if d < best["d"]:
                best.update(d=d, params=base["params"], pushes=base["pushes"], mode=base["mode"], course=dict(c))
            return d

        lo, hi = np.log1p(0.0), np.log1p(RANGES["eas"][1])
        bb = [(lo, hi), (lo, hi), (0.0, max(0.05, fe["dur"])), (np.log(0.01), np.log(2.0))]
        ramp = lambda x: {"e0": float(np.expm1(x[0])), "e1": float(np.expm1(x[1])), "t0": float(x[2]), "dur": float(np.exp(x[3]))}
        x0 = np.array([np.log1p(e_fit), np.log1p(e_fit), 0.5 * fe["dur"], np.log(0.1)])
        nm(lambda x: course_cost(ramp(x)), x0, np.array([0.7, 0.7, 0.3 * fe["dur"], 1.0]), bb, RAMP_FEV)
        d_ramp = best["d"]
        c0 = best["course"] or ramp(x0)
        bb = [(np.log(MOD_RATE[0]), np.log(MOD_RATE[1])), (0.0, MOD_DEPTH)]
        mod = lambda x: {**c0, "rate": float(np.exp(x[0])), "depth": float(x[1])}
        nm(lambda x: course_cost(mod(x)), np.array([np.log(3.0), 10.0]), np.array([0.8, 15.0]), bb, MOD_FEV)
        d_mod = best["d"]
    # the best fit's per-feature misses
    until = float(min(secs, 1.5 * fe["dur"] + 0.5))
    courses = {"eas": course_of(best["course"], until)} if best["course"] else None
    sf = sim_features(r, best["params"], best["pushes"], secs, courses)
    d, terms = F.distance(fe, sf, scale, detail=True) if sf else (10.0, {})
    return {"id": e["id"], "rec": e["rec"], "coarse": coarse, "d": d, "d_const": d_const, "d_ramp": d_ramp, "d_mod": d_mod,
            "params": best["params"], "pushes": best["pushes"], "course": best["course"], "until": until,
            "mode": best["mode"], "terms": terms, "evals": nev, "sim": {k: sf[k] for k in F.NAMES + ["dur", "voiced"]} if sf else None}


def nm(cost, x0, step, bb, maxfev):
    """Nelder–Mead from x0 with an initial simplex of the given steps, inside the bounds."""
    lo, hi = [b[0] for b in bb], [b[1] for b in bb]
    x0 = np.clip(x0, lo, hi)
    sim0 = [x0] + [np.clip(x0 + np.eye(len(x0))[i] * step, lo, hi) for i in range(len(x0))]
    minimize(cost, x0, method="Nelder-Mead", bounds=bb,
             options={"maxfev": maxfev, "initial_simplex": np.array(sim0), "xatol": 1e-3, "fatol": 1e-3})


def main():
    events = json.load(open(sys.argv[1]))
    global COURSES
    lib = json.load(open(sys.argv[2]))
    KEYS.extend(lib[0]["params"])
    outp, tmp = sys.argv[3], sys.argv[4]
    opts = dict(a.split("=", 1) for a in sys.argv[5:])
    COURSES = opts.get("courses") == "1"
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
        cs = f" (constant {f['d_const']:.2f}, ramp {f['d_ramp']:.2f})" if f["d_ramp"] is not None else ""
        print(f"{i + 1}/{len(events_fit)} {e['id']}: coarse {f['coarse']:.2f} fit {f['d']:.2f}{cs} (real NN {f['nn']:.2f}) {f['evals']} renders {time.time() - t0:.0f}s", file=sys.stderr, flush=True)
        json.dump({"scale": scale.tolist(), "names": F.NAMES, "nn": nn, "fits": list(done.values())}, open(outp, "w"), indent=1)
    r.close()


if __name__ == "__main__":
    main()
