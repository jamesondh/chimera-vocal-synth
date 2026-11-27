import type { ArabicDialect, Lang } from "../../wasm/engine.ts";
import { Chip } from "./Screen.tsx";
import { Menu } from "./Controls.tsx";

const LANGS: { value: Lang; label: string; name: string }[] = [
  { value: "en", label: "EN", name: "English" },
  { value: "tr", label: "TR", name: "Turkish" },
  { value: "ar", label: "AR", name: "Arabic" },
];

/** The language typed words are read in: two chips for a screen's tools. `off` lights neither (IPA input). */
export function LangPick(props: { value: Lang; onChange: (l: Lang) => void; off?: boolean; arabicDialect?: ArabicDialect; onArabicDialect?: (d: ArabicDialect) => void }) {
  return (
    <div className="lang-pick" role="group" aria-label="Language">
      {LANGS.map((l) => (
        <Chip key={l.value} on={!props.off && props.value === l.value} onClick={() => props.onChange(l.value)} title={l.name} aria-label={l.name}>
          {l.label}
        </Chip>
      ))}
      {props.value === "ar" && props.arabicDialect && props.onArabicDialect && (
        <Menu label="Arabic pronunciation" value={props.arabicDialect} options={[{ value: "msa", label: "Standard Arabic" }, { value: "egyptian", label: "Egyptian" }]} onChange={(d) => props.onArabicDialect?.(d as ArabicDialect)} />
      )}
    </div>
  );
}
