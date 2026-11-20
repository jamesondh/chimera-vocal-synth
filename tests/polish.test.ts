/**
 * Polish through the page's path: the wasm calls Speak and Sing make, and the words Speak shows
 * at the mouth.
 */
import { describe, expect, test } from "bun:test";
import { initEngine } from "./harness.ts";

await initEngine();
const { polishPhrases, lyricsToSyllables, planPhrases } = await import("../src/wasm/engine.ts");
const { wordSpans } = await import("../src/ui/words.ts");
const male = { female: 0, child: 0 };

describe("Polish", () => {
  test("text becomes phrases with Polish sounds and stress", () => {
    const phrases = polishPhrases("Dzień dobry, masz czas?");
    expect(phrases.length).toBe(2);
    expect(phrases[0].segments.map((s) => s.ipa).join("")).toBe("dʑɛɲdɔbɾɨ");
    expect(phrases[0].segments.filter((s) => s.stress === 1).map((s) => s.ipa)).toEqual(["ɛ", "ɔ"]);
    // a question without a question word rises
    expect(phrases[1].intonation).toBe("question");
  });

  test("nasal vowels and palatalized consonants reach the planner", () => {
    const segs = polishPhrases("mąż pies")[0].segments;
    expect(segs[1]).toMatchObject({ ipa: "ɔ", nasalized: true });
    expect(segs[3]).toMatchObject({ ipa: "p", palatalized: true });
  });

  test("each word gets its span in the plan, a vowelless word with the next", () => {
    const phrases = polishPhrases("Jestem w domu.");
    const plan = planPhrases(phrases, { body: male, t0: 0 });
    const words = wordSpans(phrases, plan);
    expect(words.map((w) => w.text)).toEqual(["jestem", "w domu"]);
    expect(words[1].t0).toBeGreaterThanOrEqual(words[0].t1 - 1e-9);
  });

  test("lyrics in PL become Polish syllables; other scripts still go to their own front ends", () => {
    const syl = lyricsToSyllables("Dzień dobry 빵 дом", "pl");
    expect(syl.map((s) => s.text)).toEqual(["Dzień", "do", "bry", "빵", "дом"]);
    expect(syl[0].wordEnd).toBe(true);
  });

  test("lyrics are English unless PL is chosen", () => {
    // English claims only ASCII letters: ź and ń are skipped
    expect(lyricsToSyllables("dzień").every((s) => /^[ -~]*$/.test(s.text))).toBe(true);
  });
});
