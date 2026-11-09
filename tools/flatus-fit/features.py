"""Acoustic features of flatulence events, and the distance between two events.

Used on real recordings and on the flatulence creature's renders alike (docs/flatulence.md,
"Validation against recordings"). Every feature is independent of the recording's level and
gain, because the recordings are uncalibrated: durations and envelope shape, pitch, periodicity,
subharmonics and the spectrum's shape between 62.5 Hz and 8 kHz.

Run with `python3 -I` on downloaded data. numpy and scipy only.
"""

import numpy as np
from scipy import signal

FS = 48000
HOP = 0.01  # analysis hop (s)
FMIN, FMAX = 40.0, 1000.0
APERIODIC = 0.3  # YIN aperiodicity below which a frame counts as periodic (as docs/flatulence.md)
BANDS = [62.5 * 2**k for k in range(8)]  # octave band edges, 62.5 Hz .. 8 kHz


def bandpass(x, fs=FS):
    """50 Hz–10 kHz: drops DC, handling rumble and the codecs' top end."""
    sos = signal.butter(4, [50.0, 10000.0], btype="band", fs=fs, output="sos")
    return signal.sosfiltfilt(sos, x)


def envelope_db(x, fs=FS, hop=HOP, win=None):
    """RMS level per hop (dB, arbitrary reference)."""
    h = int(round(hop * fs))
    w = int(round((win or hop) * fs))
    n = max(0, (len(x) - w) // h + 1)
    if n == 0:
        return np.zeros(0)
    idx = np.arange(w)[None, :] + h * np.arange(n)[:, None]
    e = np.sqrt(np.mean(x[idx] ** 2, axis=1) + 1e-20)
    return 20 * np.log10(e)


def segment(x, fs=FS, gap=0.2, min_len=0.05, floor_margin=15.0, below_peak=35.0, event_snr=15.0, event_below=25.0):
    """Split a recording into events: runs of 10 ms frames louder than both the noise floor
    (10th percentile) + `floor_margin` dB and the loudest frame − `below_peak` dB, joined across
    gaps shorter than `gap` s, at least `min_len` s long, whose loudest frame is at least
    `event_snr` dB above the floor and within `event_below` dB of the recording's loudest (fainter
    ones are tails, rumbles or noise). Returns [(start, end)] in samples."""
    xb = bandpass(x, fs)
    env = envelope_db(xb, fs)
    if len(env) == 0:
        return []
    floor = np.percentile(env, 10)
    thr = max(floor + floor_margin, env.max() - below_peak)
    on = env > thr
    runs = []
    i = 0
    while i < len(on):
        if on[i]:
            j = i
            while j < len(on) and on[j]:
                j += 1
            runs.append([i, j])
            i = j
        else:
            i += 1
    merged = []
    for r in runs:
        if merged and (r[0] - merged[-1][1]) * HOP < gap:
            merged[-1][1] = r[1]
        else:
            merged.append(r)
    h = int(HOP * fs)
    top = env.max()
    return [(a * h, b * h) for a, b in merged
            if (b - a) * HOP >= min_len and env[a:b].max() >= max(floor + event_snr, top - event_below)]


def yin(x, fs, fmin=FMIN, fmax=FMAX, win=0.05, hop=HOP):
    """Frame-wise YIN (de Cheveigné & Kawahara 2002): f0 (Hz), aperiodicity (the cumulative-mean
    normalised difference at the chosen lag) and the normalised autocorrelation there."""
    W = int(win * fs)
    H = int(round(hop * fs))
    tmax = int(fs / fmin) + 1
    tmin = max(2, int(fs / fmax))
    L = W + tmax
    n = (len(x) - L) // H + 1
    if n <= 0:
        return np.zeros(0), np.ones(0), np.zeros(0)
    idx = np.arange(L)[None, :] + H * np.arange(n)[:, None]
    fr = x[idx]
    nfft = 1 << int(np.ceil(np.log2(L + W)))
    A = np.fft.rfft(fr, nfft)
    B = np.fft.rfft(fr[:, :W], nfft)
    c = np.fft.irfft(A * np.conj(B), nfft)[:, : tmax + 1]  # c[τ] = Σ_j x_j x_{j+τ}
    sq = np.cumsum(np.concatenate([np.zeros((n, 1)), fr**2], axis=1), axis=1)
    E1 = sq[:, W][:, None]
    taus = np.arange(tmax + 1)
    E2 = sq[:, taus + W] - sq[:, taus]
    d = np.maximum(E1 + E2 - 2 * c, 0)
    cm = np.cumsum(d[:, 1:], axis=1)
    dn = np.ones_like(d)
    dn[:, 1:] = d[:, 1:] * taus[1:] / np.maximum(cm, 1e-30)
    f0 = np.zeros(n)
    ap = np.ones(n)
    rr = np.zeros(n)
    for k in range(n):
        row = dn[k]
        t = -1
        cand = np.where(row[tmin:tmax] < 0.15)[0]
        if len(cand):
            t = cand[0] + tmin
            while t + 1 < tmax and row[t + 1] < row[t]:
                t += 1
        else:
            t = tmin + int(np.argmin(row[tmin:tmax]))
        if 1 <= t < tmax:
            a, b, cc = row[t - 1], row[t], row[t + 1]
            den = a - 2 * b + cc
            off = 0.5 * (a - cc) / den if den > 1e-12 else 0.0
            off = float(np.clip(off, -0.5, 0.5))
        else:
            off = 0.0
        f0[k] = fs / (t + off)
        ap[k] = row[t]
        rr[k] = c[k, t] / np.sqrt(max(E1[k, 0] * E2[k, t], 1e-30))
    return f0, ap, rr


def ltas_bands(x, fs):
    """Octave-band levels (dB re the total over 62.5 Hz–8 kHz), centroid (Hz) and the spectrum's
    tilt (dB per octave, regression over the bands)."""
    nper = min(4096, len(x))
    f, p = signal.welch(x, fs, nperseg=nper, noverlap=nper * 3 // 4, scaling="spectrum")
    lv = []
    for lo, hi in zip(BANDS[:-1], BANDS[1:]):
        m = (f >= lo) & (f < hi)
        lv.append(p[m].sum() + 1e-30)
    lv = np.array(lv)
    tot = lv.sum()
    rel = 10 * np.log10(lv / tot)
    m = (f >= BANDS[0]) & (f < BANDS[-1])
    cen = float((f[m] * p[m]).sum() / max(p[m].sum(), 1e-30))
    oct_ = np.log2(np.sqrt(np.array(BANDS[:-1]) * np.array(BANDS[1:])))
    tilt = float(np.polyfit(oct_, rel, 1)[0])
    return rel, cen, tilt


def subharmonic_ratio(x, fs, f0s, starts, win=0.05):
    """Subharmonic-to-harmonic ratio (dB; after Sun 2002): spectral amplitude at the half
    harmonics (k − ½)·f0 against the harmonics k·f0, k = 1…5, over periodic frames. Period
    doubling and alternating pulses raise it."""
    W = int(win * fs)
    nfft = 1 << int(np.ceil(np.log2(4 * W)))
    w = np.hanning(W)
    fr = np.fft.rfftfreq(nfft, 1 / fs)
    vals = []
    for f0, s in zip(f0s, starts):
        seg = x[s : s + W]
        if len(seg) < W:
            continue
        a = np.abs(np.fft.rfft(seg * w, nfft))
        def amp(fq):
            i = int(round(fq / fr[1]))
            return a[max(0, i - 2) : i + 3].max() if i + 3 < len(a) else 0.0
        h = sum(amp(k * f0) for k in range(1, 6))
        sh = sum(amp((k - 0.5) * f0) for k in range(1, 6))
        if h > 0:
            vals.append(20 * np.log10(max(sh, 1e-12) / h))
    return float(np.median(vals)) if vals else -30.0


NAMES = [
    "log_dur", "log_attack", "log_release", "sustain", "bursts",
    "f0", "f0_range", "f0_slope", "f0_jitter",
    "periodic", "hnr", "shr",
    "centroid", "tilt",
    "b88", "b177", "b354", "b707", "b1414", "b2828", "b5657",
]

# weights: temporal, pitch, periodicity, spectrum groups of comparable total weight
WEIGHTS = {
    "log_dur": 1.0, "log_attack": 0.4, "log_release": 0.4, "sustain": 0.4, "bursts": 0.8,
    "f0": 1.2, "f0_range": 0.5, "f0_slope": 0.5, "f0_jitter": 0.5,
    "periodic": 1.0, "hnr": 0.8, "shr": 0.5,
    "centroid": 1.0, "tilt": 0.6,
    "b88": 0.2, "b177": 0.2, "b354": 0.2, "b707": 0.2, "b1414": 0.2, "b2828": 0.2, "b5657": 0.2,
}
PITCH = {"f0", "f0_range", "f0_slope", "f0_jitter", "shr"}


def features(x, fs=FS):
    """Features of one event (a clip holding one event and a little silence around it).
    Returns a dict; `voiced` is the fraction of loud frames that are periodic."""
    x = np.asarray(x, dtype=np.float64)
    if len(x) < int(0.2 * fs) or np.max(np.abs(x)) < 1e-5:
        return None  # silent
    xb = bandpass(x, fs)
    env = envelope_db(xb, fs)
    if len(env) < 3:
        return None
    floor = np.percentile(env, 5)
    pk = env.max()
    thr = max(pk - 30.0, floor + 6.0)
    loud = np.where(env > thr)[0]
    a, b = loud[0], loud[-1] + 1
    dur = (b - a) * HOP
    ipk = int(np.argmax(env))
    near = np.where(env[a:b] > pk - 6.0)[0]
    attack = (near[0]) * HOP
    release = (b - a - 1 - near[-1]) * HOP
    sustain = float(np.mean(env[a:b] > pk - 10.0))
    # bursts: peaks of a 5 ms envelope (smoothed over 15 ms) rising 6 dB from the dips around them
    e5 = envelope_db(xb, fs, hop=0.005, win=0.015)
    seg5 = e5[2 * a : 2 * b]
    if len(seg5) >= 3:
        pks, _ = signal.find_peaks(seg5, prominence=6.0, height=pk - 25.0, distance=6)
        nb = max(1, len(pks))
    else:
        nb = 1
    # pitch and periodicity on a 16 kHz copy
    fs2 = 16000
    x2 = signal.resample_poly(xb, fs2, fs)
    f0, ap, rr = yin(x2, fs2)
    m = min(len(f0), len(env))
    frames = np.arange(a, min(b, m))
    per = frames[(ap[frames] < APERIODIC) & np.isfinite(f0[frames]) & (f0[frames] > 0)]
    voiced = len(per) / max(1, len(frames))
    rc = np.clip(np.nan_to_num(rr[frames], nan=0.0), 1e-3, 0.9999)
    hnr = float(np.median(np.clip(10 * np.log10(rc / (1 - rc)), -10, 30))) if len(frames) else -10.0
    out = {"dur": dur, "voiced": voiced, "n_periodic": len(per)}
    lf = np.log2(f0[per]) if len(per) else np.zeros(0)
    if len(lf) >= 3:
        # octave-jump guard: a frame more than 0.7 oct from the median is dropped
        keep = np.abs(lf - np.median(lf)) < 0.7
        lf, tp = lf[keep], per[keep] * HOP
        pk_frames = per[keep]
    if len(lf) >= 3:
        f0_med = float(np.median(lf))
        f0_range = float(np.percentile(lf, 90) - np.percentile(lf, 10))
        f0_slope = float(np.polyfit(tp, lf, 1)[0]) if np.ptp(tp) > 0 else 0.0
        if len(lf) >= 5:
            sm = signal.medfilt(lf, 5)
            f0_jit = float(np.std((lf - sm)[2:-2]) * 12)
        else:
            f0_jit = 0.0
        shr = subharmonic_ratio(x2, fs2, 2 ** lf, (pk_frames * int(HOP * fs2)).tolist())
    else:
        f0_med, f0_range, f0_slope, f0_jit, shr = np.log2(150.0), 0.0, 0.0, 0.0, -30.0
    seg = xb[a * int(HOP * fs) : b * int(HOP * fs)]
    rel, cen, tilt = ltas_bands(seg, fs)
    out.update({
        "log_dur": np.log10(dur),
        "log_attack": np.log10(attack + 0.01),
        "log_release": np.log10(release + 0.01),
        "sustain": sustain,
        "bursts": np.log2(nb),
        "f0": f0_med, "f0_range": f0_range, "f0_slope": float(np.clip(f0_slope, -4, 4)), "f0_jitter": f0_jit,
        "periodic": voiced, "hnr": hnr, "shr": shr,
        "centroid": np.log2(cen), "tilt": tilt,
    })
    for k, v in zip(NAMES[-7:], rel):
        out[k] = float(v)
    out["vec"] = [float(out[k]) for k in NAMES]
    return out


def pitch_weight(fa, fb):
    """f0 terms count fully when both events are mostly periodic and not at all when either is
    mostly noise (their periodicity terms carry that difference)."""
    v = min(fa["periodic"], fb["periodic"])
    return float(np.clip((v - 0.1) / 0.2, 0.0, 1.0))


def distance(fa, fb, scale, detail=False):
    """Normalised distance: RMS over the features of (a − b)/scale, weighted. `scale` is each
    feature's spread across the real events (`feature_scale`)."""
    pw = pitch_weight(fa, fb)
    num = 0.0
    den = 0.0
    terms = {}
    for i, k in enumerate(NAMES):
        w = WEIGHTS[k] * (pw if k in PITCH else 1.0)
        z = (fa["vec"][i] - fb["vec"][i]) / scale[i]
        terms[k] = (float(z), float(w))  # signed miss (a − b in spreads) and its weight
        num += w * z * z
        den += WEIGHTS[k]
    d = float(np.sqrt(num / den))
    return (d, terms) if detail else d


def feature_scale(feats):
    """Per-feature spread across the real events: the standard deviation, the pitch features'
    over the events that are mostly periodic (elsewhere they are undefined), floored at a small
    absolute value."""
    V = np.array([f["vec"] for f in feats])
    sd = V.std(axis=0)
    per = np.array([f["periodic"] >= 0.3 for f in feats])
    if per.sum() >= 5:
        for i, k in enumerate(NAMES):
            if k in PITCH:
                sd[i] = V[per, i].std()
    return np.maximum(sd, 1e-3)
