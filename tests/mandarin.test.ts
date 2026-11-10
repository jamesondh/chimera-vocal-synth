/**
 * Mandarin through the wasm build: characters → tones (with sandhi), tone contours in the plan,
 * word spans, and lyrics one syllable per character.
 */
import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { gunzipSync } from "node:zlib";
import { initEngine } from "./harness.ts";

await initEngine();
const { MandarinG2P, isMandarin, lyricsToSyllables, planPhrases } = await import("../src/wasm/engine.ts");
const { wordSpans } = await import("../src/ui/words.ts");
const zh = new MandarinG2P();
zh.setDictionaryText(gunzipSync(readFileSync(new URL("../public/data/mandarin.txt.gz", import.meta.url))).toString());
const male = { female: 0, child: 0 };
const tones = (text: string) => zh.toPhrases(text).map((p) => p.segments.flatMap((s) => (s.tone ? [s.tone] : [])));

describe("mandarin", () => {
  test("characters read through the dictionary, with tone sandhi", () => {
    expect(tones("你好，银行。")).toEqual([[2, 3], [2, 2]]);
    expect(tones("一个不是")).toEqual([[2, 4, 2, 4]]);
    expect(zh.toPhrases("银行")[0].segments.map((s) => s.ipa).join("")).toBe("inhɑŋ");
    expect(zh.toPhrases("ni3 hao3 ma?")[0].intonation).toBe("question");
  });

  test("a fourth tone falls and a second rises across its vowel", () => {
    for (const [text, rising] of [["ma4", false], ["ma2", true]] as const) {
      const plan = planPhrases(zh.toPhrases(text), { body: male, t0: 0, f0: 110 });
      const f0 = plan.gestures.filter((g) => g.label === "pitch").sort((a, b) => a.t0 - b.t0).map((g) => g.targets.f0!);
      expect(f0.length).toBe(2);
      expect(f0[1] > f0[0]).toBe(rising);
    }
  });

  test("word spans cover each dictionary word", () => {
    const phrases = zh.toPhrases("我们去银行。");
    const words = wordSpans(phrases, planPhrases(phrases, { body: male, t0: 0 }));
    expect(words.map((w) => w.text)).toEqual(["我们", "去", "银行"]);
  });

  test("lyrics: one syllable per character, and detection", () => {
    const syl = lyricsToSyllables("一闪一闪亮晶晶");
    expect(syl.map((s) => s.text).join("")).toBe("一闪一闪亮晶晶");
    expect(syl[6].onset.map((s) => s.ipa)).toEqual(["tɕ"]);
    expect(isMandarin("你好")).toBe(true);
    expect(isMandarin("Hello there")).toBe(false);
  });
});

test("the transcription puts each tone letter at its syllable's end", async () => {
  const { transcribe } = await import("../src/ui/words.ts");
  expect(transcribe(zh.toPhrases("今天很好"))).toBe("tɕin˥ tʰjɛn˥ hən˧˥ haʊ˨˩˦");
});
