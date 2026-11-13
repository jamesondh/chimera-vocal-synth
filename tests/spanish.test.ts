/**
 * The Spanish front-end through the wasm build: the language routing (`G2P.setLang`), the
 * dialect presets, and Spanish syllables for singing. The rules themselves are covered deeply
 * by the Rust test (`crates/chimera/tests/spanish.rs`); this checks the page's wiring.
 */
import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { gunzipSync } from "node:zlib";
import { initEngine } from "./harness.ts";

await initEngine();
const { G2P, DIALECT_PRESETS, lyricsToSyllables, planUtterance } = await import("../src/wasm/engine.ts");

const g2p = new G2P();
// the dictionary the page itself loads (English needs it; Spanish never does)
g2p.setDictionaryText(gunzipSync(readFileSync(new URL("../public/data/cmudict.txt.gz", import.meta.url))).toString());

/** the IPA of a text's first phrase, stress marks dropped for readability */
function ipa(text: string): string {
  return g2p
    .toPhrases(text)[0]
    .segments.map((s) => s.ipa)
    .join("");
}

describe("spanish voice language", () => {
  test("english keeps its route until the voice switches", () => {
    g2p.setLang("en");
    // "hello" goes through CMUdict: the English diphthong symbol ends the word
    expect(ipa("Hello.")).toBe("həloʊ");
  });

  test("spanish says its rules (no dictionary): silent h, the r sounds, written stress", () => {
    g2p.setLang("es", DIALECT_PRESETS.neutro);
    expect(ipa("Hola")).toBe("ola"); // the h is silent
    expect(ipa("perro")).toBe("pero"); // rr = the trill r
    expect(ipa("caro")).toBe("kaɾo"); // single r between vowels = the tap
    const ps = g2p.toPhrases("Hola. ¿Cómo estás?");
    expect(ps.map((p) => p.words.join(" "))).toEqual(["hola", "cómo estás"]);
    expect(ps[1].intonation).toBe("question");
    // the ˈ sits on the stressed vowel (gui-TA-rra)
    const segs = g2p.toPhrases("guitarra")[0].segments;
    expect(segs[segs.findIndex((s) => s.stress === 1)].ipa).toBe("a");
  });

  test("the dialect presets swap ll/y and the coda s", () => {
    g2p.setLang("es", DIALECT_PRESETS.neutro);
    expect(ipa("llave")).toBe("ʝaβe");
    g2p.setLang("es", DIALECT_PRESETS.rioplatense);
    expect(ipa("llave")).toBe("ʃaβe");
    expect(ipa("mismo")).toBe("mihmo");
    g2p.setLang("es", DIALECT_PRESETS.castellano);
    expect(ipa("caza")).toBe("kaθa");
    g2p.setLang("es", DIALECT_PRESETS.neutro);
    expect(ipa("caza")).toBe("kasa");
  });

  test("sung syllables follow the voice language", () => {
    g2p.setLang("es", DIALECT_PRESETS.neutro);
    expect(lyricsToSyllables("país").map((s) => s.text)).toEqual(["pa", "ís"]); // the hiatus holds
    expect(lyricsToSyllables("La luna llena").map((s) => s.text)).toEqual(["La", "lu", "na", "lle", "na"]);
    g2p.setLang("en");
    // English is untouched: silent final e, no accented letters
    expect(lyricsToSyllables("little").map((s) => s.text)).toEqual(["lit", "tle"]);
  });

  test("the planner aspirates English stops, never Spanish ones", () => {
    // a syllable-initial /p/ before a vowel: the English plan opens the folds wide (0.36), the
    // Spanish one barely parts them (0.22)
    const segs = [{ ipa: "p" }, { ipa: "a", stress: 1 }];
    const aspiration = (lang?: "english" | "spanish") =>
      planUtterance(segs, { body: { female: 0, child: 0 }, t0: 0.05, lang }).gestures.find((g) => g.label === "aspiration")?.targets.abduction;
    expect(aspiration()).toBe(0.36); // English is the default
    expect(aspiration("english")).toBe(0.36);
    expect(aspiration("spanish")).toBe(0.22);
  });
});
