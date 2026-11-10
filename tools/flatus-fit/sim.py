"""Render the flatulence creature through `examples/flatus_render` (one persistent process)."""

import os
import subprocess
import numpy as np

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
BIN = os.environ.get("FLATUS_RENDER") or os.path.join(ROOT, "target", "release", "examples", "flatus_render")

# the creature's physical controls and their ranges (crates/chimera/src/creatures/flatus.rs)
RANGES = {
    "push": (0.0, 80.0),
    "tone": (2.0, 150.0),  # models before the two sphincters (commit c258ac2 and earlier)
    "eas": (0.0, 250.0),
    "ias": (0.0, 80.0),
    "gas": (10.0, 300.0),
    "spread": (0.0, 1.0),
    "size": (0.6, 1.25),
    "mucus": (0.0, 3.0),
}


class Renderer:
    def __init__(self, tmpdir, extra=None):
        os.makedirs(tmpdir, exist_ok=True)
        self.tmp = os.path.join(tmpdir, f"r{os.getpid()}.f32")
        self.extra = extra or {}
        self.p = subprocess.Popen([BIN], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)

    def render(self, params, pushes=None, secs=8.0, courses=None):
        """params: {name: value}; pushes: [(on, off)] in seconds (None: held throughout);
        courses: {name: [(t, value), …]}, a control's course over time (it replaces the value)."""
        args = [self.tmp, f"{secs:.3f}"]
        courses = courses or {}
        for k, v in {**self.extra, **params}.items():
            lo, hi = RANGES.get(k, (-1e9, 1e9))
            if k in courses:
                args.append(f"{k}=" + ",".join(f"{t:.4f}:{min(hi, max(lo, x)):.5g}" for t, x in courses[k]))
            else:
                args.append(f"{k}={min(hi, max(lo, v)):.6g}")
        if pushes:
            args.append("pushes=" + ",".join(f"{a:.4f}:{b:.4f}" for a, b in pushes))
        self.p.stdin.write(" ".join(args) + "\n")
        self.p.stdin.flush()
        line = self.p.stdout.readline()
        if not line.startswith("ok"):
            raise RuntimeError("renderer: " + line)
        return np.fromfile(self.tmp, dtype="<f4").astype(np.float64)

    def close(self):
        self.p.stdin.close()
        self.p.wait()
