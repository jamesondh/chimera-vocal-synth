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
  /** how many voices it renders now (those that play, and those still ringing) */
  const rendered = () => (p as unknown as { rendered: number }).rendered;
  return { send, run, rendered };
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

test("under a creature the voices taken away stop at once, and they come back on voice 0's clock", () => {
  const w = worklet();
  w.send({ type: "voices", count: 3 });
  w.send({ type: "source", values: { f0: 220 }, voice: 2 });
  w.run(0.2);
  // Sing to Bestiary: the creature is voice 0's, whose human voice stops at once and whose human
  // clock stands still. The others once rang on for their second: mixed into the creature, and
  // 0.3 s ahead of voice 0 when Sing came back before the second was over
  w.send({ type: "creature", id: "songbird" });
  w.send({ type: "voices", count: 1 });
  expect(w.rendered()).toBe(1);
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

test("only a voice that had a note rings on when it is taken away", () => {
  const w = worklet();
  // the chip, from 2 back to 1 by way of 3 and 4: voices 2 and 3 never sang
  w.send({ type: "voices", count: 2 });
  w.send({ type: "source", values: { f0: 220 }, voice: 1, snap: true });
  w.send({ type: "gate", level: 1, voice: 1 });
  w.run(0.1);
  w.send({ type: "gate", level: 0, voice: 1 });
  for (const count of [3, 4, 1]) w.send({ type: "voices", count });
  expect(w.rendered()).toBe(2);
  // for a second, then one voice
  w.run(0.9);
  expect(w.rendered()).toBe(2);
  w.run(0.2);
  expect(w.rendered()).toBe(1);
  // and with no note on any of them, nothing rings on at all
  for (const count of [3, 4, 1]) w.send({ type: "voices", count });
  expect(w.rendered()).toBe(1);
});

test("a voice made after a new body starts at that body's pitch, not at the old dial's", () => {
  const w = worklet();
  w.send({ type: "source", values: { f0: 100 } });
  w.send({ type: "body", body: { female: 0, child: 1 } });
  w.send({ type: "voices", count: 2 });
  w.send({ type: "gate", level: 1, voice: 0 });
  w.send({ type: "gate", level: 1, voice: 1 });
  const { tele } = w.run(0.3);
  expect(Math.abs(cents(tele.voices![1].f0, tele.voices![0].f0))).toBeLessThan(30);
  expect(tele.voices![0].f0).toBeGreaterThan(200);
});
