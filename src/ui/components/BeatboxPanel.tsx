import { useEffect, useRef, useState } from "react";
import { useStore } from "../store.ts";
import { engine } from "../../audio/AudioEngine.ts";
import { BEAT_SOUNDS, BEAT_PRESETS, formatBeat, parseBeat, type Airstream } from "../../wasm/engine.ts";
import { Keys, MidiButton } from "./Keys.tsx";
import { ActionBar, PrimaryButton } from "./ActionBar.tsx";
import { Menu } from "./Controls.tsx";
import { Chip, Screen } from "./Screen.tsx";
import { EncoderPages } from "./Encoder.tsx";
import { Spectrogram } from "./Spectrogram.tsx";
import type { InfoFn, SlotsFn } from "./AnatomyView.tsx";
import { V } from "../colors.ts";
import { t } from "../i18n/index.ts";

const AIR: Record<Airstream, string> = {
  pulmonic: "Breath out",
  ejective: "Larynx piston",
  click: "Tongue suction",
  ingressive: "Breath in",
  voiced: "Voice",
};

const NOTE = ["C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B"];
const noteName = (m: number) => `${NOTE[((m % 12) + 12) % 12]}${Math.floor(m / 12) - 1}`;

/** Step under the playhead, polled from telemetry (re-renders only when it changes). */
export function useBeatStep(): number {
  const [step, setStep] = useState(-1);
  useEffect(() => {
    let raf = 0;
    const loop = () => {
      const s = engine.latest?.beatStep ?? -1;
      setStep((prev) => (prev === s ? prev : s));
      raf = requestAnimationFrame(loop);
    };
    raf = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(raf);
  }, []);
  return step;
}

/** Beatbox's readouts under the chin: the articulators a mouth drum uses */
export const beatSlots: SlotsFn = (_m, a, f) => [
  { label: t("Jaw"), value: a.jaw.toFixed(1), unit: "cm", color: V.red },
  { label: t("Lips"), value: a.lipAp < 0.05 ? t("Sealed") : a.lipWidth < 0.55 ? t("Round") : t("Open"), color: V.pink },
  { label: t("Larynx"), value: `${a.larynx >= 0 ? "+" : ""}${a.larynx.toFixed(1)}`, unit: "cm", color: V.green },
  { label: t("Soft palate"), value: f.velumPort > 0.02 ? t("Open") : t("Shut"), color: V.white },
];
/** the big symbol owns the top right, so nothing else is written there */
export const beatInfo: InfoFn = () => [];

function Pads() {
  const beatHit = useStore((s) => s.beatHit);
  const [lit, setLit] = useState<string | null>(null);
  const timer = useRef(0);
  const hit = (id: string) => {
    beatHit(id);
    setLit(id);
    clearTimeout(timer.current);
    timer.current = window.setTimeout(() => setLit(null), 160);
  };
  const hitRef = useRef(hit);
  hitRef.current = hit;
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.repeat || e.metaKey || e.ctrlKey || e.altKey) return;
      const t = e.target as HTMLElement;
      if (t && t.tagName === "SELECT") return;
      if (t && (t.tagName === "INPUT" || t.tagName === "TEXTAREA") && (t as HTMLInputElement).type !== "range" && (t as HTMLInputElement).type !== "checkbox") return;
      const i = "1234567890".indexOf(e.key);
      if (i >= 0 && i < BEAT_SOUNDS.length) {
        hitRef.current(BEAT_SOUNDS[i].id);
        e.preventDefault();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
  return (
    <div className="pads">
      {BEAT_SOUNDS.map((s, i) => (
        <button
          key={s.id}
          className={`pad pad-${s.role}${lit === s.id ? " lit" : ""}`}
          onPointerDown={() => hit(s.id)}
          onKeyDown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              hit(s.id);
            }
          }}
          title={t(s.hint)}
          aria-label={`${t(s.name)}: ${t(s.hint)}. ${t("Key")} ${(i + 1) % 10}`}
        >
          <span className="pad-sym">{s.id}</span>
          <span className="pad-name">{t(s.name)}</span>
          <span className="pad-air">{t(AIR[s.airstream])}</span>
          <span className="pad-key" aria-hidden="true">
            {(i + 1) % 10}
          </span>
        </button>
      ))}
    </div>
  );
}

function Sequencer() {
  const beat = useStore((s) => s.beat);
  const toggle = useStore((s) => s.toggleBeatStep);
  const now = useBeatStep();
  return (
    <div className="seq-grid" style={{ "--steps": beat.steps.length } as React.CSSProperties} role="grid" aria-label={t("Beat pattern: one sound per step")}>
      {BEAT_SOUNDS.map((s) => (
        <div className="seq-row" role="row" key={s.id}>
          <span className="seq-label" role="rowheader">
            <b>{s.id}</b> {t(s.name)}
          </span>
          {beat.steps.map((st, i) => {
            const on = st?.id === s.id;
            return (
              <button
                key={i}
                role="gridcell"
                className={`cell${on ? " on" : ""}${i === now ? " now" : ""}${i % 4 === 0 ? " downbeat" : ""}`}
                onClick={() => toggle(i, s.id)}
                aria-pressed={on}
                aria-label={`${t(s.name)}, ${t("step")} ${i + 1}`}
              >
                {on && s.id === "m" && st?.note !== undefined ? <span className="cell-note">{noteName(st.note)}</span> : null}
              </button>
            );
          })}
        </div>
      ))}
    </div>
  );
}

