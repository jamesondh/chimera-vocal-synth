/**
 * CHIMERA AudioWorklet processor: the Rust engine (wasm) on the audio thread.
 *
 * Messages map to engine calls as they arrive (between render quanta, on this thread), so their
 * order is the order they take effect. process() renders, measures load and posts fixed-size
 * telemetry (~60 Hz). If the engine traps (a bug), the processor outputs silence from then on and
 * reports once; the main thread may build a new node, which restarts playback state.
 *
 * A chord is several engines in this one instance, one voice for each note (docs/plugin.md,
 * "Voices" and "The web app's chords"). Voice 0 is the one voice there always is:
 * speech, tunes, the beat and the creatures are its own. The body, the pose and the dials go to
 * every voice; a note goes to its own. With more than one voice the sum goes through the plugin's
 * limiter (`chimera::voice::chord_limiter`); with one, the output is voice 0's as before.
 */
import { writeGestures, writeSteps, fromUtf8, utf8, CONTROL_KEYS } from "../wasm/wire.ts";
import { TELE_HEADER, TELE_LABELS, decodeTelemetry, telemetryLength } from "./telemetry.ts";
import type { ToWorklet, FromWorklet, TelemetryMessage } from "./protocol.ts";
import type { BodyMix, CreatureTelemetry } from "../wasm/types.ts";

// Minimal AudioWorkletGlobalScope typings
declare const sampleRate: number;
declare function registerProcessor(name: string, ctor: unknown): void;
declare class AudioWorkletProcessor {
  readonly port: MessagePort;
  constructor(options?: unknown);
}

type Fn = (...a: number[]) => number;
type Exports = { memory: WebAssembly.Memory } & Record<string, Fn>;

const ARTIC_KEYS = CONTROL_KEYS.slice(0, 17);
const SOURCE_KEYS = ["f0", "glide", "pressure", "breathiness", "tension", "fry", "roughness", "vibratoDepth", "vibratoRate", "drift", "voiced", "brightness"];
/** gestures taken from one schedule message */
const MAX_GESTURES = 4000;
/** a chord's voices at most (src/ui/voiceAlloc.ts) */
const MAX_VOICES = 8;
/** voices taken away ring on for this long (s), so their release is heard */
const VOICE_TAIL = 1;

/** What the engine says about the creature it switched to (crates/chimera-wasm/src/lib.rs). */
interface CreatureInfo {
  id: string;
  params: string[];
  calls: string[];
  tubes: { name: string; n: number }[];
  sources: string[];
}

class ChimeraProcessor extends AudioWorkletProcessor {
  private readonly x: Exports;
  /** voice 0 */
  private readonly h: number;
  /** every voice made so far (made when the count first asks for it); engines[0] is h */
  private readonly engines: number[] = [];
  /** voices that play, and voices rendered (those plus the ones taken away, for VOICE_TAIL) */
  private count = 1;
  private rendered = 1;
  private tailLeft = 0;
  /** voices 0 to sang − 1 have had a note since they joined: only those have anything to ring on with */
  private sang = 1;
  /** the voice the telemetry is of */
  private shown = 0;
  /** the chord limiter, whose buffer takes the sum of the voices (made with the second voice) */
  private mix = 0;
  /** what every voice plays from, for a voice made later: the body, the pose and the dials */
  private shared: { body: BodyMix | null; artic: Record<string, number>; tipFree: boolean | undefined; source: Record<string, number> } = { body: null, artic: {}, tipFree: undefined, source: {} };
  private failed = false;
  private telemetryOn = true;
  private blocks = 0;
  private load = 0;
  private creature: CreatureInfo | null = null;

  constructor(options: { processorOptions: { module: WebAssembly.Module } }) {
    super();
    this.x = new WebAssembly.Instance(options.processorOptions.module, {}).exports as unknown as Exports;
    this.h = this.x.engine_new(sampleRate);
    this.engines.push(this.h);
    this.port.onmessage = (e: MessageEvent<ToWorklet>) => {
      if (this.failed) return;
      try {
        this.handle(e.data);
      } catch (err) {
        this.fail(err);
      }
    };
    this.postReady();
  }

  private post(msg: FromWorklet): void {
    this.port.postMessage(msg);
  }

