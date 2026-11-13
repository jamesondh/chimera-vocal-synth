import { useEffect, useMemo, useState } from "react";
import { skippedLines } from "../skipped.ts";
import { useStore } from "../store.ts";
import { speakText, speakPhrases, stopSpeaking, onPlan, currentPlan, spokenWordNow, speakingNow } from "../speech.ts";
import { mouthWords } from "../mouthWords.ts";
import { tokenizeIpa, DIALECT_PRESETS, type DialectPreset } from "../../wasm/engine.ts";
import { setVoiceLang, currentVoiceLang, type VoiceLang } from "../speech.ts";
import { Menu, Slider, Segmented } from "./Controls.tsx";
import { Chip, Screen } from "./Screen.tsx";
import { ActionBar, PrimaryButton } from "./ActionBar.tsx";
import { ScoreView, planNow } from "./ScoreView.tsx";
import { TubeView } from "./TubeView.tsx";
import { Stroboscope } from "./Stroboscope.tsx";
import type { InfoFn, SlotsFn } from "./AnatomyView.tsx";
import { V } from "../colors.ts";
import { engine } from "../../audio/AudioEngine.ts";
import { t, t as tt } from "../i18n/index.ts";

// the examples stay English on purpose: the engine speaks English (CMUdict + letter-to-sound)
const EXAMPLES = ["Hello. Can you hear me?", "She sells sea shells by the sea shore.", "Every sound here is air moving through a throat.", "The quick brown fox jumps over the lazy dog."];

// IPA pulmonic consonant chart (rows = manner, columns = place); empty cells are null
const PLACES = ["Bilabial", "Labiodental", "Dental", "Alveolar", "Postalveolar", "Retroflex", "Palatal", "Velar", "Uvular", "Pharyngeal", "Glottal"];
/** short headings for the chart's narrow columns (the full names are the cells' titles) */
const PLACE_SHORT = ["Bilab", "Labdent", "Dental", "Alveol", "Postalv", "Retrofl", "Palatal", "Velar", "Uvular", "Pharyn", "Glottal"];
const MANNER_SHORT: Record<string, string> = { "Lateral fricative": "Lat fric", Approximant: "Approx" };
const CHART: [string, (string | null)[][]][] = [
  ["Plosive", [["p", "b"], [null, null], [null, null], ["t", "d"], [null, null], ["ʈ", "ɖ"], ["c", "ɟ"], ["k", "g"], ["q", "ɢ"], [null, null], ["ʔ", null]]],
  ["Nasal", [[null, "m"], [null, "ɱ"], [null, null], [null, "n"], [null, null], [null, "ɳ"], [null, "ɲ"], [null, "ŋ"], [null, "ɴ"], [null, null], [null, null]]],
  ["Trill", [[null, "ʙ"], [null, null], [null, null], [null, "r"], [null, null], [null, null], [null, null], [null, null], [null, "ʀ"], [null, null], [null, null]]],
  ["Tap", [[null, null], [null, null], [null, null], [null, "ɾ"], [null, null], [null, "ɽ"], [null, null], [null, null], [null, null], [null, null], [null, null]]],
  ["Fricative", [["ɸ", "β"], ["f", "v"], ["θ", "ð"], ["s", "z"], ["ʃ", "ʒ"], ["ʂ", "ʐ"], ["ç", "ʝ"], ["x", "ɣ"], ["χ", "ʁ"], ["ħ", "ʕ"], ["h", "ɦ"]]],
  ["Lateral fricative", [[null, null], [null, null], [null, null], ["ɬ", "ɮ"], [null, null], [null, null], [null, null], [null, null], [null, null], [null, null], [null, null]]],
  ["Approximant", [[null, null], [null, "ʋ"], [null, null], [null, "ɹ"], [null, null], [null, "ɻ"], [null, "j"], [null, "ɰ"], [null, null], [null, null], [null, null]]],
  ["Lateral", [[null, null], [null, null], [null, null], [null, "l"], [null, null], [null, "ɭ"], [null, "ʎ"], [null, null], [null, null], [null, null], [null, null]]],
];

/** the segment sounding now in the current utterance */
function soundingSegment(): string | null {
  const sp = currentPlan();
  const now = planNow(sp);
  if (!sp || now < 0) return null;
  return sp.plan.segments.find((s) => now >= s.t0 && now < s.t1)?.ipa ?? null;
}

