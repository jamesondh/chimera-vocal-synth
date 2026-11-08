import { useEffect, useMemo, useRef, useState } from "react";
import { useStore } from "../store.ts";
import { stopSpeaking, spokenWordNow } from "../speech.ts";
import { currentReading, letterAt, onReading, readArabic, soundingNow, speakArabic, type ArabicReading } from "../arabic.ts";
import { LETTER_FORMS, MAKHARIJ, PRESET_GROUPS, SIFAT, letterOf, letterText, type Letter, type LetterForm } from "../arabicData.ts";
import { mouthWords } from "../mouthWords.ts";
import { ARABIC } from "../render/vector.ts";
import { V } from "../colors.ts";
import { Slider, Toggle } from "./Controls.tsx";
import { Chip, Screen } from "./Screen.tsx";
import { ScoreView } from "./ScoreView.tsx";
import type { SlotsFn } from "./AnatomyView.tsx";

/** the Arabic word being said, at the mouth */
export const arabicWords = mouthWords(spokenWordNow, V.white, 1, ARABIC);

/** where the sound being made is made (the chart's point of articulation), in short */
function makhrajNow(): string | null {
  const s = soundingNow();
  const r = currentReading();
  if (!s || !r) return null;
  const ch = letterAt(r.text, s.source);
  const l = ch ? letterOf(ch) : undefined;
  if (!l) return null;
  return MAKHARIJ.find((m) => m.letters.includes(l))?.short ?? null;
}

/** Arabic's readouts under the chin: the sound, where it is made, jaw, soft palate */
export const arabicSlots: SlotsFn = (_m, a, f) => {
  const s = soundingNow();
  return [
    s ? { label: "Sound", value: s.ipa, color: V.green, ipa: true } : { label: "Sound", value: "-", color: V.green },
    { label: "Made at", value: (s && makhrajNow()) ?? "-", color: V.pink },
    { label: "Jaw", value: a.jaw.toFixed(1), unit: "cm", color: V.red },
    { label: "Soft palate", value: f.velumPort > 0.02 ? "Open" : "Shut", color: V.white },
  ];
};

/** The sounding segment, polled (it changes with the audio clock, not with React state). */
function useSounding(): ReturnType<typeof soundingNow> {
  const [now, setNow] = useState<ReturnType<typeof soundingNow>>(null);
  const last = useRef<string>("");
  useEffect(() => {
    const id = setInterval(() => {
      const s = soundingNow();
      const key = s ? `${s.index}` : "";
      if (key !== last.current) {
        last.current = key;
        setNow(s);
      }
    }, 40);
    return () => clearInterval(id);
  }, []);
  return now;
}

/** Arabic text with the letters being said lit. */
function ArabicText(props: { text: string; span: [number, number] | null }) {
  const chars = useMemo(() => Array.from(props.text), [props.text]);
  const [a, b] = props.span ?? [0, 0];
  return (
    <p className="ar-text" dir="rtl" lang="ar">
      {props.span ? (
        <>
          {chars.slice(0, a).join("")}
          <span className="ar-now">{chars.slice(a, b).join("")}</span>
          {chars.slice(b).join("")}
        </>
      ) : (
        props.text
      )}
    </p>
  );
}

/** The rules behind the sound being made, in words */
const RULE_NAMES: Record<string, string> = {
  madd: "Madd: a long vowel, two counts",
  "madd-muttasil": "Madd muttaṣil: a hamza follows in the word",
  "madd-munfasil": "Madd munfaṣil: the next word starts with a hamza",
  "madd-lazim": "Madd lāzim: a doubled or still letter follows",
  "madd-arid": "Madd ʿāriḍ: before the last letter, in pause",
  "madd-sign": "The text marks this madd",
  shadda: "Shadda: a doubled consonant",
  ghunna: "Ghunna: held through the nose",
  tanwin: "Tanwīn: the n of nunation",
  "nun-sakin": "Nūn sākin before a consonant",
  "mim-sakin": "Mīm sākin before a consonant",
  iqlab: "Iqlāb",
  qalqala: "Qalqala letter without a vowel",
  tafkhim: "Tafkhīm: heavy, the tongue drawn back",
  tarqiq: "Tarqīq: light",
  allah: "The lām of the name Allāh",
  qamariyya: "The article before a moon letter",
  shamsiyya: "A sun letter: the article's lām joins it",
  "hamzat-wasl": "Hamzat al-waṣl, said at the start",
  "helping-vowel": "A vowel joining two words",
  shortened: "A long vowel shortened before a still letter",
  waqf: "Waqf: the pause form",
  "taa-marbuta": "Tāʾ marbūṭa",
  guessed: "No vowel written: guessed",
};

