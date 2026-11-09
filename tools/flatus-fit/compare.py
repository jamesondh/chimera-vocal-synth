"""Spectrograms of recorded events beside the renders of their best fits (a picture to check the
numbers by eye; nothing here feeds the fit).

    python3 -I compare.py <fit.json> <clip dir> <out.png> <tmpdir> [id …]
"""
import json, os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from scipy import signal
from scipy.io import wavfile
from sim import Renderer


def spec(ax, x, fs, title):
    f, t, S = signal.spectrogram(x, fs, nperseg=1024, noverlap=896)
    S = 10 * np.log10(S + 1e-20)
    m = f < 4000
    ax.pcolormesh(t, f[m], S[m], vmin=S.max() - 60, vmax=S.max(), shading="auto", cmap="magma")
    ax.set_title(title, fontsize=7)
    ax.tick_params(labelsize=6)


def main():
    fit = json.load(open(sys.argv[1]))
    cdir, outp, tmp = sys.argv[2], sys.argv[3], sys.argv[4]
    fits = {f["id"]: f for f in fit["fits"]}
    ids = sys.argv[5:] or list(fits)
    r = Renderer(tmp)
    fig, axs = plt.subplots(len(ids), 2, figsize=(9, 1.9 * len(ids)), squeeze=False)
    for row, i in zip(axs, ids):
        f = fits[i]
        fs, x = wavfile.read(os.path.join(cdir, i + ".wav"))
        spec(row[0], x.astype(float), fs, f"{i} (recording)")
        y = r.render(f["params"], f["pushes"], max(3.0, len(x) / fs * 2.5))
        spec(row[1], np.concatenate([np.zeros(4800), y]), 48000, f"fit d={f['d']:.2f}")
    plt.tight_layout()
    plt.savefig(outp, dpi=70)
    r.close()


if __name__ == "__main__":
    main()
