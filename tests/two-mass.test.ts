/**
 * The two-mass vocal folds through the worklet's ABI: the source's "glottis" value (slot 12 of
 * engine_set_source) switches the fold model, and the folds sing on their own.
 */
import { expect, test } from "bun:test";
import { WasmVoice } from "./harness.ts";

function sing(glottis: "parametric" | "twoMass"): { out: Float32Array; f0: number; voicing: number } {
  const v = new WasmVoice(48000);
  v.setSource({ f0: 150, glottis });
  v.setGate(1);
  const out = v.render(48000);
  const { f0, voicing } = v.state();
  return { out, f0, voicing };
}

test("glottis: twoMass sings with self-oscillating folds, and parametric stays the default", () => {
  const folds = sing("twoMass");
  const pulse = sing("parametric");
  const tail = folds.out.subarray(24000);
  const rms = Math.sqrt(tail.reduce((s, x) => s + x * x, 0) / tail.length);
  expect(tail.every(Number.isFinite)).toBe(true);
  expect(rms).toBeGreaterThan(0.01);
  expect(folds.voicing).toBeGreaterThan(0.5);
  expect(Math.abs(folds.f0 - 150)).toBeLessThan(5);
  // a different source: not the same samples as the drawn pulse
  expect(folds.out.subarray(24000, 25000)).not.toEqual(pulse.out.subarray(24000, 25000));
  // and naming nothing keeps the drawn pulse, sample for sample
  const v = new WasmVoice(48000);
  v.setSource({ f0: 150 });
  v.setGate(1);
  expect(v.render(48000)).toEqual(pulse.out);
});
