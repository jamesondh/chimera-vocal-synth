"""Arabic pronunciation of the engine, judged by Whisper (large-v3-turbo, multilingual, on MLX).

usage: arabic_judge.py <outdir> [--against earlier/arabic.json] [--control]

Two tests, on renders by `chimera render arabic` in the male and female voices:

1. Minimal pairs (arabic-pairs.tsv), forced choice. Each word of a pair (سيف/صيف, قلب/كلب, عمل/أمل, …)
   is rendered alone, and Whisper scores both spellings against the audio: log P(text | audio)
   from its decoder, teacher-forced. A trial is heard as intended when the rendered word's spelling
   scores higher. That still carries the language model's preference for the commoner word, so
   the pair is also scored without it: the contrast is carried when the audio moves the
   preference the right way, [s(A|a) − s(B|a)] − [s(A|b) − s(B|b)] > 0, where the prior cancels.
2. Sentences (arabic-sentences.tsv), free transcription (greedy, language ar): character error
   rate after removing vowel marks and folding hamza seats, alif maqṣūra and tāʾ marbūṭa, at most
   100 % per sentence. The "held" sentences are reported but never decide.

--control runs the same tests on the macOS Arabic voice (say -v Majed): a real speech synthesizer,
so its scores show what the judge can hear, the ceiling for these words.

With --against it compares the trial accuracy (both bodies): BETTER from +MARGIN points, WORSE
past −SLACK, otherwise NO CLEAR CHANGE.

Set up once with tools/asr/setup-arabic.sh (Apple silicon: an MLX env in ~/.cache/chimera-asr-ar;
the model, about 1.6 GB, downloads into the Hugging Face cache on first use).
"""

import json
import os
import re
import subprocess
import sys
import wave
from concurrent.futures import ThreadPoolExecutor

import mlx.core as mx
import numpy as np
import mlx_whisper
from mlx_whisper.audio import N_FRAMES, N_SAMPLES, log_mel_spectrogram, pad_or_trim
from mlx_whisper.load_models import load_model
from mlx_whisper.tokenizer import get_tokenizer

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
MODEL = "mlx-community/whisper-large-v3-turbo"
BODIES = ("male", "female")
MARGIN, SLACK = 3.0, 3.0

MARKS = re.compile("[ؐ-ًؚ-ٰٟۖ-ۭ࣓-ࣿـ]")


def norm(text):
    t = MARKS.sub("", text)
    t = re.sub("[أإآٱ]", "ا", t).replace("ى", "ي").replace("ة", "ه").replace("ؤ", "و").replace("ئ", "ي")
    t = re.sub(r"[^ء-ي ]+", " ", t)
    return " ".join(t.split())


def edits(ref, hyp):
    d = list(range(len(hyp) + 1))
    for i, r in enumerate(ref, 1):
        prev, d[0] = d[0], i
        for j, h in enumerate(hyp, 1):
            prev, d[j] = d[j], min(d[j] + 1, d[j - 1] + 1, prev + (r != h))
    return d[len(hyp)]


