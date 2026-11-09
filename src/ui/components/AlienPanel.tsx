import { useStore } from "../store.ts";
import { ALIEN_PRESETS, type AlienShape } from "../../wasm/engine.ts";
import { speakText } from "../speech.ts";
import type { EncoderPage } from "./Encoder.tsx";
import { Chip } from "./Screen.tsx";

const times = (v: number) => ({ value: v.toFixed(2), unit: "×" });
const pct = (v: number) => (v <= 0 ? { value: "None", unit: "" } : { value: String(Math.round(v * 100)), unit: "%" });
const semis = (v: number) => ({ value: `${v >= 0 ? "+" : ""}${Math.round(v)}`, unit: "st" });

/** Alien mode's screen (in place of the vowel map, which holds only for human bodies): the aliens to meet. */
export function AlienPanel() {
  const alienId = useStore((s) => s.alienId);
  const meetAlien = useStore((s) => s.meetAlien);
  const p = ALIEN_PRESETS.find((x) => x.id === alienId);
  return (
    <div className="alien-panel">
      <div className="alien-presets" role="group" aria-label="Aliens">
        {ALIEN_PRESETS.map((a) => (
          <Chip key={a.id} on={a.id === alienId} onClick={() => meetAlien(a.id)} title={a.blurb}>
            {a.name}
          </Chip>
        ))}
      </div>
      <p className="alien-blurb">{p ? p.blurb : "Your own alien: shape it with the Alien body, Organs and Mouths pages."}</p>
      {p && (
        <Chip
          toggle={false}
          onClick={() => {
            const s = useStore.getState();
            void s.start().then(() => speakText(p.line, { ipa: false, body: s.body, rate: p.rate, f0: s.source.f0 }));
          }}
        >
          Say “{p.line}”
        </Chip>
      )}
    </div>
  );
}

/** The alien's shape as encoder pages, ahead of the voice's own (empty for a human). */
export function useAlienPages(): EncoderPage[] {
  const a = useStore((s) => s.alien);
  const setAlien = useStore((s) => s.setAlien);
  if (!a) return [];
  const on = (k: keyof AlienShape) => (v: number) => setAlien({ ...a, [k]: v });
  return [
    {
      name: "Alien body",
      encoders: [
        { label: "Size", value: a.size, min: 0.6, max: 2.5, log: true, onChange: on("size"), format: times },
        { label: "Throat", value: a.throat, min: 0.6, max: 1.8, onChange: on("throat"), format: times },
        { label: "Lips", value: a.lips, min: 0.5, max: 2, onChange: on("lips"), format: times },
        { label: "Voice pitch", value: a.pitch, min: 0.25, max: 4, log: true, onChange: on("pitch"), format: times },
      ],
    },
    {
      name: "Organs",
      encoders: [
        { label: "Throat sac", value: a.sac, min: 0, max: 1, onChange: on("sac"), format: pct },
        { label: "2nd voicebox", value: a.twin, min: 0, max: 1, onChange: on("twin"), format: pct },
        { label: "Interval", value: a.interval, min: -24, max: 24, onChange: (v) => on("interval")(Math.round(v)), format: semis },
        { label: "Crest", value: a.crest, min: 0, max: 1, onChange: on("crest"), format: pct },
      ],
    },
    {
      name: "Mouths",
      encoders: [
        { label: "Mouths", value: a.mouths, min: 1, max: 2, onChange: (v) => on("mouths")(Math.round(v)), format: (v) => ({ value: String(Math.round(v)), unit: "" }) },
        // with one mouth, the second's interval shows as off
        { label: "2nd mouth", value: a.mouth2, min: -24, max: 24, onChange: (v) => on("mouth2")(Math.round(v)), format: (v) => (Math.round(a.mouths) >= 2 ? semis(v) : { value: "Off", unit: "" }) },
      ],
    },
  ];
}
