"""Cut the included recordings of a manifest into events and measure them.

    python3 -I events.py <manifest.json> <wav dir> <out events.json> [<clip dir>]

`wav dir` holds each recording decoded to mono 48 kHz float WAV under the manifest's file name
with a .wav extension (`ffmpeg -i in -ac 1 -ar 48000 -c:a pcm_f32le out.wav`). Events are cut by
`features.segment`; at most `MAX_EVENTS` per recording (the loudest; `MAX_EVENTS_LONG` for the
long archive compilation), each with 0.1 s of context either side. The manifest may exclude
single events by index (`skip_events`).
"""
import json, os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import numpy as np
from scipy.io import wavfile
import features as F

MAX_EVENTS = 3
MAX_EVENTS_LONG = 12
MIN_DUR = 0.08


def main():
    man = json.load(open(sys.argv[1]))
    wdir, outp = sys.argv[2], sys.argv[3]
    cdir = sys.argv[4] if len(sys.argv) > 4 else None
    events = []
    for m in man:
        if not m.get("include"):
            continue
        fs, x = wavfile.read(os.path.join(wdir, os.path.splitext(m["file"])[0] + ".wav"))
        assert fs == F.FS
        x = x.astype(np.float64)
        m["duration"] = round(len(x) / fs, 2)
        segs = F.segment(x, fs)
        pad = int(0.1 * fs)
        cand = []
        for i, (a, b) in enumerate(segs):
            if i in m.get("skip_events", []):
                continue
            clip = x[max(0, a - pad) : b + pad]
            f = F.features(clip, fs)
            if f is None or f["dur"] < MIN_DUR:
                continue
            level = float(np.sqrt(np.mean(x[a:b] ** 2)))
            cand.append((level, i, a, b, f, clip))
        cap = MAX_EVENTS_LONG if m["duration"] > 60 else MAX_EVENTS
        if len(cand) > cap:
            if m["duration"] > 60:  # spread over the compilation: every k-th event by time
                idx = np.linspace(0, len(cand) - 1, cap).round().astype(int)
                cand = [cand[j] for j in idx]
            else:
                cand = sorted(cand, key=lambda c: -c[0])[:cap]
        for level, i, a, b, f, clip in sorted(cand, key=lambda c: c[1]):
            eid = f"{m['id']}.{i}"
            events.append({"id": eid, "rec": m["id"], "index": i, "start": a / fs, "end": b / fs,
                           "feat": {k: (float(v) if isinstance(v, (np.floating, float)) else v) for k, v in f.items()}})
            if cdir:
                os.makedirs(cdir, exist_ok=True)
                wavfile.write(os.path.join(cdir, eid + ".wav"), fs, clip.astype(np.float32))
    json.dump(events, open(outp, "w"), indent=1)
    print(len(events), "events from", len({e["rec"] for e in events}), "recordings")


if __name__ == "__main__":
    main()