/** Beatbox's main button: play or stop the beat; the pattern and the close-mic effect beside it. */
function BeatActions() {
  const playing = useStore((s) => s.beat.playing);
  const preset = useStore((s) => s.beat.preset);
  const setPlaying = useStore((s) => s.setBeatPlaying);
  const loadPreset = useStore((s) => s.loadBeatPreset);
  const mic = useStore((s) => s.mic);
  const setMic = useStore((s) => s.setMic);
  const press = () => setPlaying(!playing);
  return (
    <ActionBar label={t("Beat")} onSpace={press}>
      <PrimaryButton label={t("Play beat")} playing={playing} onClick={press} />
      <Menu
        label={t("Pattern")}
        value={preset ?? ""}
        options={[...(preset ? [] : [{ value: "", label: t("Your beat"), disabled: true }]), ...BEAT_PRESETS.map((p) => ({ value: p.name, label: p.name }))]}
        onChange={(name) => loadPreset(name)}
      />
      {/* an effect on the sound, not the microphone: "Close mic" read as "shut the mic" */}
      <Chip on={mic} onClick={() => setMic(!mic)} title={t("Close-mic effect: the bass, presence and squash of a mic held to the lips")}>
        {t("Mic FX")}
      </Chip>
    </ActionBar>
  );
}

function Transport() {
  const beat = useStore((s) => s.beat);
  const setTempo = useStore((s) => s.setBeatTempo);
  const setSteps = useStore((s) => s.setBeatSteps);
  const [text, setText] = useState(() => formatBeat(beat.steps));
  const editing = useRef(false);
  useEffect(() => {
    if (!editing.current) setText(formatBeat(beat.steps));
  }, [beat.steps]);
  return (
    <div className="transport">
      <div className="transport-encoders">
        <EncoderPages
          label={t("Transport")}
          columns={2}
          pages={[
            {
              name: t("Tempo"),
              encoders: [
                { label: t("Tempo"), value: beat.bpm, min: 60, max: 180, onChange: (v) => setTempo({ bpm: Math.round(v) }), format: (v) => ({ value: String(Math.round(v)), unit: "bpm" }) },
                { label: t("Swing"), value: beat.swing, min: 0, max: 0.6, onChange: (v) => setTempo({ swing: v }), format: (v) => ({ value: String(Math.round(v * 100)), unit: "%" }) },
              ],
            },
          ]}
        />
      </div>
      <label className="sbn">
        <span className="dim">{t("Beat as text")}</span>
        <input
          value={text}
          title={t("B kick, t hat, Pf snare, K rimshot, ^K inward, ! click, m hum (m:43 sets its note), . rest")}
          placeholder="B . t . | Pf . t ."
          spellCheck={false}
          onFocus={() => (editing.current = true)}
          onBlur={() => {
            editing.current = false;
            setText(formatBeat(beat.steps));
          }}
          onChange={(e) => {
            setText(e.target.value);
            const steps = parseBeat(e.target.value);
            if (steps.length) setSteps(steps);
          }}
        />
      </label>
    </div>
  );
}

/** Beatbox mode's right column: the beat button, the step sequencer, the waterfall, the pads, tempo and the beat as text, the keys (hum bass). */
export function BeatboxRight() {
  return (
    <div className="right right-beatbox">
      <BeatActions />
      <Screen className="s-seq" title={t("Step sequencer")}>
        <Sequencer />
      </Screen>
      <Screen className="s-wf" title={t("Spectrogram")}>
        <Spectrogram />
      </Screen>
      <Screen className="s-pads" title={t("Pads")}>
        <Pads />
      </Screen>
      <Screen className="s-transport" title={t("Tempo")}>
        <Transport />
      </Screen>
      {/* the keys hum a bass note under the beat */}
      <Screen className="s-keys s-keys-compact" title={t("Bass")} tools={<MidiButton />}>
        <Keys />
      </Screen>
    </div>
  );
}

/** The sound being made, in big outlined letters over the face, fading after each hit. */
export function BeatOverlay() {
  const steps = useStore((s) => s.beat.steps);
  const playing = useStore((s) => s.beat.playing);
  const now = useBeatStep();
  const cell = now >= 0 ? steps[now] : null;
  if (!playing || !cell) return null;
  const s = BEAT_SOUNDS.find((x) => x.id === cell.id);
  return (
    <div className="beat-overlay" data-layout="overlay" aria-hidden="true" key={now}>
      <span className="beat-sym">{cell.id}</span>
      <span className="beat-name">{s ? t(s.name) : ""}</span>
      {s && <span className="beat-air">{t(AIR[s.airstream])}</span>}
      <span className="beat-step">
        {t("Step")} {String(now + 1).padStart(2, "0")} / {steps.length}
      </span>
    </div>
  );
}
