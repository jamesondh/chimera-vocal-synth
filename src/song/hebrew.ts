/**
 * Hebrew lyrics → the sounds of each letter, by ReNikud (https://github.com/renikud/renikud,
 * CC BY 4.0; Melichov, Kolani and Alper, IEEE SLT 2026): a 20 MB model that reads unvocalized
 * Hebrew in context. Everyday Hebrew doesn't write its vowels, so no spelling rule can read it.
 *
 * The model and its runtime (onnxruntime-web) are fetched only when the lyrics have Hebrew in
 * them, from Hugging Face at a pinned revision, and kept in the browser's cache. A line's reading
 * is one IPA chunk per Hebrew letter (א–ת) in order, which is what the engine's Hebrew front end
 * takes (crates/chimera/src/phonetics/hebrew.rs). The decoding follows renikud-onnx's `phonemize`.
 */
const MODEL_URL = "https://huggingface.co/renikud/renikud/resolve/a59c632c8cbac0f8c63f298cad77ef3a3a2547c9/model.onnx";
const CACHE = "chimera-renikud";
const STRESS = "ˈ";
const NONE = "∅";

const isLetter = (c: string) => c >= "א" && c <= "ת";
const letters = (line: string) => line.match(/[א-ת]/g)?.length ?? 0;

/** The lines read so far: line → one IPA chunk per letter. */
const readings = new Map<string, string[]>();

/** Keep a line's reading: one IPA chunk for each of its letters (א–ת), in order. */
export function setHebrewReading(line: string, chunks: string[]): void {
  if (chunks.length !== letters(line)) throw new Error("a Hebrew reading needs one chunk per letter");
  readings.delete(line);
  readings.set(line, chunks);
  // the lines typed on the way to the one that is sung
  if (readings.size > 500) readings.delete(readings.keys().next().value as string);
}

/** The lines of `text` with Hebrew in them and no reading yet. */
export function unreadHebrew(text: string): string[] {
  return [...new Set(text.split("\n"))].filter((l) => letters(l) > 0 && !readings.has(l));
}

/**
 * What the engine takes with lyrics: a chunk for every Hebrew letter of `text`, silent ones for a
 * line not read yet (its words are skipped). Nothing for lyrics with no Hebrew.
 */
export function hebrewChunks(text: string): string[] | undefined {
  if (!letters(text)) return undefined;
  return text.split("\n").flatMap((l) => readings.get(l) ?? Array<string>(letters(l)).fill(""));
}

/** What the model file says about itself (its ONNX metadata). */
export interface Meta {
  vocab: Record<string, number>;
  consonants: Record<string, string>;
  vowels: Record<string, string>;
  cls: number;
  sep: number;
  /** the consonants each letter can be (indices into `consonants`) */
  allowed: Record<string, number[]>;
  /** a letter followed by a geresh: צ׳ is tʃ */
  geresh: Record<string, string>;
}

/** One row per token (the line's characters between a first and a last marker token). */
export interface Logits {
  consonant: ArrayLike<number>;
  vowel: ArrayLike<number>;
  stress: ArrayLike<number>;
}

