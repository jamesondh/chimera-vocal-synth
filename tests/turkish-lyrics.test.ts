/**
 * Turkish lyrics through the wasm engine: the "tr" option reads the spelling by rule, and the
 * syllables plan into a sung score like English ones. Without the option, lyrics stay English.
 */
import { describe, expect, test } from "bun:test";
import { initEngine } from "./harness.ts";

await initEngine();
const { lyricsToSyllables, planSong } = await import("../src/wasm/engine.ts");
const male = { female: 0, child: 0 };

describe("Turkish lyrics", () => {
  test("syllables and sounds come from the Turkish spelling", () => {
    const syl = lyricsToSyllables("Üsküdar'a gider iken", "tr");
    expect(syl.map((s) => s.text)).toEqual(["Üs", "kü", "da", "r'a", "gi", "der", "i", "ken"]);
    expect(syl.map((s) => s.nucleus.ipa)).toEqual(["y", "y", "a", "a", "i", "e", "i", "e"]);
    // front k and g before front vowels
    expect(syl[1].onset.map((s) => s.ipa)).toEqual(["c"]);
    expect(syl[4].onset.map((s) => s.ipa)).toEqual(["ɟ"]);
  });

  test("they plan into a song", () => {
    const syl = lyricsToSyllables("ışık gönül", "tr");
    const notes = syl.map((s, i) => ({ midi: 57 + i, beats: 1, syl: s }));
    const plan = planSong(notes, { body: male, bpm: 90, t0: 0, vibrato: 30 });
    expect(plan.gestures.length).toBeGreaterThan(0);
    const sung = plan.segments.map((s) => s.ipa);
    for (const v of ["ɯ", "ø", "y"]) expect(sung).toContain(v);
  });

  test("lyrics stay English by default", () => {
    expect(lyricsToSyllables("hello there")).toEqual(lyricsToSyllables("hello there", "en"));
    expect(lyricsToSyllables("gider", "tr")[0].nucleus.ipa).toBe("i");
  });
});
