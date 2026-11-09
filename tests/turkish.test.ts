/**
 * Turkish through the page's path: the wasm call the Speak panel makes, and the words it shows at
 * the mouth.
 */
import { describe, expect, test } from "bun:test";
import { initEngine } from "./harness.ts";

await initEngine();
const { turkishPhrases, turkishLyricsToSyllables, planPhrases } = await import("../src/wasm/engine.ts");
const { wordSpans } = await import("../src/ui/words.ts");
const male = { female: 0, child: 0 };

describe("Turkish", () => {
  test("text becomes phrases with Turkish sounds and stress", () => {
    const phrases = turkishPhrases("Merhaba, nasılsın?");
    expect(phrases.length).toBe(2);
    expect(phrases[1].segments.map((s) => s.ipa).join("")).toBe("nasɯɫsɯn");
    expect(phrases[1].segments.find((s) => s.stress === 1)?.ipa).toBe("a");
    expect(phrases[1].intonation).toBe("statement");
  });

  test("each word gets its span in the plan", () => {
    const phrases = turkishPhrases("Bugün hava çok güzel.");
    const plan = planPhrases(phrases, { body: male, t0: 0 });
    const words = wordSpans(phrases, plan);
    expect(words.map((w) => w.text)).toEqual(["bugün", "hava", "çok", "güzel"]);
    for (let i = 1; i < words.length; i++) expect(words[i].t0).toBeGreaterThanOrEqual(words[i - 1].t1 - 1e-9);
  });

  test("lyrics become Turkish syllables", () => {
    const syl = turkishLyricsToSyllables("Gesi bağlarında");
    expect(syl.map((s) => s.text)).toEqual(["Ge", "si", "bağ", "la", "rın", "da"]);
    expect(syl[1].wordEnd).toBe(true);
  });
});
