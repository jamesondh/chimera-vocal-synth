/**
 * Brazilian Portuguese through the page's path: the wasm calls Speak and Sing make, and the words
 * Speak shows at the mouth.
 */
import { describe, expect, test } from "bun:test";
import { initEngine } from "./harness.ts";

await initEngine();
const { phrases: phrasesIn, lyricsToSyllables, planPhrases } = await import("../src/wasm/engine.ts");
const { wordSpans } = await import("../src/ui/words.ts");
const male = { female: 0, child: 0 };

describe("Portuguese", () => {
  test("text becomes phrases with Brazilian sounds and stress", () => {
    const phrases = phrasesIn("Bom dia! Como você está?", "pt");
    expect(phrases.length).toBe(2);
    expect(phrases[0].segments.map((s) => s.ipa).join("")).toBe("bɔ̃mdʒiɐ");
    expect(phrases[1].segments.filter((s) => s.stress === 1).map((s) => s.ipa)).toEqual(["o", "e", "a"]);
    expect(phrases[1].intonation).toBe("statement");
  });

  test("the three r's and the nasal diphthong", () => {
    expect(phrasesIn("Português", "pt")[0].segments.map((s) => s.ipa)).toEqual([
      "p", "u", "ʁ", "t", "u", "g", "ɛ", "s",
    ]);
    expect(phrasesIn("pão", "pt")[0].segments.map((s) => s.ipa)).toEqual(["p", "ɐ̃w"]);
    expect(phrasesIn("carioca", "pt")[0].segments.map((s) => s.ipa)).toEqual([
      "k", "a", "ɾ", "i", "o", "k", "ɐ",
    ]);
  });

  test("each word gets its span in the plan", () => {
    const phrases = phrasesIn("Bom dia. Como você está?", "pt");
    const plan = planPhrases(phrases, { body: male, t0: 0 });
    const words = wordSpans(phrases, plan);
    expect(words.map((w) => w.text)).toEqual(["Bom", "dia", "Como", "você", "está"]);
    for (let i = 1; i < words.length; i++) expect(words[i].t0).toBeGreaterThanOrEqual(words[i - 1].t1 - 1e-9);
  });

  test("lyrics in PT become Portuguese syllables; other scripts still go to their own front ends", () => {
    const syl = lyricsToSyllables("Português coração 빵 дом", "pt");
    expect(syl.map((s) => s.text)).toEqual(["por", "tu", "guê", "co", "ra", "ção", "빵", "дом"]);
    expect(syl[2].wordEnd).toBe(true);
  });

  test("lyrics are English unless PT is chosen", () => {
    expect(lyricsToSyllables("çok güzel").every((s) => /^[ -~]*$/.test(s.text))).toBe(true);
    expect(lyricsToSyllables("çok güzel", "en")).toEqual(lyricsToSyllables("çok güzel"));
  });
});
