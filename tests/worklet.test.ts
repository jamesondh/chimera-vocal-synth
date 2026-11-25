/**
 * The AudioWorklet processor itself (src/worklet/processor.ts), driven as the browser drives it:
 * messages on its port, then process() a quantum at a time. Its chords are several engines in one
 * wasm instance; the engine's own calls are tested in chord-abi.test.ts.
 */
import { expect, test } from "bun:test";
import { wasmModule, workletProcessor } from "./harness.ts";
import type { FromWorklet, TelemetryMessage, ToWorklet } from "../src/worklet/protocol.ts";

const FS = 48000;
const Processor = await workletProcessor(FS);

function worklet() {
  const p = new Processor({ processorOptions: { module: wasmModule } });
  const out = new Float32Array(128);
  const send = (m: ToWorklet) => p.port.onmessage({ data: m });
  /** Render `sec` seconds; the output's peak, and the telemetry of the span's end. */
  const run = (sec: number) => {
    let peak = 0;
    for (let i = 0; i < Math.round((sec * FS) / 128); i++) {
      p.process([], [[out]]);
      for (const s of out) peak = Math.max(peak, Math.abs(s));
    }
    const posted = p.posted as FromWorklet[];
    const tele = posted.filter((m): m is TelemetryMessage => m.type === "telemetry").at(-1)!;
    const error = posted.find((m) => m.type === "error");
    if (error) throw new Error(JSON.stringify(error));
    return { peak, tele };
  };
  return { send, run };
}
const cents = (f: number, hz: number) => 1200 * Math.log2(f / hz);

test("a note goes to its own voice, and one that starts from silence starts on its pitch", () => {
  const w = worklet();
  w.send({ type: "voices", count: 3 });
  w.send({ type: "source", values: { f0: 110 } });
  w.run(0.1);
  // two octaves up: by the source's glide this takes the voice tens of milliseconds
  w.send({ type: "source", values: { f0: 440 }, voice: 1, snap: true });
  w.send({ type: "gate", level: 1, voice: 1 });
  const { tele } = w.run(0.02);
  expect(Math.abs(cents(tele.voices![1].f0, 440))).toBeLessThan(30);
  expect(Math.abs(cents(tele.voices![0].f0, 110))).toBeLessThan(30);
  expect(Math.abs(cents(tele.voices![2].f0, 110))).toBeLessThan(30);
  // only voice 1 breathes, and it is heard: the output is the sum of the voices, not voice 0
  const late = w.run(0.4);
  expect(late.tele.voices!.map((v) => v.voicing > 0.3)).toEqual([false, true, false]);
  expect(late.peak).toBeGreaterThan(0.05);
});

test("the sum of a loud chord goes through the limiter; one voice does not", () => {
  const loud = (count: number) => {
    const w = worklet();
    w.send({ type: "voices", count });
    w.send({ type: "source", values: { f0: 140, pressure: 16 } });
    for (let v = 0; v < count; v++) w.send({ type: "gate", level: 1, voice: v });
    w.run(0.5);
    return w.run(0.5).peak;
  };
  const one = loud(1), four = loud(4);
  // four voices in unison are four times one: over full scale with no limiter on the sum
  expect(4 * one).toBeGreaterThan(1.2);
  expect(four).toBeLessThan(1);
  expect(four).toBeGreaterThan(0.5);
});

test("a voice that comes back takes voice 0's clock, also one still ringing from before", () => {
  const w = worklet();
  w.send({ type: "voices", count: 3 });
  w.run(0.2);
  // under a creature the human clock of voice 0 stands still, and the voices taken away ring on
  w.send({ type: "creature", id: "songbird" });
  w.send({ type: "voices", count: 1 });
  w.run(0.3);
  w.send({ type: "creature", id: "human" });
  w.send({ type: "voices", count: 3 });
  const clockOf = (voice: number) => {
    w.send({ type: "show", voice });
    return w.run(0.05).tele.t.clock;
  };
  const a = clockOf(0), b = clockOf(2), c = clockOf(0);
  // one timeline: each reading 50 ms after the one before, whichever voice it is of
  expect(b - a).toBeCloseTo(0.05, 2);
  expect(c - b).toBeCloseTo(0.05, 2);
});
