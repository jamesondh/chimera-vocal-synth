// Hebrew in the web app: the model's output becomes one IPA chunk per letter (src/song/hebrew.ts),
// and a line with a reading reaches the engine's Hebrew front end through the wasm build. The
// model itself (20 MB, fetched in the browser) is not run here.
import { describe, expect, test } from "bun:test";
import { initEngine } from "./harness.ts";

await initEngine();
const { lyricsToSyllables } = await import("../src/wasm/engine.ts");
const { decode, modelInput, onnxMetadata, readMeta, setHebrewReading, unreadHebrew } = await import("../src/song/hebrew.ts");

// the model's own tables, cut down to the letters used here
const meta = readMeta({
  vocab: JSON.stringify({ "[UNK]": 3, " ": 73, "'": 47, ר: 98, ו: 79, ח: 81, ג: 76, ב: 75 }),
  consonant_vocab: JSON.stringify({ 0: "∅", 1: "b", 2: "v", 3: "χ", 4: "ʁ", 5: "ɡ", 6: "dʒ" }),
  vowel_vocab: JSON.stringify({ 0: "∅", 1: "a", 2: "u" }),
  cls_token_id: "1",
  sep_token_id: "2",
  letter_consonant_constraints: JSON.stringify({ ר: [4], ו: [2, 0], ח: [3], ג: [5, 6], ב: [1, 2] }),
  geresh_map: JSON.stringify({ ג: "dʒ" }),
});

/** logits that pick `picks[i] = [consonant, vowel, stress]` for the line's i-th character */
function logits(picks: [number, number, number][]) {
  const rows = [[0, 0, 0], ...picks, [0, 0, 0]];
  const hot = (n: number, k: (r: number[]) => number) => rows.flatMap((r) => Array.from({ length: n }, (_, i) => (i === k(r) ? 5 : 0)));
  return { consonant: hot(7, (r) => r[0]), vowel: hot(3, (r) => r[1]), stress: rows.flatMap((r) => [0, r[2]]) };
}

describe("Hebrew lyrics in the web app", () => {
  test("a chunk per letter: the word's stress, a silent letter, the a before a final ח", () => {
    const { chars, counted, ids } = modelInput("רוח", meta);
    expect(ids).toEqual([1, 98, 79, 81, 2]);
    expect(decode(chars, counted, meta, logits([[4, 2, 9], [0, 0, 1], [3, 1, 0]]))).toEqual(["ʁˈu", "", "aχ"]);
  });

  test("a consonant the letter can't be gives way to one it can; a geresh picks its sound", () => {
    // ב can't be χ (3): of b and v the likelier, here the first; ג' is dʒ whatever the model says
    const { chars, counted } = modelInput("ג׳ב", meta);
    expect(chars).toEqual(["ג", "'", "ב"]);
    expect(decode(chars, counted, meta, logits([[5, 1, 1], [0, 0, 0], [3, 0, 0]]))).toEqual(["dʒˈa", "b"]);
  });

  test("a presentation form reaches the model as its letter but is not a letter the engine counts", () => {
    const { chars, counted } = modelInput("\uFB31 ב", meta); // U+FB31, then a plain bet
    expect(chars).toEqual(["ב", "ּ", " ", "ב"]);
    expect(decode(chars, counted, meta, logits([[1, 1, 1], [0, 0, 0], [0, 0, 0], [2, 2, 1]]))).toEqual(["vˈu"]);
  });

  test("the metadata is read from the ONNX file's own bytes", () => {
    const enc = new TextEncoder();
    const str = (field: number, s: string) => [field * 8 + 2, enc.encode(s).length, ...enc.encode(s)];
    const entry = [...str(1, "vocab"), ...str(2, '{"א":74}')];
    // ir_version (a varint), a graph to skip, one metadata entry
    const file = new Uint8Array([8, 9, 7 * 8 + 2, 3, 1, 2, 3, 14 * 8 + 2, entry.length, ...entry]);
    expect(onnxMetadata(file)).toEqual({ vocab: '{"א":74}' });
  });

  test("a line with a reading is sung; one without is skipped, and the rest still sings", () => {
    const text = "שלום star\nעולם";
    expect(lyricsToSyllables(text).map((s) => s.text)).toEqual(["star"]);
    expect(unreadHebrew(text)).toEqual(["שלום star", "עולם"]);
    setHebrewReading("עולם", ["ʔo", "", "lˈa", "m"]);
    expect(lyricsToSyllables(text).map((s) => s.text)).toEqual(["star", "עו", "לם"]);
    setHebrewReading("שלום star", ["ʃa", "lˈo", "", "m"]);
    expect(lyricsToSyllables(text).map((s) => s.text)).toEqual(["ש", "לום", "star", "עו", "לם"]);
    expect(unreadHebrew(text)).toEqual([]);
    expect(() => setHebrewReading("שלום", ["ʃa"])).toThrow();
  });
});

describe("right-to-left text", () => {
  test("the first letter decides, and a word that goes on gets its hyphen in reading order", async () => {
    const { isRtl, goesOn } = await import("../src/ui/rtl.ts");
    expect(["שלום", "“שלום”", "שלום star", "1. שלום"].map(isRtl)).toEqual([true, true, true, true]);
    expect(["star שלום", "별", "мир", "", "123"].map(isRtl)).toEqual([false, false, false, false, false]);
    expect(goesOn("ש")).toBe("ש-\u200F");
    expect(goesOn("twin")).toBe("twin-");
  });
});