def load16k(path):
    """Mono float32 at 16 kHz from a 16-bit PCM WAV (band-limited FFT resampling)."""
    with wave.open(path) as w:
        ch, rate = w.getnchannels(), w.getframerate()
        x = np.frombuffer(w.readframes(w.getnframes()), dtype="<i2").astype(np.float64) / 32768.0
    x = x.reshape(-1, ch).mean(axis=1)
    if rate == 16000:
        return x.astype(np.float32)
    n = int(round(len(x) * 16000 / rate))
    return (np.fft.irfft(np.fft.rfft(x)[: n // 2 + 1], n) * (n / len(x))).astype(np.float32)


class Judge:
    def __init__(self):
        self.model = load_model(MODEL)
        self.tok = get_tokenizer(self.model.is_multilingual, num_languages=self.model.num_languages, language="ar", task="transcribe")
        self.prefix = list(self.tok.sot_sequence_including_notimestamps)

    def features(self, audio):
        mel = log_mel_spectrogram(audio, n_mels=self.model.dims.n_mels, padding=N_SAMPLES)
        mel = pad_or_trim(mel[:N_FRAMES], N_FRAMES, axis=-2)
        return self.model.embed_audio(mel[None])

    def score(self, feats, text):
        """log P(text | audio), teacher-forced, end of text included."""
        toks = self.tok.encode(" " + text)
        seq = self.prefix + toks + [self.tok.eot]
        logits = self.model.logits(mx.array([seq[:-1]]), feats).astype(mx.float32)
        lp = logits - mx.logsumexp(logits, axis=-1, keepdims=True)
        k = len(self.prefix)
        pos = mx.arange(k - 1, len(seq) - 1)
        return float(lp[0, pos, mx.array(seq[k:])].sum())

    def transcribe(self, audio):
        r = mlx_whisper.transcribe(audio, path_or_hf_repo=MODEL, language="ar", temperature=0.0,
                                   condition_on_previous_text=False, verbose=None)
        return r["text"].strip()


def render_engine(text, body, stem):
    exe = os.path.join(REPO, "target/release/chimera")
    subprocess.run([exe, "render", "arabic", stem, text, body], check=True, capture_output=True)


def render_control(text, body, stem):
    subprocess.run(["say", "-v", "Majed", "--file-format=WAVE", "--data-format=LEI16@16000", "-o", stem + ".wav", text], check=True)


def main():
    args = sys.argv[1:]
    against = None
    if "--against" in args:
        i = args.index("--against")
        against = args[i + 1]
        del args[i : i + 2]
    control = "--control" in args
    args = [a for a in args if a != "--control"]
    out = args[0]
    os.makedirs(out, exist_ok=True)
    bodies = ("majed",) if control else BODIES
    render = render_control if control else render_engine
    pairs = [line.rstrip("\n").split("\t") for line in open(os.path.join(HERE, "arabic-pairs.tsv")) if line.strip()]
    sentences = [line.rstrip("\n").split("\t") for line in open(os.path.join(HERE, "arabic-sentences.tsv")) if line.strip()]

    jobs = []
    for i, (_, va, vb, _, _) in enumerate(pairs):
        for body in bodies:
            jobs += [(va, body, os.path.join(out, f"pair{i + 1:02d}a-{body}")), (vb, body, os.path.join(out, f"pair{i + 1:02d}b-{body}"))]
    for i, (_, text, _) in enumerate(sentences):
        for body in bodies:
            jobs.append((text, body, os.path.join(out, f"sent{i + 1:02d}-{body}")))
    with ThreadPoolExecutor(os.cpu_count()) as pool:
        list(pool.map(lambda j: render(*j), jobs))

    judge = Judge()
    trials, carried, rows = {}, {}, []
    for i, (contrast, va, vb, ta, tb) in enumerate(pairs):
        for body in bodies:
            fa = judge.features(load16k(os.path.join(out, f"pair{i + 1:02d}a-{body}.wav")))
            fb = judge.features(load16k(os.path.join(out, f"pair{i + 1:02d}b-{body}.wav")))
            s = {("a", "A"): judge.score(fa, ta), ("a", "B"): judge.score(fa, tb), ("b", "A"): judge.score(fb, ta), ("b", "B"): judge.score(fb, tb)}
            ok_a = s["a", "A"] > s["a", "B"]
            ok_b = s["b", "B"] > s["b", "A"]
            d = (s["a", "A"] - s["a", "B"]) - (s["b", "A"] - s["b", "B"])
            for key in (contrast, f"body:{body}", "all"):
                t = trials.setdefault(key, [0, 0])
                t[0] += ok_a + ok_b
                t[1] += 2
                c = carried.setdefault(key, [0, 0])
                c[0] += d > 0
                c[1] += 1
            rows.append({"contrast": contrast, "a": ta, "b": tb, "body": body, "heardA": ok_a, "heardB": ok_b, "carried": d > 0, "d": round(d, 2)})
            mark = lambda ok: "ok" if ok else "--"
            print(f"  {contrast:18s} {body:6s} {ta:>8s} {mark(ok_a)}  {tb:>8s} {mark(ok_b)}  contrast {d:+6.2f}")

    sent_rows, cer = [], {}
    for i, (kind, text, part) in enumerate(sentences):
        for body in bodies:
            heard = judge.transcribe(load16k(os.path.join(out, f"sent{i + 1:02d}-{body}.wav")))
            ref, hyp = norm(text), norm(heard)
            # capped at the sentence's length: a Whisper repetition loop counts as all wrong, once
            e = min(edits(ref, hyp), len(ref))
            for key in (part, f"{part}-{body}"):
                c = cer.setdefault(key, [0, 0])
                c[0] += e
                c[1] += len(ref)
            sent_rows.append({"kind": kind, "set": part, "body": body, "text": text, "heard": heard, "errors": e, "chars": len(ref)})
            if part == "train":
                print(f"  {part:5s} {body:6s} {e:2d}/{len(ref):2d}  {ref}  →  {hyp}")

    pct = lambda d: {k: round(100.0 * a / b, 1) for k, (a, b) in sorted(d.items())}
    report = {"heard": pct(trials), "carried": pct(carried), "cer": pct(cer), "pairs": rows, "sentences": sent_rows}
    with open(os.path.join(out, "arabic.json"), "w") as f:
        json.dump(report, f, indent=1, ensure_ascii=False)
    print("heard as intended (% of trials): " + ", ".join(f"{k} {v}" for k, v in report["heard"].items()))
    print("contrast carried (% of pairs):   " + ", ".join(f"{k} {v}" for k, v in report["carried"].items()))
    print("character error rate (%):        " + ", ".join(f"{k} {v}" for k, v in report["cer"].items()))
    if against:
        before = json.load(open(against))
        old, new = before["heard"]["all"], report["heard"]["all"]
        verdict = "BETTER" if new >= old + MARGIN else "WORSE" if new < old - SLACK else "NO CLEAR CHANGE"
        print(f"arabic verdict against {against}: {verdict} (heard as intended {old} → {new} %; "
              f"training CER {before['cer'].get('train')} → {report['cer'].get('train')} %)")


main()