/** Arabic's right column: what to read, the presets, the letters by where they are made, the score. */
export function ArabicRight() {
  const body = useStore((s) => s.body);
  const f0 = useStore((s) => s.source.f0);
  const start = useStore((s) => s.start);
  const [text, setText] = useState(PRESET_GROUPS[4].presets[0].text);
  const [waqf, setWaqf] = useState(true);
  const [rate, setRate] = useState(1);
  const [group, setGroup] = useState(PRESET_GROUPS[0].id);
  const [form, setForm] = useState<LetterForm>("fatha");
  const [picked, setPicked] = useState<Letter>(MAKHARIJ[4].letters[0]);
  const [reading, setReading] = useState<ArabicReading | null>(currentReading());
  useEffect(() => onReading(setReading), []);
  const now = useSounding();

  // the reading of what is typed, as it is typed
  const preview = useMemo(() => {
    try {
      const ps = readArabic(text, { waqf });
      return { ipa: ps.map((p) => p.ipa).join(" | "), guessed: ps.reduce((n, p) => n + p.guessed, 0) };
    } catch {
      return { ipa: "", guessed: 0 };
    }
  }, [text, waqf]);

  const say = async (t: string, w = waqf) => {
    await start();
    speakArabic(t, { body, rate, f0, waqf: w });
  };
  const playLetter = (l: Letter) => {
    setPicked(l);
    const { text: t, waqf: w } = letterText(l, form);
    void say(t, w);
  };

  const live = reading && now ? reading : null;
  const span = live && live.text === text ? now!.source : null;
  const nowLetter = live && now ? letterOf(letterAt(live.text, now.source) ?? "") : undefined;
  const presets = PRESET_GROUPS.find((g) => g.id === group)!;

  return (
    <div className="right right-arabic">
      <Screen className="s-read" title="Read" sub="Vowelled Arabic" tools={<Toggle label="Pause forms" checked={waqf} onChange={setWaqf} />}>
        <div className="say ar-say">
          <div className="say-row">
            <textarea
              dir="rtl"
              lang="ar"
              value={text}
              onChange={(e) => setText(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) void say(text);
                e.stopPropagation();
              }}
              rows={2}
              spellCheck={false}
              aria-label="Arabic text to read"
              placeholder="اكتب نصًّا مشكولًا"
            />
            <div className="say-actions">
              <button className="primary" onClick={() => void say(text)}>
                Speak
              </button>
              <button className="ghost" onClick={stopSpeaking}>
                Stop
              </button>
            </div>
          </div>
          <ArabicText text={text} span={span} />
          <div className="say-row">
            <p className="transcription" aria-live="polite" title={preview.ipa}>
              {preview.ipa ? `/${preview.ipa}/` : "The reading appears here."}
            </p>
            <div className="say-rate">
              <Slider label="Rate" value={rate} min={0.5} max={1.5} onChange={setRate} format={(v) => `${v.toFixed(2)}×`} />
            </div>
          </div>
          <p className="ar-why" aria-live="polite">
            {now && now.rules.length
              ? `${now.ipa}: ${now.rules.map((r) => RULE_NAMES[r] ?? r).join(" · ")}`
              : preview.guessed
                ? `${preview.guessed} letter${preview.guessed > 1 ? "s have" : " has"} no vowel mark: read with a guess. Add the marks for an exact reading.`
                : "Each sound is shaped by the rules of reading: they show here as it is said."}
          </p>
        </div>
      </Screen>
      <Screen
        className="s-presets"
        title="Presets"
        tools={
          <div className="ar-groups" role="group" aria-label="Preset group">
            {PRESET_GROUPS.map((g) => (
              <Chip key={g.id} on={g.id === group} onClick={() => setGroup(g.id)}>
                {g.title}
              </Chip>
            ))}
          </div>
        }
      >
        <div className="ar-presets" role="group" aria-label={presets.title}>
          {presets.presets.map((p) => (
            <button
              key={p.text}
              type="button"
              className="ar-preset"
              title={p.en}
              onClick={() => {
                setText(p.text);
                void say(p.text);
              }}
            >
              <span className="ar-glyphs" dir="rtl" lang="ar">
                {p.text}
              </span>
              <span className="ar-gloss">{p.en}</span>
            </button>
          ))}
        </div>
      </Screen>
      <Screen
        className="s-letters"
        title="Letters"
        sub="Throat to lips"
        tools={
          <div className="ar-forms" role="group" aria-label="How a letter is said">
            {LETTER_FORMS.map((f) => (
              <Chip key={f.id} on={f.id === form} onClick={() => setForm(f.id)} title={f.title} className={f.id === "name" ? undefined : "ar-chip"}>
                {f.label}
              </Chip>
            ))}
          </div>
        }
      >
        <div className="ar-chart">
          {MAKHARIJ.map((m) => (
            <div key={m.id} className="ar-makhraj" title={m.where}>
              <div className="ar-makhraj-head">
                <span className="ar-makhraj-ar" lang="ar">
                  {m.ar}
                </span>
                <span className="ar-makhraj-en">{m.en}</span>
              </div>
              <div className="ar-letters">
                {m.letters.map((l) => (
                  <button key={l.id} type="button" className={`ar-letter${l === nowLetter ? " now" : ""}${l === picked ? " picked" : ""}`} onClick={() => playLetter(l)} title={`${l.latin} /${l.ipa}/`} aria-label={`${l.latin}, ${l.ipa}`}>
                    <span className="ar-letter-glyph" lang="ar">
                      {l.madd ? `ـ${l.char}` : l.char}
                    </span>
                    <span className="ar-letter-ipa">{l.ipa}</span>
                  </button>
                ))}
              </div>
            </div>
          ))}
        </div>
        <div className="ar-letter-info" aria-live="polite">
          <span className="ar-letter-name">
            <span lang="ar">{picked.name}</span> {picked.latin} /{picked.ipa}/
          </span>
          <span className="ar-sifat">
            {picked.sifat.map((s) => (
              <span key={s} className="ar-sifa" title={SIFAT[s].hint}>
                {SIFAT[s].en}
              </span>
            ))}
          </span>
        </div>
      </Screen>
      <Screen className="s-score" title="Gestural score">
        <ScoreView />
      </Screen>
    </div>
  );
}
