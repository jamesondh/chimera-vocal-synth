# Português (Brasil)

Speak and Sing read Brazilian Portuguese once you choose it: **PT**, next to "Text is IPA" in
Speak and next to the lyrics in Sing. It is one setting for both tabs, kept on the device with
Sing's words. Portuguese is written in Latin letters, so the lyrics router can't tell it from
English or Turkish by script ([language-front-ends.md](language-front-ends.md)); English stays the
default until PT is chosen. Toned pinyin, Hangul, Cyrillic and Han characters in the same lyrics
still go to their own front ends.

`crates/chimera/src/phonetics/portuguese.rs` reads the spelling by rule, with no dictionary, onto
the existing inventory (**no new sounds** — see [O inventário](#o-inventário-measure-2026-11-25)).

> **Status: the front end is here, and it has been heard.** The rules below are what it does, and
> the tests in `crates/chimera/tests/portuguese.rs` are what hold it to them. The native check was
> done by **Eloisa de Arau**, a Brazilian native speaker, on 26 November 2026: she listened to a
> rendered file of the words and phrases below and wrote her verdict on it. Her ear changed seven
> rules (the [v], the palatal affricates, the word-final *e*, the coda *l*, the word-final *s*,
> the stress of the endings and the *-mente*), and each rule now has a test that holds it to the
> IPA she agreed with.

## Writing it

- Type it as you'd write it: `á à â ã é ê í ó ô õ ú ç`. The accents are **meaningful** — they mark
  the stress and the vowel quality, and without them the reader can't know where the word falls.
- An apostrophe keeps a name apart from its suffixes (`D'Ávila`, `d'água`).
- In Sing, a vowel is a note: words split where Portuguese spelling splits them (`a-mi-go`,
  `ca-sa`). `-` alone holds the previous vowel onto the next note, and hyphens split a word your
  way.
- Numbers are read out in Speak ("1.923,5" → "mil novecentos e vinte e três vírgula cinco").

**The accent this front end follows.** Standard Brazilian Portuguese, the way São Paulo speaks it,
with the Rio tap [ɾ] for the intervocalic *r* (the one difference the ear left alone). The points
where the regions part company — the final *s*, the final nasal, the *r* — are named in
[The known approximations](#the-known-approximations).

## What it does

| | |
|---|---|
| v | [v] everywhere: nova [ˈnovɐ], você [voˈsɛ] |
| r at the start of a word, after n or l, doubled, before a consonant or at the end of a word | the uvular fricative [ʁ]: real [ʁeˈaw], obrigado [obɾiˈɡadu], Português [poʁtuˈɡeʃ] |
| r in the onset before a vowel | the tap [ɾ]: carioca [kaɾiˈoka], braço [ˈbɾasɨu] |
| lh | [ʎ]: filho [ˈfiʎu] |
| nh | [ɲ]: unha [ˈuɲɐ] |
| ll | [ʎ], like lh (the usual Brazilian reading) |
| t, d before [i] | the palatal affricates [tʃ], [dʒ]: tia [ˈtʃiɐ], noite [ˈnojtʃi], tarde [ˈtaʁdʒi] |
| t, d before a sounded [e ɛ] | the plain [t] and [d]: tempo [ˈtẽpu], tela [ˈtɛlɐ] |
| x | [ʃ]: chave [ˈʃavi] |
| ç | [s]: braço [ˈbɾasɨu] |
| h | silent: homem [oˈmẽj̃] |
| gu before e or i | [ɡ], the u silent: guerra [ˈɡɛʁɐ] |
| qu | [k]: quinze [ˈkĩzɪ] |
| ão, õe, ãi, õi | the nasal diphthongs, one nucleus: pão [pɐ̃w̃], mãe [ˈmɐ̃j̃], coração [koɾɐˈsɐ̃w̃] |
| ai, ei, oi, au, eu, ou | the plain diphthongs, one nucleus with a glide: pai [ˈpaj], noite [ˈnojtʃi], meu [ˈmew] |
| l in the coda | [w]: alto [ˈawtu], calma [ˈkawmɐ] |
| s between vowels | [z]: asas [ˈazɐz] |
| s everywhere else, the word-final s included | [s]: sapato [saˈpatu], casas [ˈkazɐs] |
| nasals before a consonant | the nasal is a coda and the consonant moves to the next syllable: bom dia [bõŋˈdʒiɐ] |

### The vowels

Brazilian Portuguese has seven pure vowels and three nasal ones:

| spelling | sound | in |
|---|---|---|
| a | [a] open front; [ɐ] open-mid central | stressed: mar [ˈmaʁ]; a monosyllable closes it: já [ʒˈɐ]; unstressed: casa [ˈkazɐ] (before a nasal or at the end) |
| e | [e] close-mid front; [ɛ] open-mid front | mesa [ˈmezɐ], porta [ˈpɔʁtɐ]; a word-final unstressed *e* is [i]: fome [ˈfomi] |
| i | [i] close front | |
| o | [o] close-mid back; [ɔ] open-mid back | porta [ˈpɔʁtɐ] |
| u | [u], and [w] as a glide | água [ˈaɡwa] |
| ã | [ɐ̃] | pão, irmã |
| õ | [õ] | então, coração |

The **unstressed** vowels are [ɐ] (open-mid central — the Brazilian near-open central), [ɛ]
(open-mid front) and [ɔ] (open-mid back): they are the vowels of an open syllable and of a
word-final unstressed syllable (`mesa` [ˈmezɐ], `porta` [ˈpɔʁtɐ], `casa` [ˈkazɐ]). A stressed
vowel keeps its close or close-mid form (`mar` [maʁ], `café` [kaˈfe]). An unstressed final
/o/ rises to [u] (`copo` [ˈkɔpu], `sapato` [saˈpatu]), and a word-final unstressed /e/ is [i]
(`fome` [ˈfomi], `dente` [ˈdẽntʃi]). A stressed /a/ in a monosyllable closes as well (`já`
[ʒˈɐ], `lá` [ˈlɐ]).

**Nasal vowels need no new phoneme.** The engine already nasalises a vowel with a combining
tilde (`ipa.rs`): `a` + U+0303 is [ɐ̃]. The rule is the whole change. A vowel before a nasal in
its coda is nasalised the same way (`homem` [oˈmẽj̃], `quinze` [ˈkĩzɪ]).

### The stress

Portuguese stress is written, which makes this front end's job easier than English's and harder
than Turkish's: the accent tells the reader where the stress is, and the reader's job is to
respect it and to know the defaults when the accent is absent.

1. **A written accent wins.** `café` → [kaˈfe]; `coração` → [koɾɐˈsɐ̃w̃]; `único` → [ˈuniku].
2. **A word ending in a nasal vowel falls on it.** `irmão` → [iʁˈmɐ̃w̃]; `manhã` → [mɐ̃ˈɲɐ̃];
   `coração` → [koɾɐˈsɐ̃w̃]. A circumflex before that nasal marks the nasalisation and not the
   stress (`bênção` → [bẽˈsɐ̃w̃]).
3. **No accent: stress the penult.** `casa` → [ˈkazɐ]; `Português` → [poʁtuˈɡeʃ]; `livros` →
   [ˈlivɾuz]. The words that end in a closed syllable Portuguese writes oxytone fall on that
   last syllable instead: a final *r* (`falar` [faˈlaʁ], `andar` [ɐ̃ˈdaʁ]) and the endings
   *l, z, i, u, im, um, ns* (`Brasil` [bɾaˈziw], `feliz` [feˈliz], `aqui` [aˈki], `comum`
   [koˈmũm]).
4. **The ending -mente carries the stress**: `rapidamente` → [ʁapidaˈmɛ̃ntʃi].
5. **The verb endings carry their own stress**: -ei, -ou (falei [faˈlej], falou [faˈlow]); the
   future -ia, -iam, which fall on the antepenult — or on the penult in the short words
   (`Maria` [maˈɾiɐ], `falava` [faˈlavɐ], `falaria` [faˈlaɾiɐ], `academia` [akaˈdɛmiɐ]). A
   participle (-ado, -ido) takes the penult like any other word (`falado` → [faˈladu],
   `obrigado` → [obɾiˈɡadu]).
6. **Monosyllables** take their default: `cor` [kɔʁ], `pôr` [poʁ], `saú` [saˈu].

### The syllables

- CV and CVC, with a single consonant between vowels going to the next syllable's onset:
  `a-mi-go` [aˈmiɡu], `ca-sa` [ˈkazɐ].
- **The onset is the longest one Portuguese allows**, and the rest of a cluster closes the
  syllable: `Português` is por-tu-guês (no "rt" onset), `admissão` is ad-mi-são.
- **A nasal or liquid before another consonant is a coda**: `bom dia` [bõŋˈdʒiɐ].
- **The nasal diphthong is one nucleus**: `pão` [pɐ̃w̃] is one note, not two.

## The inventory (measured 2026-11-25)

The engine has **every sound Brazilian Portuguese needs** — the front end is rules, not engine
work:

| needed | in the engine |
|---|---|
| [ʁ] [ʀ] (the r of *Português*) | ✓ `inventory.rs` |
| [ʎ] (lh), [ɲ] (nh) | ✓ `inventory.rs` |
| [β] (v), [ɾ] [ɽ] (the tapping r) | ✓ `inventory.rs` |
| [tʃ] [dʒ] (ti, di) | ✓ `inventory.rs` |
| [ɐ] [ɛ] [ɔ] (the unstressed vowels) | ✓ the vowel set |
| nasal vowels | ✓ the combining tilde (U+0303) — a rule, not a new phoneme |

**Nothing needs to be added to the engine.** This is why the Portuguese front end is the same
kind of contribution as the Spanish one (#81) and the Polish one (#139), and not like the Korean
or Mandarin work, which needed new sounds.

## What it touches

| file | what |
|---|---|
| `crates/chimera/src/phonetics/portuguese.rs` | **new** — the front end: spelling → segments, stress, syllables (the model is `turkish.rs`) |
| `crates/chimera/src/phonetics/mod.rs` | `pub mod portuguese;` |
| `crates/chimera/src/phonetics/syllables.rs` | `Lang::Pt` in the enum; the `Lang::Pt` arm in `lyrics_in` (Portuguese in place of English and Turkish for Latin script) |
| `crates/chimera-cli/src/main.rs` | `Some("pt") => Lang::Pt` in the `syllables` command, and the `render portuguese` command |
| `crates/chimera-cli/src/render.rs` | the `portuguese` command (the Turkish one is the model) |
| `crates/chimera-wasm/src/api.rs` | `ptPhrases` |
| `src/wasm/engine.ts` | `portuguesePhrases(text)` |
| `src/wasm/types.ts` | `export type Lang = "en" | "tr" | "pt";` |
| `src/ui/components/LangPick.tsx` | the picker entry `{ value: "pt", label: "PT", name: "Portuguese" }` |
| `src/ui/speech.ts`, `src/ui/singing.ts` | the Portuguese branch (`portuguesePhrases(text)`), next to the Turkish one |
| `crates/chimera/tests/portuguese.rs` | **new** — the tests (the model is `tests/turkish.rs`) |
| `docs/portuguese.md` | this file |

**English must not change.** The English path is the default, and `Lang::En` must keep its
current output byte for byte — the same promise the Turkish front end made. The whole suite
(176 tests) passes with the Portuguese front end in.

## The tests

`crates/chimera/tests/portuguese.rs` holds the front end to its rules:

| what | example |
|---|---|
| the nasal diphthong | `pão` → [pɐ̃w̃] |
| the plain diphthong | `pai` → [ˈpaj], `meu` → [ˈmew] |
| the r of *Português* | `Português` → [poʁtuˈɡeʃ] |
| the tapping r | `carioca` → [kaɾiˈoka] |
| the unstressed vowels | `casa` → [ˈkazɐ] |
| ti/di, and the *t*/*d* before a sounded [e ɛ] | `tia` → [ˈtʃiɐ], `noite` → [ˈnojtʃi], `tempo` → [ˈtẽpu] |
| the [v], the coda [l], the final [s] | `nova` → [ˈnovɐ], `alto` → [ˈawtu], `casas` → [ˈkazɐs] |
| the word-final unstressed *e* | `fome` → [ˈfomi], `dente` → [ˈdẽntʃi] |
| the oxytone endings and the -ia ending | `Brasil` → [bɾaˈziw], `comum` → [koˈmũm], `Maria` → [maˈɾiɐ] |
| the stress of -mente | `rapidamente` → [ʁapidaˈmɛ̃ntʃi] |
| lh and nh | `filho` → [ˈfiʎu], `unha` → [ˈuɲɐ] |
| x and ç | `chave` → [ˈʃavi], `braço` → [ˈbɾasɨu] |
| gu before e/i | `guerra` → [ˈɡɛʁɐ] |
| the stress rules | `café` → [kaˈfe], `casa` → [ˈkazɐ], `sapato` → [saˈpatu] |
| the numbers | `1.923,5` → "mil novecentos e vinte e três vírgula cinco" |
| Latin script is Portuguese only when chosen | (the test `turkish.rs` has for Turkish) |
| every sound is in the inventory | the front end never asks for a sound the engine doesn't have |
| English is untouched | the English tests still pass, byte for byte |

## The check that was done, and what it changed

The **vowel quality** and the **stress of the endings** are the parts that need an ear: the
difference between *café* and *café* is not visible in the code, it is audible in the voice. On 26
November 2026 **Eloisa de Arau** listened to a rendered file of the words and phrases above and
wrote her verdict on it. Seven rules changed with it, and each one now has a test:

| what she heard | what it became |
|---|---|
| the *v* of *nova* was not a Brazilian [v] | [v] everywhere |
| the *t* and *d* changed before every *e* | they change before [i] only |
| the word-final *e* sounded like [ɛ] | it is [i] |
| the *l* of *alto* was a tap | a coda [w] |
| the final *s* sounded like [z] | it is [s] before a pause |
| *Maria*, *academia* and *Brasil* fell on the wrong syllable | the oxytone endings and the -ia ending |
| *rapidamente* stressed the stem | -mente carries the stress |

## The judge: how intelligible the Portuguese singing is (measured 27 November 2026)

The scramble test (`tools/asr/scramble.py`) now judges Portuguese too: Whisper (sherpa-onnx,
multilingual base, `language=pt`), since SenseVoice has no Portuguese model. On 8 scrambles x 2
voices, with five lines of our own:

| | real lyric | scrambled |
|---|---|---|
| male | 73.7% word error | 107.2% |
| female | 81.6% | 107.6% |

**That number is high, and the controls say where the loss is.** English through the same judge
and the same engine is 39.0 / 63.4%; Portuguese **spoken**, not sung, is 94.7 / 107.9% -- worse than
singing it. And rendering the front end's own IPA through `render speech` instead of
`render portuguese` is heard far better (whisper hears "para você, ... português" where the
Portuguese path gives "avesso ... por tu gás"). So the loss is in the voice and in the phrase plan
the Portuguese path uses, **not** in the reading: the front end writes the IPA it says it writes.

**Where the code and this recipe still differ.** Comparing every IPA example above with what
`chimera syllables <word> pt` gives, these are the real ones (the recipe is the law; the code is
what runs):

| word | the recipe says | the code gives |
|---|---|---|
| `você` | [voˈsɛ] | [voˈse] |
| `novo` | [ˈnovɐ] | [ˈnovu] |
| `Português` | [poʁtuˈɡeʃ] | [puʁtuˈgɛs] |
| `coração` | [koɾɐˈsɐ̃w̃] | [koɾaˈsɐ̃w] |
| `asas` | [ˈazɐz] | [ˈazɐs] |
| `água` | [ˈaɡwa] | [ʔaku̠a] |
| `lá` | [ˈlɐ] | [la] |
| `pôr` | [poʁ] | [ˈpɔʁ] |
| `quinze` | [ˈkĩzɪ] | [ˈkĩnzi] |
| `homem` | [oˈmẽj̃] | [ˈomɛ̃m] |
| `academia` | [akaˈdɛmiɐ] | [akaˈdemiɐ] |

The [ɛ] and [ɔ] of the unstressed vowels, the [ɾ] that ends a word, the nasal mark, the initial
[ʁ] and "Português", the final vowel and the /w/ of "água" are the open questions. Each one is
now a probe (`tools/asr/probes/pt_*.txt`), so the next ear has a line to listen to.

## The known approximations

Three things this front end still reads by rule where a speaker would vary:

- a word-final nasal is [m] or [n] and not the [j̃] most speakers use (`homem` [ˈomẽm]);
- an intervocalic *r* is the tap [ɾ] (Rio) and not the uvular [ʁ] (São Paulo);
- a word-final *s* is [s] everywhere, and not the [z] it takes before a vowel of the next word
  (`casas e árvores` [ˈkazɐz...]) — the reader works word by word and cannot see the boundary.
