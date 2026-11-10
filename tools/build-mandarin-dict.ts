/**
 * Build the Mandarin pronunciation dictionary shipped to the browser, from mozillazg's
 * pinyin-data (characters, most common reading first) and phrase-pinyin-data (words, which fix
 * the readings of polyphones: 银行 yín háng), both MIT.
 * Usage: bun tools/build-mandarin-dict.ts
 * Output: public/data/mandarin.txt.gz (lines: "汉字<TAB>pīn yīn", one reading per entry) and
 *         public/data/MANDARIN-LICENSE.txt
 */
import { writeFileSync } from "node:fs";
import { gzipSync } from "node:zlib";

const CHARS = "https://raw.githubusercontent.com/mozillazg/pinyin-data/v0.16.0";
const PHRASES = "https://raw.githubusercontent.com/mozillazg/phrase-pinyin-data/v0.20.0";
const get = async (url: string) => {
  const r = await fetch(url);
  if (!r.ok) throw new Error(`${url}: ${r.status}`);
  return r.text();
};

const out: string[] = [];
// "U+884C: xíng,háng,…  # 行": the first reading, CJK unified ideographs only
for (const line of (await get(`${CHARS}/pinyin.txt`)).split("\n")) {
  const m = line.match(/^U\+([0-9A-F]+): ([^,#\s]+)/);
  if (!m) continue;
  const cp = parseInt(m[1], 16);
  if (cp < 0x4e00 || cp > 0x9fff) continue;
  out.push(`${String.fromCodePoint(cp)}\t${m[2]}`);
}
const nChars = out.length;
// "银行: yín háng"
for (const line of (await get(`${PHRASES}/pinyin.txt`)).split("\n")) {
  const m = line.match(/^([^#:]+): (.+)$/);
  if (m && [...m[1]].length > 1) out.push(`${m[1]}\t${m[2].trim()}`);
}
const gz = gzipSync(Buffer.from(out.join("\n"), "utf8"), { level: 9 });
writeFileSync("public/data/mandarin.txt.gz", gz);
writeFileSync(
  "public/data/MANDARIN-LICENSE.txt",
  `pinyin-data v0.16.0 (${CHARS})\n\n${await get(`${CHARS}/LICENSE`)}\n\nphrase-pinyin-data v0.20.0 (${PHRASES})\n\n${await get(`${PHRASES}/LICENSE`)}`,
);
console.log(`${nChars} characters, ${out.length - nChars} phrases, ${(gz.length / 1024).toFixed(0)} KiB gzipped`);
