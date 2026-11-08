/**
 * The Arabic page's content: the letters by their points of articulation (makhārij, as tajweed
 * teaches them, from the deepest throat to the lips), each letter's qualities (ṣifāt), and the
 * words and sentences to play. The sounds come from the engine's Arabic reader; nothing here is
 * pronunciation, only what to say.
 */

/** A quality of a letter (ṣifa), as tajweed names it. */
export interface Sifa {
  ar: string;
  en: string;
  /** what it means for the sound */
  hint: string;
}

export const SIFAT = {
  jahr: { ar: "جَهْر", en: "Jahr", hint: "the sound is held: no breath runs with it" },
  hams: { ar: "هَمْس", en: "Hams", hint: "breath runs with the sound" },
  shidda: { ar: "شِدَّة", en: "Shidda", hint: "the sound stops: a closure" },
  tawassut: { ar: "تَوَسُّط", en: "Tawassuṭ", hint: "between: the sound neither stops nor runs freely" },
  rakhawa: { ar: "رَخَاوَة", en: "Rakhāwa", hint: "the sound runs: a fricative" },
  istila: { ar: "اِسْتِعْلَاء", en: "Istiʿlāʾ", hint: "the back of the tongue rises: a heavy letter" },
  istifal: { ar: "اِسْتِفَال", en: "Istifāl", hint: "the tongue stays low: a light letter" },
  itbaq: { ar: "إِطْبَاق", en: "Iṭbāq", hint: "the tongue covers the palate" },
  infitah: { ar: "اِنْفِتَاح", en: "Infitāḥ", hint: "the tongue leaves the palate open" },
  idhlaq: { ar: "إِذْلَاق", en: "Idhlāq", hint: "made with the lips or the tongue's tip: fluent" },
  ismat: { ar: "إِصْمَات", en: "Iṣmāt", hint: "made further in" },
  safir: { ar: "صَفِير", en: "Ṣafīr", hint: "a whistle" },
  qalqala: { ar: "قَلْقَلَة", en: "Qalqala", hint: "an echo when it has no vowel" },
  lin: { ar: "لِين", en: "Līn", hint: "eased: wāw and yāʾ without a vowel after fatḥa" },
  inhiraf: { ar: "اِنْحِرَاف", en: "Inḥirāf", hint: "the tongue turns aside" },
  takrir: { ar: "تَكْرِير", en: "Takrīr", hint: "the tip tends to repeat: kept small" },
  tafashshi: { ar: "تَفَشِّي", en: "Tafashshī", hint: "the air spreads through the mouth" },
  istitala: { ar: "اِسْتِطَالَة", en: "Istiṭāla", hint: "the sound extends along the tongue's edge" },
  ghunna: { ar: "غُنَّة", en: "Ghunna", hint: "a sound through the nose" },
} satisfies Record<string, Sifa>;

export type SifaId = keyof typeof SIFAT;

/** A letter as the chart shows and plays it. */
export interface Letter {
  /** unique within the chart (wāw and yāʾ appear as consonants and as madd letters) */
  id: string;
  char: string;
  /** its name, vowelled (said by the "Name" form) */
  name: string;
  /** its name in Latin letters */
  latin: string;
  /** its usual sound, in IPA */
  ipa: string;
  sifat: SifaId[];
  /** the madd letters have their own forms: the long vowel after a hamza */
  madd?: string;
}

/** A point of articulation and its letters. */
export interface Makhraj {
  id: string;
  ar: string;
  en: string;
  /** one word, for the readout under the chin */
  short: string;
  /** where on the head, in words */
  where: string;
  letters: Letter[];
}

const L = (id: string, char: string, name: string, latin: string, ipa: string, sifat: SifaId[], madd?: string): Letter => ({ id, char, name, latin, ipa, sifat, madd });

