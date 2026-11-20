import { DIALECT_PRESETS, type DialectPreset, type Lang } from "../../wasm/engine.ts";
import { Menu } from "./Controls.tsx";
import { Chip } from "./Screen.tsx";

const LANGS: { value: Lang; label: string; name: string }[] = [
  { value: "en", label: "EN", name: "English" },
  { value: "tr", label: "TR", name: "Turkish" },
  { value: "es", label: "ES", name: "Spanish" },
];

/** The language typed words are read in: two chips for a screen's tools. `off` lights neither (IPA input). */
export function LangPick(props: { value: Lang; onChange: (l: Lang) => void; off?: boolean; dialect?: DialectPreset; onDialectChange?: (d: DialectPreset) => void }) {
  return (
    <div className="lang-pick" role="group" aria-label="Language">
      {LANGS.map((l) => (
        <Chip key={l.value} on={!props.off && props.value === l.value} onClick={() => props.onChange(l.value)} title={l.name} aria-label={l.name}>
          {l.label}
        </Chip>
      ))}
      {props.value === "es" && props.dialect && props.onDialectChange && (
        <Menu
          label="Accent"
          value={props.dialect}
          options={(Object.keys(DIALECT_PRESETS) as DialectPreset[]).map((d) => ({
            value: d,
            label: d === "neutro" ? "Neutro" : d === "rioplatense" ? "Rioplatense" : "Castellano",
          }))}
          onChange={props.onDialectChange}
        />
      )}
    </div>
  );
}
