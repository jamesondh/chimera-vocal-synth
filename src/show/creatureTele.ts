/**
 * A creature's recorded telemetry (`chimera render creature-show … --tele show.ctele`), replayed as
 * the live worklet's creature telemetry, so the Bestiary's drawing moves exactly as the engine did.
 *
 * File layout (little-endian; crates/chimera-cli/src/creature_show.rs `CreatureTeleRecorder`):
 * "CHCT", u32 version, sampleRate, hop, frames, frameLen, metaLen; metaLen bytes of JSON
 * `{id, params, calls, tubes: [{name, n}], sources, cues: [{at, end, name, hint}]}`; then `frames`
 * × `frameLen` f32. A frame is the wasm creature telemetry block: gate, outRms, clock, call index
 * (−1 none), params, per tube areas then rms, sources. Frame i is the state at sample i·hop.
 */
import type { CreatureTelemetry } from "../wasm/types.ts";

export const CTELE_VERSION = 1;
const HEADER_BYTES = 28;

export interface Cue {
  at: number;
  end: number;
  name: string;
  hint: string;
}

interface Meta {
  id: string;
  params: string[];
  calls: string[];
  tubes: { name: string; n: number }[];
  sources: string[];
  cues: Cue[];
}

export class CreatureTeleTrack {
  readonly sampleRate: number;
  readonly hop: number;
  readonly frames: number;
  readonly id: string;
  readonly cues: readonly Cue[];
  private readonly meta: Meta;
  private readonly len: number;
  private readonly data: Float32Array;

  constructor(buf: ArrayBuffer) {
    if (buf.byteLength < HEADER_BYTES) throw new Error("creature telemetry file is too short for its header");
    if (new TextDecoder().decode(new Uint8Array(buf, 0, 4)) !== "CHCT") throw new Error("not a CHIMERA creature telemetry file");
    const u = new DataView(buf);
    const at = (i: number) => u.getUint32(4 + 4 * i, true);
    const version = at(0);
    if (version !== CTELE_VERSION) throw new Error(`creature telemetry file version ${version}; this app reads ${CTELE_VERSION}`);
    this.sampleRate = at(1);
    this.hop = at(2);
    this.frames = at(3);
    this.len = at(4);
    const metaLen = at(5);
    if (metaLen % 4 || HEADER_BYTES + metaLen > buf.byteLength) throw new Error("creature telemetry file header is inconsistent");
    const m = JSON.parse(new TextDecoder().decode(new Uint8Array(buf, HEADER_BYTES, metaLen))) as Meta;
    this.meta = m;
    this.id = m.id;
    this.cues = m.cues ?? [];
    const want = 4 + m.params.length + m.tubes.reduce((s, t) => s + 2 * t.n, 0) + m.sources.length;
    if (this.len !== want || this.frames < 1 || this.hop < 1 || this.sampleRate < 1) throw new Error("creature telemetry file header is inconsistent");
    const bytes = HEADER_BYTES + metaLen + 4 * this.frames * this.len;
    if (buf.byteLength !== bytes) throw new Error(`creature telemetry file is ${buf.byteLength} bytes; its header says ${bytes}`);
    this.data = new Float32Array(buf, HEADER_BYTES + metaLen, this.frames * this.len);
  }

  /** seconds the recording covers */
  get duration(): number {
    return ((this.frames - 1) * this.hop) / this.sampleRate;
  }

  /** the frame at time t (s), clamped to the recording */
  indexAt(t: number): number {
    const i = Math.floor((t * this.sampleRate) / this.hop + 1e-6);
    return Math.min(this.frames - 1, Math.max(0, Number.isFinite(i) ? i : 0));
  }

  /** frame i as the live creature telemetry (arrays are views into the recording) */
  telemetry(i: number): CreatureTelemetry {
    const m = this.meta, d = this.data;
    let o = i * this.len;
    const [gate, outRms, clock, call] = d.subarray(o, (o += 4));
    const params: Record<string, number> = {};
    for (const k of m.params) params[k] = d[o++];
    const tubes: CreatureTelemetry["tubes"] = {};
    for (const t of m.tubes) {
      const areas = d.subarray(o, (o += t.n));
      const rms = d.subarray(o, (o += t.n));
      tubes[t.name] = { areas, rms };
    }
    const source: Record<string, number> = {};
    for (const k of m.sources) source[k] = d[o++];
    return { kind: "creature", id: m.id, params, gate, outRms, clock, tubes, source, call: call >= 0 ? (m.calls[call] ?? null) : null };
  }

  /** the cue playing at time t: the last one begun by then, while it lasts */
  cueAt(t: number): Cue | null {
    let c: Cue | null = null;
    for (const q of this.cues) if (q.at <= t + 1e-9) c = q;
    return c && t < c.end ? c : null;
  }
}
