/**
 * Finnish through the page's path: the wasm calls Speak and Sing make, and the words Speak shows
 * at the mouth.
 */
import { describe, expect, test } from "bun:test";
import { initEngine } from "./harness.ts";

await initEngine();
const { finnishPhrases, lyricsToSyllables, planPhrases } = await import("../src/wasm/engine.ts");
const { wordSpans } = await import("../src/ui/words.ts");
const female = { female: 1, child: 0 };

describe("Finnish", () => {
  test("text becomes phrases with Finnish vowels, dental stops, and initial stress", () => {
    const phrases = finnishPhrases("Tuiki, tuiki tähtönen!");
    expect(phrases.length).toBe(2);
    expect(phrases[0].segments.map((s) => s.ipa).join("")).toBe("t̪u̝iki");
    expect(phrases[1].segments.map((s) => s.ipa).join("")).toBe("t̪u̝ikit̪æ̞ht̪œnen");
    expect(phrases[1].segments[1].stress).toBe(1);
    expect(phrases[1].intonation).toBe("statement");
  });

  test("doubled vowels and consonants contrast with short ones", () => {
    const tuli = finnishPhrases("tuli")[0].segments;
    const tuuli = finnishPhrases("tuuli")[0].segments;
    const tulli = finnishPhrases("tulli")[0].segments;
    expect(tuli[1].length).toBe(0.55);
    expect(tuuli[1].length).toBe(1.4);
    expect(tulli[1].length).toBe(0.55);
    expect(tulli[2].length).toBe(2.2);
  });

  test("each word gets its span in the plan", () => {
    const phrases = finnishPhrases("Revontulet leiskuvat yöllä.");
    const plan = planPhrases(phrases, { body: female, t0: 0 });
    const words = wordSpans(phrases, plan);
    expect(words.map((w) => w.text)).toEqual(["revontulet", "leiskuvat", "yöllä"]);
    for (let i = 1; i < words.length; i++) expect(words[i].t0).toBeGreaterThanOrEqual(words[i - 1].t1 - 1e-9);
  });

  test("lyrics in FI become Finnish syllables; other scripts still go to their own front ends", () => {
    const syl = lyricsToSyllables("Tuiki tähtien 빵 дом", "fi");
    expect(syl.map((s) => s.text)).toEqual(["Tui", "ki", "täh", "ti", "en", "빵", "дом"]);
    expect(syl[1].wordEnd).toBe(true);
    expect(syl[4].wordEnd).toBe(true);
  });

  test("lyrics are English unless FI is chosen", () => {
    expect(lyricsToSyllables("yöllä tähtönen").every((s) => /^[ -~]*$/.test(s.text))).toBe(true);
    expect(lyricsToSyllables("yöllä tähtönen", "en")).toEqual(lyricsToSyllables("yöllä tähtönen"));
  });
});
