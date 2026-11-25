/**
 * The plugin's voices, as its last poll found them: one for each finger of a chord
 * (docs/plugin.md, "Voices"). Empty in the web app, which has one voice. The pitch lines and
 * readouts read it each frame (mutated in place: no re-render per poll).
 */

/** What the plugin says of one of its voices. */
export interface PluginVoice {
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

export const pluginVoices: { voices: PluginVoice[]; shown: number; clock: number; polls: number } = { voices: [], shown: 0, clock: 0, polls: 0 };

/** Voicing above this is a voice heard singing, as the readouts count it. */
export const VOICED = 0.3;

/** The pitches the plugin's voices sing now (Hz), lowest first; null in the web app. */
export function voicedPitches(): number[] | null {
  if (!pluginVoices.voices.length) return null;
  return pluginVoices.voices
    .filter((v) => v.voicing > VOICED && v.f0 > 20)
    .map((v) => v.f0)
    .sort((a, b) => a - b);
}
