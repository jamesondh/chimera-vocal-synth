/**
 * The spectrum of a recorded track at a moment, as the live waterfall's AnalyserNode measures its
 * output (a Blackman window over the last `n` samples, magnitude in dB re full scale, unsmoothed),
 * so a show's waterfall can be drawn at any playhead, the same every time.
 */

/** dB per bin (n/2 of them) of the `n` samples before `end` (zeros before the track). */
export function spectrumDb(x: Float32Array, end: number, n = 2048): Float32Array {
  const re = new Float64Array(n), im = new Float64Array(n);
  const a0 = 0.42, a1 = 0.5, a2 = 0.08;
  for (let i = 0; i < n; i++) {
    const k = end - n + i;
    const w = a0 - a1 * Math.cos((2 * Math.PI * i) / n) + a2 * Math.cos((4 * Math.PI * i) / n);
    re[i] = k >= 0 && k < x.length ? x[k] * w : 0;
  }
  fft(re, im);
  const out = new Float32Array(n / 2);
  for (let k = 0; k < n / 2; k++) out[k] = 20 * Math.log10(Math.hypot(re[k], im[k]) / n + 1e-12);
  return out;
}

/** In-place radix-2 FFT (n a power of two). */
function fft(re: Float64Array, im: Float64Array): void {
  const n = re.length;
  for (let i = 1, j = 0; i < n; i++) {
    let bit = n >> 1;
    for (; j & bit; bit >>= 1) j ^= bit;
    j ^= bit;
    if (i < j) {
      [re[i], re[j]] = [re[j], re[i]];
      [im[i], im[j]] = [im[j], im[i]];
    }
  }
  for (let len = 2; len <= n; len <<= 1) {
    const ang = (-2 * Math.PI) / len;
    const wr = Math.cos(ang), wi = Math.sin(ang);
    for (let i = 0; i < n; i += len) {
      let cr = 1, ci = 0;
      for (let k = 0; k < len / 2; k++) {
        const a = i + k, b = a + len / 2;
        const tr = re[b] * cr - im[b] * ci, ti = re[b] * ci + im[b] * cr;
        re[b] = re[a] - tr;
        im[b] = im[a] - ti;
        re[a] += tr;
        im[a] += ti;
        const nr = cr * wr - ci * wi;
        ci = cr * wi + ci * wr;
        cr = nr;
      }
    }
  }
}
