/**
 * What the worklet's chords use of the engine's audio-thread ABI (src/worklet/processor.ts): a
 * voice's pitch, a joining voice's clock, and the limiter on the sum (the plugin's).
 */
import { expect, test } from "bun:test";
import { wasmModule } from "./harness.ts";

type Fn = (...a: number[]) => number;
const FS = 48000;
const x = new WebAssembly.Instance(wasmModule, {}).exports as unknown as { memory: WebAssembly.Memory } & Record<string, Fn>;

/** a voice in this one instance, breathing at f0 */
function voiceAt(f0: number): number {
  const h = x.engine_new(FS);
  const v = new Float64Array(x.memory.buffer, x.engine_values(h), 32).fill(NaN);
  v[0] = f0;
  x.engine_set_source(h);
  x.engine_set_gate(h, 1);
  return h;
}

test("each voice in one instance is its own: its own pitch, as its telemetry says", () => {
  const hs = [110, 138.6, 164.8].map(voiceAt);
  for (let i = 0; i < FS / 128; i++) for (const h of hs) x.engine_render(h, 128);
  for (const [i, f0] of [110, 138.6, 164.8].entries()) {
    const p = new Float64Array(x.memory.buffer, x.engine_pitch(hs[i]), 2);
    const tele = new Float64Array(x.memory.buffer, x.engine_telemetry(hs[i]), 16);
    expect([p[0], p[1]]).toEqual([tele[3], tele[4]]);
    expect(Math.abs(1200 * Math.log2(p[0] / f0))).toBeLessThan(30);
    expect(p[1]).toBeGreaterThan(0.5);
  }
});

test("a voice that joins takes another's clock", () => {
  const a = voiceAt(110), b = voiceAt(110);
  for (let i = 0; i < 100; i++) x.engine_render(a, 128);
  x.engine_set_clock(b, x.engine_clock(a));
  expect(x.engine_clock(b)).toBe(x.engine_clock(a));
});

test("the limiter turns a loud sum down below full scale and leaves a quiet one as it is", () => {
  const m = x.mix_new(FS);
  const n = 128;
  const buf = () => new Float32Array(x.memory.buffer, x.mix_buffer(m), n);
  let peak = 0;
  for (let i = 0; i < 400; i++) {
    buf().set(Float32Array.from({ length: n }, (_, k) => 2.4 * Math.sin((2 * Math.PI * 220 * (i * n + k)) / FS)));
    x.mix_limit(m, n);
    if (i > 10) for (const s of buf()) peak = Math.max(peak, Math.abs(s));
  }
  expect(peak).toBeLessThan(1);
  expect(peak).toBeGreaterThan(0.6);
  x.mix_reset(m);
  const quiet = Float32Array.from({ length: n }, (_, k) => 0.3 * Math.sin((2 * Math.PI * 220 * k) / FS));
  buf().set(quiet);
  x.mix_limit(m, n);
  expect(Array.from(buf())).toEqual(Array.from(quiet));
});
