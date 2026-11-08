/**
 * A melody typed in solfège, the way it is often written down for Turkish songs:
 *
 *   do re mi_ | sol(i) sib la___ - fa#
 *
 * - a note is do re mi fa sol la si; "b" flattens it a semitone ("sib"), "#" sharpens it ("fa#")
 * - "(i)" puts it an octave up (ince), "(k)" an octave down (kalın); "(ii)" two octaves
 * - each note lasts one step; every "_" after it adds a step ("sol___" is four steps)
 * - "-" is a rest of one step, lengthened the same way ("-_")
 * - "|" bar lines, commas and line breaks are ignored
 *
 * Plain "do" is middle C (MIDI 60), the adult female register; a male voice sings an octave lower.
 * Pure: no engine calls, so it can be tested on its own.
 */

/** One step-timed event: a pitch (MIDI, may be fractional) or a rest (null). */
export interface MelodyNote {
  midi: number | null;
  steps: number;
}

export interface ParsedMelody {
  notes: MelodyNote[];
  /** the first token that isn't a note, rest or bar line, if any */
  error: string | null;
}

const DEGREES: Record<string, number> = { do: 0, re: 2, mi: 4, fa: 5, sol: 7, la: 9, si: 11 };
/** middle C */
const DO = 60;

// note name, accidental, octave mark, lengthening
const NOTE = /^(do|re|mi|fa|sol|la|si)(b|#)?(?:\((i+|k+)\))?(_*)$/;
const REST = /^-(_*)$/;

export function parseMelody(text: string): ParsedMelody {
  const notes: MelodyNote[] = [];
  const tokens = text
    .toLocaleLowerCase("tr")
    .replace(/[|,]/g, " ")
    .split(/\s+/)
    .filter(Boolean);
  for (const tok of tokens) {
    const rest = REST.exec(tok);
    if (rest) {
      notes.push({ midi: null, steps: 1 + rest[1].length });
      continue;
    }
    const m = NOTE.exec(tok);
    if (!m) return { notes, error: tok };
    const [, name, acc, oct, hold] = m;
    const shift = acc === "b" ? -1 : acc === "#" ? 1 : 0;
    const octave = oct ? (oct[0] === "i" ? oct.length : -oct.length) : 0;
    notes.push({ midi: DO + DEGREES[name] + shift + 12 * octave, steps: 1 + hold.length });
  }
  return { notes, error: null };
}
