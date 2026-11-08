/**
 * The Arabic page on the wasm build: the reader through the facade (segments that know their
 * letters), its words at the mouth, its letters' forms, and the /arabic address.
 */
import { describe, expect, test } from "bun:test";
import { initEngine } from "./harness.ts";

await initEngine();
const { arabicPhrases, planPhrases } = await import("../src/wasm/engine.ts");
const { wordSpans } = await import("../src/ui/words.ts");
const { LETTERS, LETTER_FORMS, PRESET_GROUPS, letterOf, letterText } = await import("../src/ui/arabicData.ts");
const { modeOfPath, pathOfMode } = await import("../src/ui/route.ts");
const male = { female: 0, child: 0 };

describe("the Arabic reader through the facade", () => {
  test("the basmala: segments with their letters, words and rules", () => {
    const text = "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ";
    const [p] = arabicPhrases(text);
    expect(p.ipa.replace(/ˈ/g, "")).toBe("bismi lːaːhi rˤːaˤħmaːni rˤːaˤħiːm");
    expect(p.sources.length).toBe(p.segments.length);
    expect(p.rules.length).toBe(p.segments.length);
    expect(p.words).toEqual(["بِسْمِ", "ٱللَّهِ", "ٱلرَّحْمَٰنِ", "ٱلرَّحِيمِ"]);
    const chars = Array.from(text);
    const b = p.segments.findIndex((s) => s.ipa === "b");
    expect(chars.slice(...p.sources[b]).join("")).toBe("بِ");
    // the doubled heavy rāʾ is read from the article's silent lām and the rāʾ
    const r = p.segments.findIndex((s) => s.ipa === "rˤ");
    expect(p.rules[r]).toContain("shamsiyya");
    expect(p.rules[r]).toContain("tafkhim");
  });

  test("its words fall where they are said", () => {
    const phrases = arabicPhrases("السَّلَامُ عَلَيْكُمْ. كَيْفَ حَالُكَ؟");
    const plan = planPhrases(phrases, { body: male, t0: 0 });
    // one planned segment per read segment (Arabic never expands a diphthong)
    expect(plan.segments.length).toBe(phrases.reduce((n, p) => n + p.segments.length, 0));
    expect(wordSpans(phrases, plan).map((w) => w.text)).toEqual(["السَّلَامُ", "عَلَيْكُمْ", "كَيْفَ", "حَالُكَ"]);
    expect(phrases[1].intonation).toBe("question");
  });

  test("pause forms can be turned off", () => {
    expect(arabicPhrases("كِتَابٌ")[0].ipa.replace(/ˈ/g, "")).toBe("kitaːb");
    expect(arabicPhrases("كِتَابٌ", { waqf: false })[0].ipa.replace(/ˈ/g, "")).toBe("kitaːbun");
  });
});

describe("the page's content", () => {
  test("every letter in every form reads as something", () => {
    for (const l of LETTERS)
      for (const f of LETTER_FORMS) {
        const { text, waqf } = letterText(l, f.id);
        const ps = arabicPhrases(text, { waqf });
        expect(ps.length).toBeGreaterThan(0);
        expect(ps[0].guessed).toBe(0);
      }
  });

  test("a letter with fatḥa keeps its vowel, and the still form is how tajweed tests it", () => {
    const ba = LETTERS.find((l) => l.id === "ba")!;
    const read = (f: "fatha" | "sakin") => {
      const { text, waqf } = letterText(ba, f);
      return arabicPhrases(text, { waqf })[0].ipa.replace(/ˈ/g, "");
    };
    expect(read("fatha")).toBe("ba");
    expect(read("sakin")).toBe("ʔab");
  });

  test("every preset is fully vowelled", () => {
    for (const g of PRESET_GROUPS)
      for (const p of g.presets) expect(arabicPhrases(p.text).reduce((n, ph) => n + ph.guessed, 0)).toBe(0);
  });

  test("written letters find their chart letter", () => {
    expect(letterOf("أ")?.id).toBe("hamza");
    expect(letterOf("ة")?.id).toBe("ta");
    expect(letterOf("و")?.id).toBe("waw");
    expect(letterOf("ص")?.ipa).toBe("sˤ");
  });
});

describe("the address", () => {
  test("/arabic opens the Arabic page and the Arabic page lives at /arabic", () => {
    expect(modeOfPath("/arabic")).toBe("arabic");
    expect(modeOfPath("/arabic/")).toBe("arabic");
    expect(modeOfPath("/")).toBe("play");
    expect(modeOfPath("/arabicx")).toBe("play");
    expect(pathOfMode("arabic")).toBe("/arabic");
    expect(pathOfMode("speak")).toBe("/");
  });
});
