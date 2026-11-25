/**
 * The voices of a chord, one for each finger (docs/plugin.md, "Voices"): in the plugin as its last
 * poll found them, in the web app as the worklet's last telemetry did. Empty while one voice plays.
 * The pitch lines and readouts read it each frame (mutated in place: no re-render per message).
 */

/** What is known of one voice. */
export interface LiveVoice {
  /** the note it sounds (−1: none) */
  note: number;
  /** the syllable it sings (−1: none) */
  sung: number;
  /** the presses it has taken */
  serial: number;
  /** the pitch its folds make now (Hz), and how voiced they are (0 to 1) */
  f0: number;
  voicing: number;
}

export const liveVoices: {
  voices: LiveVoice[];
  shown: number;
  clock: number;
  polls: number;
  /** in the web app, the note each voice sounds (−1: none), as Sing and Play gave them */
  notes: number[];
} = { voices: [], shown: 0, clock: 0, polls: 0, notes: [] };

/** Voicing above this is a voice heard singing, as the readouts count it. */
export const VOICED = 0.3;

/** The pitches the voices of a chord sing now (Hz), lowest first; null while one voice plays. */
export function voicedPitches(): number[] | null {
  if (!liveVoices.voices.length) return null;
  return liveVoices.voices
    .filter((v) => v.voicing > VOICED && v.f0 > 0)
    .map((v) => v.f0)
    .sort((a, b) => a - b);
}

/** A worklet's telemetry of a chord's voices (the web app): each voice's pitch, and the one shown. */
export function followWorkletVoices(pitches: { f0: number; voicing: number }[] | undefined, shown: number, clock: number): void {
  if (!pitches) {
    if (liveVoices.voices.length) liveVoices.voices = [];
    return;
  }
  liveVoices.voices = pitches.map((p, v) => ({ note: liveVoices.notes[v] ?? -1, sung: -1, serial: 0, f0: p.f0, voicing: p.voicing }));
  liveVoices.shown = shown;
  liveVoices.clock = clock;
  liveVoices.polls++;
}

// the end-to-end checks read the voices (tools/e2e.ts, tools/plugin-e2e.ts)
if (typeof window !== "undefined") (window as unknown as { __chimeraVoices: typeof liveVoices }).__chimeraVoices = liveVoices;
