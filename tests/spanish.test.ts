/**
 * The Spanish front-end through the wasm build: explicit shared language routing,
 * dialect presets, and Spanish syllables for singing. The rules themselves are covered deeply
 * by the Rust test (`crates/chimera/tests/spanish.rs`); this checks the page's wiring.
 */
import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { gunzipSync } from "node:zlib";
import { initEngine } from "./harness.ts";
import type { Dialect } from "../src/wasm/engine.ts";

await initEngine();
const { EnglishG2P, DIALECT_PRESETS, lyricsToSyllables, planUtterance } = await import("../src/wasm/engine.ts");

const g2p = new EnglishG2P();
let lang: "en" | "es" = "en";
let dialect: Dialect = DIALECT_PRESETS.neutro;
const setLang = (next: "en" | "es", preset: Dialect = DIALECT_PRESETS.neutro) => { lang = next; dialect = preset; };
// the dictionary the page itself loads (English needs it; Spanish never does)
g2p.setDictionaryText(gunzipSync(readFileSync(new URL("../public/data/cmudict.txt.gz", import.meta.url))).toString());

/** the IPA of a text's first phrase, stress marks dropped for readability */
function ipa(text: string): string {
  return g2p
    .toPhrases(text, lang, dialect)[0]
    .segments.map((s) => s.ipa)
    .join("");
}

describe("spanish voice language", () => {
  test("english keeps its route until the voice switches", () => {
    setLang("en");
    // "hello" goes through CMUdict: the English diphthong symbol ends the word
    expect(ipa("Hello.")).toBe("həloʊ");
  });

  test("spanish says its rules (no dictionary): silent h, the r sounds, written stress", () => {
    setLang("es", DIALECT_PRESETS.neutro);
    expect(ipa("Hola")).toBe("ola"); // the h is silent
    expect(ipa("perro")).toBe("pero"); // rr = the trill r
    expect(ipa("caro")).toBe("kaɾo"); // single r between vowels = the tap
    const ps = g2p.toPhrases("Hola. ¿Cómo estás?", lang, dialect);
    expect(ps.map((p) => p.words.join(" "))).toEqual(["hola", "cómo estás"]);
    expect(ps[1].intonation).toBe("question");
    // the ˈ sits on the stressed vowel (gui-TA-rra)
    const segs = g2p.toPhrases("guitarra", lang, dialect)[0].segments;
    expect(segs[segs.findIndex((s) => s.stress === 1)].ipa).toBe("a");
  });

  test("the dialect presets swap ll/y and the coda s", () => {
    setLang("es", DIALECT_PRESETS.neutro);
    expect(ipa("llave")).toBe("ʝaβe");
    setLang("es", DIALECT_PRESETS.rioplatense);
    expect(ipa("llave")).toBe("ʃaβe");
    expect(ipa("mismo")).toBe("mihmo");
    setLang("es", DIALECT_PRESETS.castellano);
    expect(ipa("caza")).toBe("kaθa");
    setLang("es", DIALECT_PRESETS.neutro);
    expect(ipa("caza")).toBe("kasa");
  });

  test("sung syllables follow the voice language", () => {
    setLang("es", DIALECT_PRESETS.neutro);
    expect(lyricsToSyllables("país", lang, dialect).map((s) => s.text)).toEqual(["pa", "ís"]); // the hiatus holds
    expect(lyricsToSyllables("La luna llena", lang, dialect).map((s) => s.text)).toEqual(["La", "lu", "na", "lle", "na"]);
    setLang("en");
    // English is untouched: silent final e, no accented letters
    expect(lyricsToSyllables("little", lang, dialect).map((s) => s.text)).toEqual(["lit", "tle"]);
  });

  test("sung syllables keep the word's allophones", () => {
    setLang("es", DIALECT_PRESETS.neutro);
    // the singing front end applies the allophones before it splits syllables
    const syls = lyricsToSyllables("banco", lang, dialect);
    const onsets = syls.map((s) => s.onset.map((c) => c.ipa).join(""));
    expect(onsets).toEqual(["b", "k"]); // the ŋ is the coda of the first syllable
    expect(syls[0].coda.map((c) => c.ipa)).toEqual(["ŋ"]);
    setLang("en");
  });

  test("the planner aspirates English stops, never Spanish ones", () => {
    setLang("es", DIALECT_PRESETS.neutro);
    // a syllable-initial /p/ before a vowel: the English plan opens the folds wide (0.36). The
    // Spanish one is the front end's call: its G2P marks the stop `aspirated: false`, and the
    // planner barely parts the folds (0.22)
    const aspiration = (segs: { ipa: string; stress?: number }[]) =>
      planUtterance(segs, { body: { female: 0, child: 0 }, t0: 0.05 }).gestures.find((g) => g.label === "aspiration")?.targets.abduction;
    expect(aspiration([{ ipa: "p" }, { ipa: "a", stress: 1 }])).toBe(0.36); // English is the default
    const spanish = g2p.toPhrases("paso", lang, dialect)[0].segments;
    expect(spanish[0].aspirated).toBe(false);
    expect(aspiration(spanish.slice(0, 2))).toBe(0.22);
    setLang("en");
  });
});