/** The characters the model sees for a line, and which of them are letters the engine counts. */
export function modelInput(line: string, meta: Meta): { chars: string[]; counted: boolean[]; ids: number[] } {
  const typed = line.replace(/[׳'`´]/g, "'").replace(/[״“”]/g, '"');
  const chars: string[] = [];
  const counted: boolean[] = [];
  // decomposed, as the model was trained; a presentation form (שׁ as one character) reaches the
  // model as its letter, but the engine doesn't count it as one, so it gets no chunk
  for (const c of typed) {
    for (const d of c.normalize("NFD")) {
      chars.push(d);
      counted.push(isLetter(c));
    }
  }
  const unk = meta.vocab["[UNK]"] ?? 0;
  return { chars, counted, ids: [meta.cls, ...chars.map((c) => meta.vocab[c] ?? unk), meta.sep] };
}

function argmax(row: ArrayLike<number>, from: number, n: number, among?: number[]): number {
  let best = among ? among[0] : 0;
  for (const k of among ?? Array.from({ length: n }, (_, i) => i)) if (row[from + k] > row[from + best]) best = k;
  return best;
}

/** The model's output for a line → one IPA chunk per counted letter. */
export function decode(chars: string[], counted: boolean[], meta: Meta, out: Logits): string[] {
  const nC = Object.keys(meta.consonants).length;
  const nV = Object.keys(meta.vowels).length;
  // one stress per written word: its most stressed character
  const stressed = new Set<number>();
  for (let i = 0; i < chars.length; ) {
    if (/\s/.test(chars[i])) {
      i++;
      continue;
    }
    let best = i;
    for (; i < chars.length && !/\s/.test(chars[i]); i++) if (out.stress[(i + 1) * 2 + 1] > out.stress[(best + 1) * 2 + 1]) best = i;
    stressed.add(best);
  }
  const chunks: string[] = [];
  chars.forEach((c, i) => {
    if (!isLetter(c) || !counted[i]) return;
    const row = i + 1; // after the first marker token
    let cid = argmax(out.consonant, row * nC, nC);
    const allowed = meta.allowed[c];
    if (allowed && !allowed.includes(cid)) cid = argmax(out.consonant, row * nC, nC, allowed);
    let consonant = meta.consonants[cid] ?? NONE;
    if (meta.geresh[c] && chars[i + 1] === "'") consonant = meta.geresh[c];
    const vowel = meta.vowels[argmax(out.vowel, row * nV, nV)] ?? NONE;
    const stress = stressed.has(i) ? STRESS : "";
    const wordFinal = i + 1 >= chars.length || !/\p{L}/u.test(chars[i + 1]);
    // a word's last ח with an a: the a comes first (ʁˈuaχ)
    if (c === "ח" && wordFinal && vowel === "a") chunks.push(`${stress}aχ`);
    else chunks.push((consonant === NONE ? "" : consonant) + stress + (vowel === NONE ? "" : vowel));
  });
  return chunks;
}

/** The key → value metadata of an ONNX file (ModelProto.metadata_props, field 14). */
export function onnxMetadata(b: Uint8Array): Record<string, string> {
  const varint = (i: number): [number, number] => {
    let v = 0;
    for (let m = 1; ; m *= 128) {
      const c = b[i++];
      v += (c & 127) * m;
      if (c < 128) return [v, i];
    }
  };
  /** the length-delimited fields in [lo, hi): [field number, start, end] */
  const fields = (lo: number, hi: number): [number, number, number][] => {
    const out: [number, number, number][] = [];
    for (let i = lo; i < hi; ) {
      const [tag, at] = varint(i);
      i = at;
      const wire = tag % 8;
      if (wire === 0) i = varint(i)[1];
      else if (wire === 1) i += 8;
      else if (wire === 5) i += 4;
      else if (wire === 2) {
        const [n, start] = varint(i);
        out.push([Math.floor(tag / 8), start, start + n]);
        i = start + n;
      } else throw new Error("not an ONNX file");
    }
    return out;
  };
  const text = new TextDecoder();
  const meta: Record<string, string> = {};
  for (const [f, lo, hi] of fields(0, b.length)) {
    if (f !== 14) continue;
    const kv = fields(lo, hi);
    const part = (n: number) => {
      const p = kv.find((x) => x[0] === n);
      return p ? text.decode(b.subarray(p[1], p[2])) : "";
    };
    meta[part(1)] = part(2);
  }
  return meta;
}

export function readMeta(m: Record<string, string>): Meta {
  if (!m.vocab || !m.consonant_vocab || !m.vowel_vocab) throw new Error("the model has no vocabulary");
  return {
    vocab: JSON.parse(m.vocab),
    consonants: JSON.parse(m.consonant_vocab),
    vowels: JSON.parse(m.vowel_vocab),
    cls: Number(m.cls_token_id),
    sep: Number(m.sep_token_id),
    allowed: JSON.parse(m.letter_consonant_constraints ?? "{}"),
    geresh: JSON.parse(m.geresh_map ?? "{}"),
  };
}

async function modelBytes(): Promise<Uint8Array> {
  // the Cache API where there is one (a private window may refuse): 20 MB fetched once
  const cache = await (async () => {
    try {
      return await caches.open(CACHE);
    } catch {
      return null;
    }
  })();
  const kept = await cache?.match(MODEL_URL);
  if (kept) return new Uint8Array(await kept.arrayBuffer());
  const res = await fetch(MODEL_URL);
  if (!res.ok) throw new Error(`Hebrew model ${res.status}`);
  const bytes = new Uint8Array(await res.arrayBuffer());
  cache?.put(MODEL_URL, new Response(bytes, { headers: { "Content-Type": "application/octet-stream" } })).catch(() => {});
  return bytes;
}

type Reader = (line: string) => Promise<string[]>;
let reader: Promise<Reader> | null = null;

function load(): Promise<Reader> {
  reader ??= (async (): Promise<Reader> => {
    const [ort, wasm, bytes] = await Promise.all([
      import("onnxruntime-web/wasm"),
      import("onnxruntime-web/ort-wasm-simd-threaded.wasm?url"),
      modelBytes(),
    ]);
    const meta = readMeta(onnxMetadata(bytes));
    ort.env.wasm.numThreads = 1; // threads need a cross-origin isolated page
    ort.env.wasm.wasmPaths = { wasm: wasm.default };
    const session = await ort.InferenceSession.create(bytes, { executionProviders: ["wasm"] });
    return async (line) => {
      const { chars, counted, ids } = modelInput(line, meta);
      const dims = [1, ids.length];
      const out = await session.run({
        input_ids: new ort.Tensor("int64", BigInt64Array.from(ids, BigInt), dims),
        attention_mask: new ort.Tensor("int64", new BigInt64Array(ids.length).fill(1n), dims),
      });
      return decode(chars, counted, meta, {
        consonant: out.consonant_logits.data as Float32Array,
        vowel: out.vowel_logits.data as Float32Array,
        stress: out.stress_logits.data as Float32Array,
      });
    };
  })();
  reader.catch(() => {
    reader = null; // allow a retry
  });
  return reader;
}

let queue: Promise<void> = Promise.resolve();

/**
 * Read the Hebrew lines of `text` that have no reading yet (loading the model on the first), so
 * `lyricsToSyllables` sings them. Rejects if the model can't be had.
 */
export function readHebrew(text: string): Promise<void> {
  if (!unreadHebrew(text).length) return Promise.resolve();
  const run = queue.then(async () => {
    // an earlier call may have read some
    for (const line of unreadHebrew(text)) setHebrewReading(line, await (await load())(line));
  });
  queue = run.then(
    () => {},
    () => {},
  );
  return run;
}
