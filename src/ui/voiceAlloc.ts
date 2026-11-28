/**
 * Which of the web app's voices takes a key: the plugin's rules (docs/plugin.md, "Voices"), less
 * MPE, which the web app doesn't read. Pure: Sing and Play ask it, then make the sound.
 *
 * 1. the voice that has this key already (a repeat);
 * 2. or the voice free for the longest;
 * 3. or, with every voice busy, the voice whose note started longest ago. It takes the key as one
 *    voice takes a second key: legato, and back to the older key if that is still down when the
 *    new one lifts. A voice back on its older key is as old as that key's note again.
 *
 * With one voice every key goes to voice 0, and its keys are the one voice's keys as before.
 */
export const MAX_VOICES = 8;

export class VoiceAlloc {
  /** voices that play */
  count = 1;
  /** keys down on each voice, oldest first */
  readonly keys: number[][] = Array.from({ length: MAX_VOICES }, () => []);
  /** each voice's age: the press of its newest note, or when it became free */
  private readonly order: number[] = new Array(MAX_VOICES).fill(0);
  private readonly keyVoice = new Map<number, number>();
  private readonly keyOrder = new Map<number, number>();
  private serial = 0;

  busy(v: number): boolean {
    return this.keys[v].length > 0;
  }

  /** The voice that holds this key, if one does. */
  voiceOf(midi: number): number | undefined {
    return this.keyVoice.get(midi);
  }

  /** The voice the rules give this key (nothing changes until `press`). */
  pick(midi: number): number {
    const held = this.keyVoice.get(midi);
    if (held !== undefined) return held;
    let free = -1, oldest = 0;
    for (let v = 0; v < this.count; v++) {
      if (!this.busy(v) && (free < 0 || this.order[v] < this.order[free])) free = v;
      if (this.order[v] < this.order[oldest]) oldest = v;
    }
    return free >= 0 ? free : oldest;
  }

  /** Voice v takes the key (one down on another voice is lifted from that one first, by the caller). */
  press(midi: number, v: number): void {
    this.keyVoice.set(midi, v);
    this.keys[v] = this.keys[v].filter((k) => k !== midi).concat(midi);
    this.order[v] = ++this.serial;
    this.keyOrder.set(midi, this.serial);
  }

  /** The key lifts: its voice, and the key that voice goes back to (its newest still down). */
  lift(midi: number): { voice: number; back: number | null } | null {
    const v = this.keyVoice.get(midi);
    if (v === undefined) return null;
    this.keyVoice.delete(midi);
    this.keyOrder.delete(midi);
    this.keys[v] = this.keys[v].filter((k) => k !== midi);
    const back = this.keys[v].length ? this.keys[v][this.keys[v].length - 1] : null;
    this.order[v] = back === null ? ++this.serial : (this.keyOrder.get(back) ?? this.order[v]);
    return { voice: v, back };
  }

  /** The voice whose note is newest among those with a key down (null: no key down). */
  newest(): number | null {
    let best: number | null = null;
    for (let v = 0; v < this.count; v++) if (this.busy(v) && (best === null || this.order[v] > this.order[best])) best = v;
    return best;
  }

  /** No key down anywhere; the ages start again. */
  clear(): void {
    for (const k of this.keys) k.length = 0;
    this.order.fill(0);
    this.keyVoice.clear();
    this.keyOrder.clear();
    this.serial = 0;
  }
}
