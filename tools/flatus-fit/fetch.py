"""Download the manifest's included recordings and decode them to mono 48 kHz float WAV.

    python3 -I fetch.py <manifest.json> <raw dir> <wav dir>

The downloads are untrusted data: keep both directories outside the repository (recordings are
never committed). Needs ffmpeg. Freesound files are its public HQ previews (128 kb/s MP3).
"""
import json, os, subprocess, sys, time, urllib.request

UA = {"User-Agent": "chimera-flatus-validation/1.0 (research; see docs/flatulence.md)"}


def main():
    man = json.load(open(sys.argv[1]))
    raw, wav = sys.argv[2], sys.argv[3]
    os.makedirs(raw, exist_ok=True)
    os.makedirs(wav, exist_ok=True)
    for m in man:
        if not m.get("include"):
            continue
        p = os.path.join(raw, m["file"])
        if not os.path.exists(p):
            time.sleep(3)  # Wikimedia rate-limits bursts
            data = urllib.request.urlopen(urllib.request.Request(m["file_url"], headers=UA), timeout=120).read()
            open(p, "wb").write(data)
        w = os.path.join(wav, os.path.splitext(m["file"])[0] + ".wav")
        if not os.path.exists(w):
            subprocess.run(["ffmpeg", "-nostdin", "-loglevel", "error", "-y", "-i", p, "-ac", "1", "-ar", "48000", "-c:a", "pcm_f32le", w], check=True)
        print(m["id"], m["title"])


if __name__ == "__main__":
    main()
