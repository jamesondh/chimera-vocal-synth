"""Render a grid over the creature's physical controls once (push held until the gas is gone) and
store each render's features: the coarse stage of the fit, shared by every recording.

    python3 -I library.py <out.json> <tmpdir> [<grid overrides as JSON>]

The overrides replace or add grid dimensions, e.g. '{"mucus": [0, 1.5, 3], "size": [0.8, 1.2]}';
an override of "tone" takes the grid of the model before the two sphincters.
"""
import itertools, json, os, sys, time
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import numpy as np
import features as F
from sim import Renderer

GRID = {
    "push": [4, 8, 15, 25, 40, 60, 80],
    "eas": [0, 5, 15, 35, 80, 150],
    "ias": [0, 37, 70],
    "gas": [15, 50, 150, 300],
    "spread": [0.0, 0.4, 1.0],
    "size": [0.8, 1.2],
}
# the grid of the model before the two sphincters (one "tone"), for its libraries
GRID_TONE = {
    "push": [3, 6, 10, 15, 22, 30, 45, 60, 80],
    "tone": [2, 4, 8, 15, 25, 40, 70, 110],
    "gas": [10, 25, 60, 150, 300],
    "spread": [0.0, 0.4, 1.0],
    "size": [0.65, 1.0, 1.25],
}
PAD = int(0.1 * F.FS)


def continent(p):
    """Renders that are shut whatever the rectal recoil (≤ 36 cmH2O at 300 mL) are skipped: a speed
    filter only, with the closing pressures of crates/chimera/src/creatures/flatus.rs."""
    lim = p["push"] + 40
    if "tone" in p:  # the model before the two sphincters
        return p["tone"] > lim
    nadir = 1 - np.exp(-p["gas"] / 27.0)  # the rectoanal reflex's depth (RAIR_VC)
    upper = p["ias"] * (1 - nadir) + 0.15 * 67 + 17 / 32 * p["eas"]  # IAS, cushions, EAS share
    return p["eas"] > lim or upper > lim


def sim_features(r, params, pushes=None, secs=8.0, courses=None):
    x = r.render(params, pushes, secs, courses)
    return F.features(np.concatenate([np.zeros(PAD), x, np.zeros(PAD)]))


def main():
    out, tmp = sys.argv[1], sys.argv[2]
    over = json.loads(sys.argv[3]) if len(sys.argv) > 3 else {}
    grid = {**(GRID_TONE if "tone" in over else GRID), **over}
    r = Renderer(tmp)
    lib = []
    keys = list(grid)
    t0 = time.time()
    for i, vals in enumerate(itertools.product(*grid.values())):
        p = dict(zip(keys, vals))
        if continent(p):
            continue
        f = sim_features(r, p)
        if f is None:
            continue
        lib.append({"params": p, "feat": {k: (v if not isinstance(v, np.floating) else float(v)) for k, v in f.items()}})
        if len(lib) % 100 == 0:
            print(len(lib), i, f"{time.time() - t0:.0f}s", file=sys.stderr, flush=True)
    json.dump(lib, open(out, "w"))
    r.close()


if __name__ == "__main__":
    main()