  private fail(err: unknown): void {
    if (this.failed) return;
    this.failed = true;
    this.post({ type: "error", message: `engine stopped: ${String(err)}` });
  }

  private get mem(): ArrayBuffer {
    return this.x.memory.buffer;
  }

  private body(): BodyMix {
    return { female: this.x.engine_body_female(this.h), child: this.x.engine_body_child(this.h) };
  }

  private postReady(): void {
    const x = this.x, h = this.h;
    this.post({ type: "ready", sampleRate, body: this.body(), nPh: x.engine_n_ph(h), nOr: x.engine_n_or(h), creature: this.creature?.id ?? "human" });
  }

  /** A JSON call on this instance's engine (non-real-time: creature switches). */
  private engineJson<T>(fn: string, args: Record<string, unknown>): T {
    const x = this.x;
    const bytes = utf8(JSON.stringify(args));
    const p = x.chimera_alloc(bytes.length);
    new Uint8Array(this.mem, p, bytes.length).set(bytes);
    const status = x[fn](this.h, p, bytes.length);
    x.chimera_free(p, bytes.length);
    const v = JSON.parse(fromUtf8(new Uint8Array(this.mem, x.chimera_result_ptr(), x.chimera_result_len())));
    if (status !== 0) throw new Error(v.error);
    return v as T;
  }

  /** An engine's value area, cleared to NaN ("unchanged"). */
  private values(h = this.h): Float64Array {
    return new Float64Array(this.mem, this.x.engine_values(h), 32).fill(NaN);
  }

  /** A voice's engine (a message for a voice not playing goes nowhere). */
  private voice(v: number | undefined): number | null {
    const i = v ?? 0;
    return i >= 0 && i < this.rendered ? this.engines[i] : null;
  }

  private setArtic(h: number, values: Record<string, unknown>, tipFree: boolean | undefined): void {
    const v = this.values(h);
    for (const [k, val] of Object.entries(values)) {
      const i = ARTIC_KEYS.indexOf(k as never);
      if (i >= 0 && typeof val === "number") v[i] = val;
    }
    this.x.engine_set_artic(h, tipFree === undefined ? 2 : tipFree ? 1 : 0);
  }

  private setSource(h: number, values: Record<string, unknown>): void {
    const v = this.values(h);
    for (const [k, val] of Object.entries(values)) {
      const i = SOURCE_KEYS.indexOf(k);
      if (i >= 0) v[i] = typeof val === "boolean" ? (val ? 1 : 0) : (val as number);
    }
    this.x.engine_set_source(h);
  }

  /**
   * How many voices play. A voice that joins is made with the body, the pose and the dials the
   * others have, and takes voice 0's clock, so the telemetry of any voice is on one timeline.
   * Voices taken away ring on for VOICE_TAIL (their notes were released first).
   */
  private setCount(n: number): void {
    const x = this.x;
    n = Math.max(1, Math.min(MAX_VOICES, Math.round(n)));
    while (this.engines.length < n) {
      const h = x.engine_new(sampleRate);
      const s = this.shared;
      if (s.body) x.engine_set_body(h, s.body.female, s.body.child);
      this.setArtic(h, s.artic, s.tipFree);
      this.setSource(h, s.source);
      this.engines.push(h);
    }
    if (n > 1 && !this.mix) this.mix = x.mix_new(sampleRate);
    // (a voice still ringing has run in step with voice 0: under a creature, where voice 0's
    // human clock stands still, none rings, see below)
    const clock = x.engine_clock(this.h);
    for (let v = this.rendered; v < n; v++) x.engine_set_clock(this.engines[v], clock);
    const fewer = n < this.count;
    if (fewer) {
      this.tailLeft = Math.round(VOICE_TAIL * sampleRate);
      // a voice that only passed by has no note to end (the chip steps 2, 3, 4, 1: two voices
      // ring on, not four)
      this.rendered = Math.min(this.rendered, this.sang);
    }
    // a voice still ringing from before keeps ringing until its tail ends
    this.rendered = Math.max(n, this.rendered);
    this.count = n;
    if (this.shown >= n) this.shown = 0;
    // nothing to ring on; or a creature plays, and the human voices are not heard under it
    // (voice 0's stops at once, and so do these)
    if (this.creature || (fewer && this.rendered === n)) this.endTails();
  }