/** Speak mode's readouts under the chin: the sound being made, jaw, lip shape, soft palate */
/** the word being spoken, at the mouth */
export const speechWords = mouthWords(spokenWordNow, V.white);

export const speechSlots: SlotsFn = (_m, a, f) => {
  const sound = soundingSegment();
  const lips = a.lipAp < 0.05 ? "Closed" : a.lipWidth < 0.55 ? "Round" : a.lipAp > 1.3 ? "Open" : "Spread";
  return [
    sound ? { label: t("Sound"), value: sound, color: V.green, ipa: true } : { label: t("Sound"), value: "-", color: V.green },
    { label: t("Jaw"), value: a.jaw.toFixed(1), unit: "cm", color: V.red },
    { label: t("Lips"), value: t(lips), color: V.pink },
    { label: t("Soft palate"), value: f.velumPort > 0.02 ? t("Open") : t("Shut"), color: V.white },
  ];
};

/** what the voice is doing: voiced or not, where the noise is, the lungs */
export const speechInfo: InfoFn = (m, _f, tele) => {
  if (!tele) return [];
  const t = tele.t;
  const out = [t.voicing > 0.3 ? `${tt("Voiced")} ${t.f0.toFixed(0)} Hz` : tt("Voiceless")];
  if (t.noiseLevel > 1 && t.noiseNode >= 0) {
    const s = m.branchS(t.noiseNode, t.phLen, t.orLen);
    if (s >= 0) out.push(`${tt("Noise at")} ${s.toFixed(1)} cm`);
  }
  out.push(`${tt("Lungs")} ${t.pLung.toFixed(1)} cmH₂O`);
  return out;
};

