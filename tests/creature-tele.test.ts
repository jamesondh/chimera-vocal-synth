/**
 * The creature show's recordings (`chimera render creature-show … --tele`): a file built by hand in
 * the CLI's layout decodes into the live view's creature telemetry, damaged files are refused, and
 * a real render read back here matches, frame for frame, what the worklet posts live when it plays
 * the same call (the round trip: engine → file → page, against engine → worklet → page).
 */
import { describe, expect, test } from "bun:test";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { CreatureTeleTrack } from "../src/show/creatureTele.ts";
import type { CreatureTelemetry } from "../src/wasm/types.ts";
import { initEngine, wasmModule } from "./harness.ts";

const META = {
  id: "toad",
  params: ["push", "size"],
  calls: ["croak", "trill"],
  tubes: [{ name: "a", n: 3 }, { name: "b", n: 2 }],
  sources: ["s0", "s1"],
  cues: [{ at: 0.05, end: 0.1, name: "Croak", hint: "A croak" }, { at: 0.1, end: 0.2, name: "Trill", hint: "" }],
};
const LEN = 4 + 2 + 2 * 5 + 2;

/** frame i: gate i/10, clock i·768/48000, call 1 from frame 3 (else −1), push 10 + i, tube a's areas i + k/10 */
function file(frames: number, o: { version?: number; cut?: number } = {}): ArrayBuffer {
  let meta = new TextEncoder().encode(JSON.stringify(META));
  meta = Uint8Array.from([...meta, ...Array((4 - (meta.length % 4)) % 4).fill(32)]);
  const buf = new ArrayBuffer(28 + meta.length + 4 * frames * LEN);
  const d = new DataView(buf);
  new Uint8Array(buf).set(new TextEncoder().encode("CHCT"));
  [o.version ?? 1, 48000, 768, frames, LEN, meta.length].forEach((x, i) => d.setUint32(4 + 4 * i, x, true));
  new Uint8Array(buf, 28).set(meta);
  const f = new Float32Array(buf, 28 + meta.length);
  for (let i = 0; i < frames; i++) {
    const r = f.subarray(i * LEN, (i + 1) * LEN);
    r.set([i / 10, 0.01, (i * 768) / 48000, i >= 3 ? 1 : -1, 10 + i, 1]);
    r.set([i, i + 0.1, i + 0.2], 6); // a's areas
    r[LEN - 1] = 0.5; // s1
  }
  return o.cut ? buf.slice(0, buf.byteLength - o.cut) : buf;
}

describe("CreatureTeleTrack", () => {
  test("reads the header and each frame as the live creature telemetry", () => {
    const t = new CreatureTeleTrack(file(10));
    expect([t.sampleRate, t.hop, t.frames, t.id]).toEqual([48000, 768, 10, "toad"]);
    const m = t.telemetry(4);
    expect(m.kind).toBe("creature");
    expect(m.id).toBe("toad");
    expect(m.gate).toBeCloseTo(0.4, 6);
    expect(m.clock).toBeCloseTo(0.064, 6);
    expect(m.call).toBe("trill");
    expect(t.telemetry(2).call).toBeNull();
    expect(m.params).toEqual({ push: 14, size: 1 });
    expect(Object.keys(m.tubes)).toEqual(["a", "b"]);
    expect([...m.tubes.a.areas].map((x) => +x.toFixed(4))).toEqual([4, 4.1, 4.2]);
    expect([m.tubes.a.rms.length, m.tubes.b.areas.length, m.tubes.b.rms.length]).toEqual([3, 2, 2]);
    expect(m.source).toEqual({ s0: 0, s1: 0.5 });
  });

  test("the frame and the cue at a time", () => {
    const t = new CreatureTeleTrack(file(10));
    expect(t.indexAt(0.016)).toBe(1);
    expect(t.indexAt(0.0159)).toBe(0);
    expect([t.indexAt(-1), t.indexAt(99), t.indexAt(NaN)]).toEqual([0, 9, 0]);
    expect(t.duration).toBeCloseTo(0.144, 6);
    expect(t.cueAt(0.01)).toBeNull();
    expect(t.cueAt(0.07)?.name).toBe("Croak");
    expect(t.cueAt(0.1)?.name).toBe("Trill");
    expect(t.cueAt(0.25)).toBeNull();
  });

  test("refuses a truncated file, another version or another format", () => {
    expect(() => new CreatureTeleTrack(file(10, { cut: 4 }))).toThrow(/bytes/);
    expect(() => new CreatureTeleTrack(file(10).slice(0, 20))).toThrow(/too short/);
    expect(() => new CreatureTeleTrack(file(10, { version: 2 }))).toThrow(/version 2/);
    const other = file(2);
    new Uint8Array(other).set(new TextEncoder().encode("CHTL"));
    expect(() => new CreatureTeleTrack(other)).toThrow(/not a CHIMERA creature/);
  });
});