  /** The voices taken away stop now. */
  private endTails(): void {
    this.rendered = this.count;
    this.sang = Math.min(this.sang, this.count);
    this.tailLeft = 0;
    if (this.count === 1 && this.mix) this.x.mix_reset(this.mix);
  }

  private handle(m: ToWorklet): void {
    const x = this.x, h = this.h;
    switch (m.type) {
      case "body":
        this.shared.body = m.body;
        // a new body starts at its own pitch: the old one's is not a voice made later's either
        delete this.shared.source.f0;
        for (const e of this.engines) x.engine_set_body(e, m.body.female, m.body.child);
        this.postReady();
        break;
      case "artic": {
        Object.assign(this.shared.artic, m.values);
        if (m.tipFree !== undefined) this.shared.tipFree = m.tipFree;
        for (const e of this.engines) this.setArtic(e, m.values, m.tipFree);
        break;
      }
      case "source": {
        if (m.voice === undefined) {
          Object.assign(this.shared.source, m.values);
          for (const e of this.engines) this.setSource(e, m.values);
          break;
        }
        const e = this.voice(m.voice);
        if (e === null) break;
        this.sang = Math.max(this.sang, m.voice + 1);
        this.setSource(e, m.values);
        if (m.snap) x.engine_snap_pitch(e);
        break;
      }
      case "gate": {
        const e = this.voice(m.voice);
        if (e !== null) x.engine_set_gate(e, m.level);
        break;
      }
      case "voices":
        this.setCount(m.count);
        break;
      case "show":
        if (m.voice >= 0 && m.voice < this.count) this.shown = m.voice;
        break;
      case "telemetry":
        this.telemetryOn = m.enabled;
        break;
      case "schedule": {
        // planning happened on the main thread; the engine shifts the gestures onto its clock
        const e = this.voice(m.voice);
        if (e === null) break;
        this.sang = Math.max(this.sang, (m.voice ?? 0) + 1);
        const n = writeGestures(this.mem, x.engine_records(e), m.gestures, MAX_GESTURES);
        const off = x.engine_schedule(e, n, m.lead ?? 0.03, m.replace ? 1 : 0);
        if (m.segments.length) this.post({ type: "planned", id: m.id, t0: off, end: m.end + off, segments: m.segments.map((s) => ({ ...s, t0: s.t0 + off, t1: s.t1 + off })) });
        break;
      }
      case "stopSpeech":
        for (const e of this.engines) x.engine_stop_speech(e);
        break;
      case "release":
        // a tag is one note's, on whichever voice sings it
        for (const e of this.engines) x.engine_release(e, m.tag);
        break;
      case "beat": {
        const n = m.steps ? writeSteps(this.mem, x.engine_records(h), m.steps) : 0;
        x.engine_set_beat(h, m.steps ? 1 : 0, n, m.bpm ?? NaN, m.swing ?? NaN, m.playing === undefined ? 2 : m.playing ? 1 : 0);
        break;
      }
      case "beatHit":
        writeSteps(this.mem, x.engine_records(h), [{ id: m.id, note: m.note, accent: m.accent }]);
        x.engine_beat_hit(h);
        break;
      case "creature":
        this.creature = m.id === "human" ? (x.engine_set_human(h), null) : this.engineJson<CreatureInfo>("engine_set_creature", { id: m.id, params: m.params ?? {} });
        if (this.creature) this.endTails();
        this.postReady();
        break;
      case "cparams": {
        const c = this.creature;
        if (!c) break;
        for (const [k, val] of Object.entries(m.values)) {
          const i = c.params.indexOf(k);
          if (i >= 0) x.engine_creature_param(h, i, val);
        }
        break;
      }
      case "call": {
        const i = this.creature ? this.creature.calls.indexOf(m.id) : -1;
        if (i >= 0) x.engine_play_call(h, i, m.pitch ?? NaN, m.amp ?? NaN);
        break;
      }
      case "stopCall":
        x.engine_stop_call(h);
        break;
    }
  }

