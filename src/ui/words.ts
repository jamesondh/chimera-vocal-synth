/** Where each spoken word falls in a speech plan's timeline (for the words drawn at the mouth). */
import { plannedCounts, type Plan, type Segment } from "../wasm/engine.ts";

export interface WordSpan {
  text: string;
  /** plan time (s) */
  t0: number;
  t1: number;
}

/**
 * Each word's span in the plan of these phrases: from its first segment's start to its last's end.
 * A word ends at a segment marked `wordEnd` or at the end of its phrase. Without written words
 * (IPA input), a word is shown as its IPA.
 */
export function wordSpans(phrases: { segments: Segment[]; words?: string[] }[], plan: Plan): WordSpan[] {
  const out: WordSpan[] = [];
  const counts = plannedCounts(phrases.flatMap((p) => p.segments.map((s) => s.ipa)));
  let k = 0, j = 0;
  for (const p of phrases) {
    let wi = 0, t0: number | null = null, ipa = "";
    p.segments.forEach((s, i) => {
      const n = counts[k++];
      const first = plan.segments[j], last = plan.segments[j + n - 1];
      j += n;
      if (!first || !last) return;
      t0 ??= first.t0;
      ipa += s.ipa;
      if (s.wordEnd || i === p.segments.length - 1) {
        out.push({ text: p.words?.[wi] ?? ipa, t0, t1: last.t1 });
        wi++;
        t0 = null;
        ipa = "";
      }
    });
  }
  return out;
}

/** Chao tone letters (shown in the transcription) */
const TONE_LETTERS = ["", "˥", "˧˥", "˨˩˦", "˥˩", ""];

/** Phrases as shown under the text box: IPA with stress marks, aspiration and tone letters. */
export function transcribe(phrases: { segments: Segment[]; intonation?: string }[]): string {
  // a tone letter ends its syllable: after the nucleus, or after a nasal coda (a nasal right after
  // a toned nucleus)
  const tone = (segs: Segment[], i: number) => {
    const t = segs[i].tone ?? (/^[nŋ]$/.test(segs[i].ipa) ? segs[i - 1]?.tone : undefined);
    return t && !(segs[i].tone && /^[nŋ]$/.test(segs[i + 1]?.ipa ?? "")) ? TONE_LETTERS[t] : "";
  };
  return phrases.map((p) => p.segments.map((s, i) => (s.tone ? "" : s.stress === 1 ? "ˈ" : s.stress === 2 ? "ˌ" : "") + s.ipa + (s.aspirated ? "ʰ" : "") + tone(p.segments, i) + (s.wordEnd ? " " : "")).join("").trim() + (p.intonation === "question" ? "?" : "")).join(" | ");
}