export const MAKHARIJ: Makhraj[] = [
  {
    id: "jawf",
    short: "Mouth",
    ar: "الجَوْف",
    en: "Open mouth",
    where: "the space of the throat and mouth: the long vowels",
    letters: [L("alif", "ا", "أَلِف", "alif", "aː", ["jahr", "rakhawa", "istifal", "infitah", "ismat"], "آ"), L("waw-madd", "و", "وَاو", "wāw", "uː", ["jahr", "rakhawa", "istifal", "infitah", "ismat"], "أُو"), L("ya-madd", "ي", "يَاء", "yāʾ", "iː", ["jahr", "rakhawa", "istifal", "infitah", "ismat"], "إِي")],
  },
  {
    id: "throat-deep",
    short: "Glottis",
    ar: "أَقْصَى الحَلْق",
    en: "Deep throat",
    where: "the glottis",
    letters: [L("hamza", "ء", "هَمْزَة", "hamza", "ʔ", ["jahr", "shidda", "istifal", "infitah", "ismat"]), L("ha", "ه", "هَاء", "hāʾ", "h", ["hams", "rakhawa", "istifal", "infitah", "ismat"])],
  },
  {
    id: "throat-mid",
    short: "Pharynx",
    ar: "وَسَط الحَلْق",
    en: "Mid throat",
    where: "the pharynx: the tongue root against its back wall",
    letters: [L("ayn", "ع", "عَيْن", "ʿayn", "ʕ", ["jahr", "tawassut", "istifal", "infitah", "ismat"]), L("hha", "ح", "حَاء", "ḥāʾ", "ħ", ["hams", "rakhawa", "istifal", "infitah", "ismat"])],
  },
  {
    id: "throat-near",
    short: "Uvula",
    ar: "أَدْنَى الحَلْق",
    en: "Upper throat",
    where: "the uvula: the back of the tongue raised to it",
    letters: [L("ghayn", "غ", "غَيْن", "ghayn", "ʁ", ["jahr", "rakhawa", "istila", "infitah", "ismat"]), L("kha", "خ", "خَاء", "khāʾ", "χ", ["hams", "rakhawa", "istila", "infitah", "ismat"])],
  },
  {
    id: "tongue-back",
    short: "Velum",
    ar: "أَقْصَى اللِّسَان",
    en: "Back of the tongue",
    where: "against the soft palate (qāf deeper, kāf a little forward)",
    letters: [L("qaf", "ق", "قَاف", "qāf", "q", ["jahr", "shidda", "istila", "infitah", "ismat", "qalqala"]), L("kaf", "ك", "كَاف", "kāf", "k", ["hams", "shidda", "istifal", "infitah", "ismat"])],
  },
  {
    id: "tongue-mid",
    short: "Palate",
    ar: "وَسَط اللِّسَان",
    en: "Middle of the tongue",
    where: "against the hard palate",
    letters: [L("jim", "ج", "جِيم", "jīm", "dʒ", ["jahr", "shidda", "istifal", "infitah", "ismat", "qalqala"]), L("shin", "ش", "شِين", "shīn", "ʃ", ["hams", "rakhawa", "istifal", "infitah", "ismat", "tafashshi"]), L("ya", "ي", "يَاء", "yāʾ", "j", ["jahr", "rakhawa", "istifal", "infitah", "ismat", "lin"])],
  },
  {
    id: "tongue-edge",
    short: "Molars",
    ar: "حَافَّة اللِّسَان",
    en: "Edge of the tongue",
    where: "its sides against the upper molars and gums",
    letters: [L("dad", "ض", "ضَاد", "ḍād", "dˤ", ["jahr", "rakhawa", "istila", "itbaq", "ismat", "istitala"]), L("lam", "ل", "لَام", "lām", "l", ["jahr", "tawassut", "istifal", "infitah", "idhlaq", "inhiraf"])],
  },
  {
    id: "tongue-tip",
    short: "Ridge",
    ar: "طَرَف اللِّسَان",
    en: "Tip of the tongue",
    where: "at the gums and the front teeth",
    letters: [
      L("nun", "ن", "نُون", "nūn", "n", ["jahr", "tawassut", "istifal", "infitah", "idhlaq", "ghunna"]),
      L("ra", "ر", "رَاء", "rāʾ", "r", ["jahr", "tawassut", "istifal", "infitah", "idhlaq", "inhiraf", "takrir"]),
      L("tta", "ط", "طَاء", "ṭāʾ", "tˤ", ["jahr", "shidda", "istila", "itbaq", "ismat", "qalqala"]),
      L("dal", "د", "دَال", "dāl", "d", ["jahr", "shidda", "istifal", "infitah", "ismat", "qalqala"]),
      L("ta", "ت", "تَاء", "tāʾ", "t", ["hams", "shidda", "istifal", "infitah", "ismat"]),
      L("sad", "ص", "صَاد", "ṣād", "sˤ", ["hams", "rakhawa", "istila", "itbaq", "ismat", "safir"]),
      L("sin", "س", "سِين", "sīn", "s", ["hams", "rakhawa", "istifal", "infitah", "ismat", "safir"]),
      L("zay", "ز", "زَاي", "zāy", "z", ["jahr", "rakhawa", "istifal", "infitah", "ismat", "safir"]),
      L("zza", "ظ", "ظَاء", "ẓāʾ", "ðˤ", ["jahr", "rakhawa", "istila", "itbaq", "ismat"]),
      L("dhal", "ذ", "ذَال", "dhāl", "ð", ["jahr", "rakhawa", "istifal", "infitah", "ismat"]),
      L("tha", "ث", "ثَاء", "thāʾ", "θ", ["hams", "rakhawa", "istifal", "infitah", "ismat"]),
    ],
  },
  {
    id: "lips",
    short: "Lips",
    ar: "الشَّفَتَان",
    en: "Lips",
    where: "the lower lip on the upper teeth (fāʾ), or both lips",
    letters: [L("fa", "ف", "فَاء", "fāʾ", "f", ["hams", "rakhawa", "istifal", "infitah", "idhlaq"]), L("ba", "ب", "بَاء", "bāʾ", "b", ["jahr", "shidda", "istifal", "infitah", "idhlaq", "qalqala"]), L("mim", "م", "مِيم", "mīm", "m", ["jahr", "tawassut", "istifal", "infitah", "idhlaq", "ghunna"]), L("waw", "و", "وَاو", "wāw", "w", ["jahr", "rakhawa", "istifal", "infitah", "ismat", "lin"])],
  },
];