  process(_inputs: Float32Array[][], outputs: Float32Array[][]): boolean {
    const out = outputs[0];
    if (!out || out.length === 0) return true;
    const ch0 = out[0];
    if (this.failed) {
      for (const ch of out) ch.fill(0);
      return true;
    }
    const t0 = Date.now();
    try {
      if (this.rendered === 1) {
        const p = this.x.engine_render(this.h, ch0.length);
        ch0.set(new Float32Array(this.mem, p, ch0.length));
      } else this.renderChord(ch0);
    } catch (err) {
      ch0.fill(0);
      this.fail(err);
    }
    for (let c = 1; c < out.length; c++) out[c].set(ch0);
    const dt = Date.now() - t0;
    const budget = (ch0.length / sampleRate) * 1000;
    this.load += 0.05 * (dt / budget - this.load);
    if (this.telemetryOn && !this.failed && ++this.blocks % 6 === 0) {
      try {
        if (this.creature) this.post({ type: "ctelemetry", t: this.creatureTelemetry(this.creature), load: this.load });
        else this.post(this.humanTelemetry());
      } catch (err) {
        this.fail(err);
      }
    }
    return true;
  }

  /** Several voices: their sum, through the chord limiter; voices taken away stop after their tail. */
  private renderChord(out: Float32Array): void {
    const x = this.x, n = out.length;
    // (rendering allocates nothing, so the memory doesn't grow under these views)
    const sum = new Float32Array(this.mem, x.mix_buffer(this.mix), n);
    sum.fill(0);
    for (let v = 0; v < this.rendered; v++) {
      const part = new Float32Array(this.mem, x.engine_render(this.engines[v], n), n);
      for (let i = 0; i < n; i++) sum[i] += part[i];
    }
    x.mix_limit(this.mix, n);
    out.set(sum);
    if (this.rendered > this.count && (this.tailLeft -= n) <= 0) this.endTails();
  }

  /** The human voice's telemetry block → the message the UI reads (of the voice shown). */
  private humanTelemetry(): TelemetryMessage {
    const x = this.x, h = this.engines[this.shown] ?? this.h;
    const p = x.engine_telemetry(h);
    const block = new Float64Array(this.mem, p, telemetryLength(new Float64Array(this.mem, p, TELE_HEADER)));
    const { t, nPh, nOr } = decodeTelemetry(block, new Uint8Array(this.mem, x.engine_tele_labels(h), TELE_LABELS));
    return {
      type: "telemetry",
      t,
      load: this.load,
      body: this.body(),
      nPh,
      nOr,
      beatStep: x.engine_beat_step(this.h),
      ...(this.count > 1 ? { voices: this.pitches(), shown: this.shown } : {}),
    };
  }

  /** Each voice's pitch now (Hz) and voicing. */
  private pitches(): { f0: number; voicing: number }[] {
    const out: { f0: number; voicing: number }[] = [];
    for (let v = 0; v < this.count; v++) {
      const p = new Float64Array(this.mem, this.x.engine_pitch(this.engines[v]), 2);
      out.push({ f0: p[0], voicing: p[1] });
    }
    return out;
  }

  /** A creature's telemetry block → the message the UI reads (names from the switch's info). */
  private creatureTelemetry(c: CreatureInfo): CreatureTelemetry {
    const x = this.x, h = this.h;
    const len = 4 + c.params.length + c.tubes.reduce((s, t) => s + 2 * t.n, 0) + c.sources.length;
    const b = new Float64Array(this.mem, x.engine_creature_telemetry(h), len);
    const [gate, outRms, clock, call] = b;
    let o = 4;
    const params: Record<string, number> = {};
    for (const k of c.params) params[k] = b[o++];
    const tubes: CreatureTelemetry["tubes"] = {};
    for (const t of c.tubes) {
      const areas = Float32Array.from(b.subarray(o, (o += t.n)));
      const rms = Float32Array.from(b.subarray(o, (o += t.n)));
      tubes[t.name] = { areas, rms };
    }
    const source: Record<string, number> = {};
    for (const k of c.sources) source[k] = b[o++];
    return { kind: "creature", id: c.id, params, gate, outRms, clock, tubes, source, call: call >= 0 ? c.calls[call] : null };
  }
}

registerProcessor("chimera", ChimeraProcessor);
