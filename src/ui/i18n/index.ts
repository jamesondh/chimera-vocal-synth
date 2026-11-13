/**
 * The UI's texts under one control: `t()` renders a string in the page language. Keys are the
 * exact English strings as they appear in the code (the code's language stays English); a key
 * missing from the dictionary falls back to English, so the translation can grow piecemeal.
 * The Spanish dictionary lives in `es.ts` — that file is the whole control panel for the texts.
 *
 * The engine's phonetics stay English (CMUdict, IPA, example sentences, demo lyrics): this file
 * only touches what is written on screen, never what is spoken.
 */
import { create } from "zustand";
import { ES } from "./es.ts";

export type Lang = "es" | "en";

const STORAGE_KEY = "chimera.lang";
const LANGS: Lang[] = ["es", "en"];

/** `?lang=es|en` wins (shareable links, the e2e run), then the saved choice, then English. */
function initialLang(): Lang {
  try {
    const asked = new URLSearchParams(window.location.search).get("lang") as Lang | null;
    if (asked && LANGS.includes(asked)) return asked;
    const saved = localStorage.getItem(STORAGE_KEY) as Lang | null;
    if (saved && LANGS.includes(saved)) return saved;
  } catch {
    /* no persistence (private mode): the default still applies */
  }
  return "en";
}

interface LangStore {
  lang: Lang;
  setLang: (lang: Lang) => void;
}

export const useLangStore = create<LangStore>((set) => ({
  lang: initialLang(),
  setLang: (lang) => {
    document.documentElement.lang = lang;
    try {
      localStorage.setItem(STORAGE_KEY, lang);
    } catch {
      /* ignore */
    }
    set({ lang });
  },
}));

/** The page language; a component that renders `t()` calls this so a change re-renders it. */
export const useLang = (): Lang => useLangStore((s) => s.lang);

/** Set the page language (persisted; canvas views redraw per frame and pick it up at once). */
export const setLang = (lang: Lang): void => useLangStore.getState().setLang(lang);

/** The other language of the two (for the top bar's toggle). */
export const otherLang = (lang: Lang): Lang => (lang === "es" ? "en" : "es");

/** An English string in the page language; missing keys fall back to English. */
export function t(s: string): string {
  if (useLangStore.getState().lang === "en") return s;
  return ES[s] ?? s;
}