export const LETTERS: Letter[] = MAKHARIJ.flatMap((m) => m.letters);

/** How a letter is said when its cell is pressed. */
export type LetterForm = "fatha" | "kasra" | "damma" | "sakin" | "name";

export const LETTER_FORMS: { id: LetterForm; label: string; title: string }[] = [
  { id: "fatha", label: "َـ a", title: "With fatḥa" },
  { id: "kasra", label: "ِـ i", title: "With kasra" },
  { id: "damma", label: "ُـ u", title: "With ḍamma" },
  { id: "sakin", label: "أَـْ", title: "Without a vowel, after a hamza: how tajweed finds a letter's point of articulation" },
  { id: "name", label: "Name", title: "Its name" },
];

/**
 * What to say for a letter in a form, and whether its last word is read in pause. The vowelled
 * forms are joined (a pause would drop the vowel).
 */
export function letterText(l: Letter, f: LetterForm): { text: string; waqf: boolean } {
  if (f === "name") return { text: l.name, waqf: true };
  if (l.madd) return { text: l.madd, waqf: true };
  // a hamza is written on its seat by its vowel
  const c = l.id === "hamza" ? { fatha: "أ", kasra: "إ", damma: "أ", sakin: "أ" }[f] : l.char;
  if (f === "sakin") return { text: `أَ${c}ْ`, waqf: true };
  return { text: c + { fatha: "َ", kasra: "ِ", damma: "ُ" }[f], waqf: false };
}

/** The chart's letter for a written character (the consonant, not the madd letter). */
export function letterOf(ch: string): Letter | undefined {
  const c = { أ: "ء", إ: "ء", آ: "ء", ؤ: "ء", ئ: "ء", ٱ: "ا", ى: "ا", ة: "ت" }[ch] ?? ch;
  return LETTERS.find((l) => l.char === c && !l.madd) ?? LETTERS.find((l) => l.char === c);
}

export interface Preset {
  text: string;
  /** what it means */
  en: string;
}

export interface PresetGroup {
  id: string;
  title: string;
  presets: Preset[];
}