function ConsonantChart(props: { onPad: (c: string) => void }) {
  const [now, setNow] = useState<string | null>(null);
  useEffect(() => {
    const id = setInterval(() => setNow(engine.latest ? soundingSegment() : null), 60);
    return () => clearInterval(id);
  }, []);
  return (
    <div className="chart-scroll">
      <table className="ipa-chart">
        <thead>
          <tr>
            <th />
            {PLACES.map((p, i) => (
              <th key={p} scope="col" title={t(p)} aria-label={t(p)}>
                {t(PLACE_SHORT[i])}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {CHART.map(([manner, cells]) => (
            <tr key={manner}>
              <th scope="row" title={t(manner)} aria-label={t(manner)}>
                {t(MANNER_SHORT[manner] ?? manner)}
              </th>
              {cells.map((pair, i) => (
                <td key={i}>
                  {pair.map((sym, j) =>
                    sym ? (
                      <button key={j} className={`ipa${sym === now ? " now" : ""}`} onClick={() => props.onPad(sym)} title={t("Say it between two vowels")}>
                        {sym}
                      </button>
                    ) : (
                      <span key={j} className="ipa-empty" />
                    ),
                  )}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

/** Whether speech is sounding, polled each frame (re-renders only when it changes). */
function useSpeaking(): boolean {
  const [on, setOn] = useState(false);
  useEffect(() => {
    let raf = 0;
    const loop = () => {
      setOn(speakingNow());
      raf = requestAnimationFrame(loop);
    };
    raf = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(raf);
  }, []);
  return on;
}

/** Speak mode's right column: the speak button, what to say, the gestural score, the consonant chart, the tube and folds. */
export function SpeakRight() {
  const body = useStore((s) => s.body);
  const f0 = useStore((s) => s.source.f0);
  const start = useStore((s) => s.start);
  const [text, setText] = useState(EXAMPLES[0]);
  const [ipa, setIpa] = useState(false);
  const [rate, setRate] = useState(1);
  const [busy, setBusy] = useState(false);
  const [trans, setTrans] = useState(currentPlan()?.transcription ?? "");
  const [vowel, setVowel] = useState("ɑ");
  // the page's voice language (English by default: nothing changes until it is switched)
  const [lang, setLang] = useState<VoiceLang>(currentVoiceLang());
  const [dialect, setDialect] = useState<DialectPreset>("neutro");
  const speaking = useSpeaking();
  useEffect(() => onPlan((p) => setTrans(p?.transcription ?? "")), []);
  // in IPA mode: the symbols in the box that the voice will not say, updated as the text changes
  const skipped = useMemo(() => (ipa ? skippedLines(text) : []), [ipa, text]);

  const pickLang = (l: VoiceLang) => {
    setLang(l);
    setVoiceLang(l, l === "es" ? DIALECT_PRESETS[dialect] : undefined);
  };
  const pickDialect = (d: DialectPreset) => {
    setDialect(d);
    setVoiceLang("es", DIALECT_PRESETS[d]);
  };

  const speak = async () => {
    setBusy(true);
    try {
      await start();
      // the audio didn't start (the error shows on the bar): speaking would only light Stop
      if (!useStore.getState().started) return;
      await speakText(text, { ipa, body, rate, f0 });
    } finally {
      setBusy(false);
    }
  };
  const pad = async (c: string) => {
    await start();
    if (!useStore.getState().started) return;
    speakPhrases([{ segments: tokenizeIpa(`ˈ${vowel}${c}${vowel}`), intonation: "flat" }], { body, rate, f0 });
  };

  const press = () => (speaking ? stopSpeaking() : void speak());

  return (
    <div className="right right-speak">
      <ActionBar label={t("Speak")} onSpace={press}>
        <PrimaryButton label={t("Speak")} playing={speaking} busy={busy ? t("Preparing…") : null} onClick={press} />
        {/* puts an example in the box: the menu always reads Examples, since the box shows the words */}
        <Menu
          label={t("Examples")}
          value=""
          options={[{ value: "", label: t("Examples"), disabled: true }, ...EXAMPLES.map((ex) => ({ value: ex, label: ex }))]}
          onChange={(ex) => {
            setText(ex);
            setIpa(false);
          }}
        />
        <div className="act-slider">
          <Slider label={t("Rate")} value={rate} min={0.5} max={1.8} onChange={setRate} format={(v) => `${v.toFixed(2)}×`} />
        </div>
        <Chip on={ipa} onClick={() => setIpa(!ipa)}>
          {t("Text is IPA")}
        </Chip>
        <Segmented
          label={t("Voice")}
          value={lang}
          options={[
            { value: "en" as const, label: t("English") },
            { value: "es" as const, label: t("Spanish") },
          ]}
          onChange={pickLang}
        />
      </ActionBar>
      <Screen className="s-say" label={t("What to say")}>
        <div className="say">
          <div className="say-row">
            <textarea
              value={text}
              onChange={(e) => setText(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) void speak();
                e.stopPropagation();
              }}
              rows={2}
              spellCheck={!ipa}
              aria-label={ipa ? t("IPA to speak") : t("Text to speak")}
              placeholder={ipa ? t("Type IPA, for example ˈhɛloʊ") : t("Type something for the voice to say")}
            />
          </div>
          <div className="say-row">
            <div className="say-read">
              <p className="transcription" aria-live="polite">
                {trans ? `/${trans}/` : t("The transcription appears here.")}
              </p>
              <div className="say-skipped" aria-live="polite">
                {skipped.map((l) => (
                  <p key={l.what}>
                    {l.what}: <span className="say-skipped-sym">{l.symbols.join(" ")}</span>
                  </p>
                ))}
              </div>
            </div>
          </div>
          {lang === "es" && (
            <div className="examples" role="group" aria-label={t("Dialect")}>
              {(Object.keys(DIALECT_PRESETS) as DialectPreset[]).map((d) => (
                <Chip key={d} on={dialect === d} onClick={() => pickDialect(d)}>
                  {d === "neutro" ? t("Neutro") : d === "rioplatense" ? t("Rioplatense") : t("Castellano")}
                </Chip>
              ))}
            </div>
          )}
        </div>
      </Screen>
      <Screen className="s-score" title={t("Gestural score")}>
        <ScoreView />
      </Screen>
      <Screen
        className="s-chart"
        title={t("Consonants")}
        sub={t("Between two vowels")}
        tools={
          <div className="vowel-pick" role="group" aria-label={t("Vowel around it")}>
            {["ɑ", "i", "u", "ə"].map((v) => (
              <Chip key={v} on={vowel === v} onClick={() => setVowel(v)} className="ipa-chip">
                {v}
              </Chip>
            ))}
          </div>
        }
      >
        <ConsonantChart onPad={(c) => void pad(c)} />
      </Screen>
      <Screen className="s-tube" title={t("Tube")} sub={t("Glottis to lips")}>
        <TubeView />
      </Screen>
      <Screen className="s-folds" title={t("Vocal folds")} sub={t("Slowed 100×")}>
        <Stroboscope />
      </Screen>
    </div>
  );
}