describe("a creature show's render, read back", () => {
  test(
    "matches the worklet's live telemetry for the same call, frame by frame",
    async () => {
      const dir = mkdtempSync(join(tmpdir(), "creature-tele-"));
      writeFileSync(join(dir, "s.json"), JSON.stringify({ creature: "flatulence", tail: 0, events: [{ at: 0, call: "squeak" }] }));
      const r = Bun.spawnSync(["cargo", "run", "--quiet", "--release", "-p", "chimera-cli", "--", "render", "creature-show", join(dir, "s.json"), join(dir, "s"), "--tele", join(dir, "s.ctele")]);
      if (r.exitCode) throw new Error(`chimera render creature-show: ${r.stderr.toString()}`);
      const buf = readFileSync(join(dir, "s.ctele"));
      const track = new CreatureTeleTrack(buf.buffer.slice(buf.byteOffset, buf.byteOffset + buf.byteLength));
      expect(track.cues.map((c) => c.name)).toEqual(["Squeak"]);

      // the live path: the worklet's processor on the wasm engine, the same call at sample 0
      await initEngine();
      type Proc = { port: { onmessage: ((e: { data: unknown }) => void) | null }; process: (i: unknown, o: Float32Array[][]) => boolean };
      let Ctor: (new (o: unknown) => Proc) | null = null;
      const posted: { type: string; t?: CreatureTelemetry }[] = [];
      const g = globalThis as Record<string, unknown>;
      g.sampleRate = 48000;
      g.AudioWorkletProcessor = class {
        port = { onmessage: null as unknown, postMessage: (m: { type: string }) => posted.push(m) };
      };
      g.registerProcessor = (_name: string, ctor: typeof Ctor) => (Ctor = ctor);
      // @ts-expect-error the query makes bun evaluate the module again, so this file registers its own processor
      await import("../src/worklet/processor.ts?creature-tele");
      const p = new Ctor!({ processorOptions: { module: wasmModule } });
      p.port.onmessage!({ data: { type: "creature", id: "flatulence" } });
      p.port.onmessage!({ data: { type: "call", id: "squeak" } });
      posted.length = 0;
      // the worklet posts every 6 quanta of 128: frame k is the state at 768·k samples
      for (let i = 0; i < 6 * (track.frames - 1); i++) p.process([], [[new Float32Array(128)]]);
      const live = posted.filter((m) => m.type === "ctelemetry").map((m) => m.t!);
      expect(live.length).toBe(track.frames - 1);
      const close = (a: number, b: number, what: string) => {
        if (Math.abs(a - b) > 1e-5 * Math.max(1, Math.abs(b))) throw new Error(`${what}: file ${a}, live ${b}`);
      };
      let sounding = 0;
      live.forEach((l, k) => {
        const f = track.telemetry(k + 1);
        expect(f.call).toBe(l.call);
        for (const key of ["gate", "outRms", "clock"] as const) close(f[key], l[key], `frame ${k + 1} ${key}`);
        expect(Object.keys(f.params)).toEqual(Object.keys(l.params));
        for (const key in l.params) close(f.params[key], l.params[key], `frame ${k + 1} param ${key}`);
        expect(Object.keys(f.source)).toEqual(Object.keys(l.source));
        for (const key in l.source) close(f.source[key], l.source[key], `frame ${k + 1} source ${key}`);
        expect(Object.keys(f.tubes)).toEqual(Object.keys(l.tubes));
        for (const [name, tb] of Object.entries(l.tubes)) {
          expect([...f.tubes[name].areas]).toEqual([...tb.areas]);
          tb.rms.forEach((x, n) => close(f.tubes[name].rms[n], x, `frame ${k + 1} ${name} rms ${n}`));
        }
        sounding += +(l.outRms > 0.01); // the squeak sounds for about 0.2 s
      });
      expect(sounding).toBeGreaterThan(6);
    },
    { timeout: 600_000 },
  );
});