export const PRESET_GROUPS: PresetGroup[] = [
  {
    id: "greetings",
    title: "Greetings",
    presets: [
      { text: "السَّلَامُ عَلَيْكُمْ", en: "Peace be upon you" },
      { text: "وَعَلَيْكُمُ السَّلَامُ", en: "And upon you, peace" },
      { text: "مَرْحَبًا", en: "Hello" },
      { text: "صَبَاحُ الْخَيْرِ", en: "Good morning" },
      { text: "مَسَاءُ الْخَيْرِ", en: "Good evening" },
      { text: "كَيْفَ حَالُكَ؟", en: "How are you?" },
      { text: "شُكْرًا جَزِيلًا", en: "Thank you very much" },
      { text: "مَعَ السَّلَامَةِ", en: "Goodbye" },
    ],
  },
  {
    id: "words",
    title: "Words",
    presets: [
      { text: "كِتَابٌ", en: "book" },
      { text: "قَلَمٌ", en: "pen" },
      { text: "بَيْتٌ", en: "house" },
      { text: "مَدْرَسَةٌ", en: "school" },
      { text: "الشَّمْسُ", en: "the sun (a sun letter)" },
      { text: "الْقَمَرُ", en: "the moon (a moon letter)" },
      { text: "مَاءٌ", en: "water" },
      { text: "عَرَبِيٌّ", en: "Arabic" },
    ],
  },
  {
    id: "sentences",
    title: "Sentences",
    presets: [
      { text: "أَنَا أُحِبُّ اللُّغَةَ الْعَرَبِيَّةَ", en: "I love the Arabic language" },
      { text: "ذَهَبَ الْوَلَدُ إِلَى الْمَدْرَسَةِ", en: "The boy went to school" },
      { text: "الْعِلْمُ نُورٌ", en: "Knowledge is light" },
      { text: "الصَّبْرُ مِفْتَاحُ الْفَرَجِ", en: "Patience is the key to relief" },
      { text: "هَذَا كِتَابٌ جَمِيلٌ", en: "This is a beautiful book" },
    ],
  },
  {
    id: "pairs",
    title: "Minimal pairs",
    presets: [
      { text: "سَيْفٌ، صَيْفٌ", en: "sword, summer: s and the emphatic ṣ" },
      { text: "تِينٌ، طِينٌ", en: "figs, clay: t and ṭ" },
      { text: "دَرْبٌ، ضَرْبٌ", en: "path, striking: d and ḍ" },
      { text: "قَلْبٌ، كَلْبٌ", en: "heart, dog: q and k" },
      { text: "حَالٌ، خَالٌ", en: "state, uncle: ḥ and kh" },
      { text: "عَمَلٌ، أَمَلٌ", en: "work, hope: ʿayn and hamza" },
      { text: "هَرَمٌ، حَرَمٌ", en: "pyramid, sanctuary: h and ḥ" },
      { text: "جَمَلٌ، جَمَالٌ", en: "camel, beauty: a short and a long vowel" },
      { text: "عَلِمَ، عَلَّمَ", en: "knew, taught: a single and a doubled consonant" },
    ],
  },
  {
    id: "quran",
    title: "Qurʾān",
    presets: [
      { text: "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ", en: "In the name of God, the Most Gracious, the Most Merciful (1:1)" },
      { text: "ٱلْحَمْدُ لِلَّهِ رَبِّ ٱلْعَٰلَمِينَ", en: "All praise is for God, Lord of the worlds (1:2)" },
      { text: "ٱلرَّحْمَٰنِ ٱلرَّحِيمِ ۝ مَٰلِكِ يَوْمِ ٱلدِّينِ", en: "The Most Gracious, the Most Merciful. Master of the Day of Judgement (1:3–4)" },
      { text: "إِيَّاكَ نَعْبُدُ وَإِيَّاكَ نَسْتَعِينُ", en: "You alone we worship, and You alone we ask for help (1:5)" },
      { text: "ٱهْدِنَا ٱلصِّرَٰطَ ٱلْمُسْتَقِيمَ", en: "Guide us along the straight path (1:6)" },
      { text: "صِرَٰطَ ٱلَّذِينَ أَنْعَمْتَ عَلَيْهِمْ غَيْرِ ٱلْمَغْضُوبِ عَلَيْهِمْ وَلَا ٱلضَّآلِّينَ", en: "The path of those You have blessed, not of those who earned anger, nor of those astray (1:7)" },
      { text: "ذَٰلِكَ ٱلْكِتَٰبُ لَا رَيْبَ ۛ فِيهِ ۛ هُدًى لِّلْمُتَّقِينَ ۝ ٱلَّذِينَ يُؤْمِنُونَ بِٱلْغَيْبِ وَيُقِيمُونَ ٱلصَّلَوٰةَ وَمِمَّا رَزَقْنَٰهُمْ يُنفِقُونَ", en: "This is the Book, no doubt in it, a guidance for the mindful; who believe in the unseen, keep up the prayer, and give of what We provided them (2:2–3)" },
      { text: "لَآ أُقْسِمُ بِيَوْمِ ٱلْقِيَٰمَةِ ۝ وَلَآ أُقْسِمُ بِٱلنَّفْسِ ٱللَّوَّامَةِ ۝ أَيَحْسَبُ ٱلْإِنسَٰنُ أَلَّن نَّجْمَعَ عِظَامَهُۥ", en: "Sūrat al-Qiyāma: I swear by the Day of Resurrection; and I swear by the self-reproaching soul; does man think We will not gather his bones? (75:1–3)" },
      { text: "قُلْ هُوَ ٱللَّهُ أَحَدٌ ۝ ٱللَّهُ ٱلصَّمَدُ ۝ لَمْ يَلِدْ وَلَمْ يُولَدْ ۝ وَلَمْ يَكُن لَّهُۥ كُفُوًا أَحَدٌۢ", en: "Sūrat al-Ikhlāṣ: Say, He is God, the One; God, the Eternal; He neither begets nor is born; and none is comparable to Him (112)" },
    ],
  },
];
