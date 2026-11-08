/**
 * The Arabic page's speech: vowelled text → the engine's Arabic reader → the same planner and
 * worklet path as Speak. It remembers what each planned segment was read from, so the page can
 * light the letters being said.
 */
import type { ArabicOptions, ArabicPhrase, BodyMix } from "../wasm/types.ts";
import { arabicPhrases } from "../wasm/engine.ts";
import { currentPlan, speakPhrases, type SpokenPlan } from "./speech.ts";
import { planNow } from "./components/ScoreView.tsx";

/** What the voice is reading: the text, its phrases, and the plan they became. */
export interface ArabicReading {
  text: string;
  phrases: ArabicPhrase[];
  /** per planned segment (all phrases in order): its source span, rules and length in ḥarakāt */
  sources: [number, number][];
  rules: string[][];
  counts: number[];
  /** the reading in IPA, phrases joined by " | " */
  ipa: string;
  guessed: number;
  /** the spoken plan's id (speech.ts) */
  id: number;
}

let reading: ArabicReading | null = null;
const listeners = new Set<(r: ArabicReading | null) => void>();

export function onReading(l: (r: ArabicReading | null) => void): () => void {
  listeners.add(l);
  return () => listeners.delete(l);
}

export function currentReading(): ArabicReading | null {
  return reading;
}

/** Read a text without saying it (for the transcription as one types). */
export function readArabic(text: string, options?: ArabicOptions): ArabicPhrase[] {
  return text.trim() ? arabicPhrases(text, options) : [];
}

/** Say vowelled Arabic. Returns null when there is nothing to say. */
export function speakArabic(text: string, o: { body: BodyMix; rate: number; f0: number; waqf?: boolean; count?: number }): ArabicReading | null {
  const phrases = readArabic(text, { waqf: o.waqf ?? true, count: o.count });
  if (!phrases.length) return null;
  const sp: SpokenPlan = speakPhrases(phrases, { body: o.body, rate: o.rate, f0: o.f0 });
  reading = {
    text,
    phrases,
    sources: phrases.flatMap((p) => p.sources),
    rules: phrases.flatMap((p) => p.rules),
    counts: phrases.flatMap((p) => p.counts),
    ipa: phrases.map((p) => p.ipa).join(" | "),
    guessed: phrases.reduce((n, p) => n + p.guessed, 0),
    id: sp.id,
  };
  for (const l of listeners) l(reading);
  return reading;
}

/**
 * The segment of the reading sounding now (its IPA marked long, "aː", when it lasts two counts),
 * with its source span and rules; null if none.
 */
export function soundingNow(): { index: number; ipa: string; source: [number, number]; rules: string[] } | null {
  const sp = currentPlan();
  const r = reading;
  if (!sp || !r || sp.id !== r.id) return null;
  const now = planNow(sp);
  if (now < 0) return null;
  const index = sp.plan.segments.findIndex((s) => now >= s.t0 && now < s.t1);
  if (index < 0 || index >= r.sources.length) return null;
  const ipa = sp.plan.segments[index].ipa + (r.counts[index] >= 2 ? "ː" : "");
  return { index, ipa, source: r.sources[index], rules: r.rules[index] };
}

/** The base letter at the start of a source span (its first character that is a letter). */
export function letterAt(text: string, span: [number, number]): string | null {
  const chars = Array.from(text);
  for (let i = span[0]; i < span[1] && i < chars.length; i++) {
    if (/[\u0621-\u064A\u0671]/.test(chars[i])) return chars[i];
  }
  return null;
}
