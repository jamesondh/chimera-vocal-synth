# CHIMERA

A physical-modeling voice synthesizer in the browser. Every sound comes from a simulated
airway (lungs, vocal folds or syrinx, a branched vocal tract, lips, nose), not from samples.
The engine is Rust, compiled to WebAssembly for an AudioWorklet; the same code runs natively
for tests and the `chimera` CLI.

## What it does

- **Play**: drag the tongue, lips and jaw in a live cutaway of the head; vowel pad, encoders,
  computer keyboard and Web MIDI.
- **Speak**: English text (CMUdict with a letter-to-sound fallback) or IPA, turned into
  gestures and prosody, with the gestural score and a consonant chart on screen.
- **Sing**: piano roll with lyrics per note; consonants land before the beat.
- **Beatbox**: pads and a step sequencer, every drum an articulatory gesture.
- **Arabic** (`/arabic`): vowelled Arabic read by the rules of correct reading (waṣl and waqf,
  sun letters, shadda, madd, tafkhīm), the emphatics as a pulled-back tongue, the letters by
  where tajweed places them, presets from greetings to al-Fātiḥa. Groundwork for tajweed.
- **Bestiary**: songbird (two-voice syrinx), seal, alligator and elephant.
- **Duet** (`?show=<id>`): two 3D heads replay a pre-rendered song.

Male, female and child bodies blend continuously. Everything on screen animates from live
engine state: pressure in the airway, airflow, the vocal folds, spectra and formants.

## Arabic and Qurʾān recitation

An articulatory synthesizer is only as good as the airway it simulates, and Arabic asks more of
that airway than almost any language. Its 28 letters are made at every point from the glottis to
the lips: hamza and hāʾ at the glottis, ʿayn and ḥāʾ in the pharynx (the tongue root pulled
against its back wall), ghayn, khāʾ and qāf at the uvula, kāf at the soft palate, jīm, shīn and
yāʾ at the hard palate, ḍād along the edge of the tongue, the tip letters at the gums and teeth,
and fāʾ, bāʾ, mīm and wāw at the lips. English uses the front half of that space; it has no
pharyngeals and no uvulars. And four Arabic letters (ص ض ط ظ, the emphatics) are made twice at
once: the tip of the tongue at the teeth while its root and back pull toward the throat, which
darkens every vowel beside them. A sampled voice can paste that in; a simulated tongue has to
actually do it.

The Qurʾān is where that precision matters most. Tajweed, the discipline of reciting it, has
specified for over a thousand years exactly where each letter is made (its makhraj, from the
deepest throat to the lips) and how (its ṣifāt: voiced or breathy, stopped or running, heavy or
light), how long each long vowel is held in counted beats, when a nūn hides in the nose, merges
into the next letter or turns into a mīm. Mastering those articulations is widely held to be the
hardest part of learning to recite: reciters train for years under a teacher, letter by letter.
For a voice made of a simulated mouth, that tradition is an exact specification to meet, and a
demanding test of the anatomy.

What exists (details, measurements and the plan in [docs/arabic.md](docs/arabic.md)):

- **The sounds**: the emphatics as a pulled-back tongue (F2 of /a/ down ~280 Hz, as measured in
  Arabic), pharyngeal and uvular fricatives, unaspirated qāf, trills that trill.
- **The reader** (`phonetics/arabic.rs`): vowelled or Uthmani text, read by the rules of correct
  reading (waṣl and waqf, sun letters, shadda, madd, tafkhīm, the rāʾ and the name Allāh).
- **Tajweed**: annotated as a tajweed muṣḥaf colours it, scored over the whole Qurʾān against an
  expert annotation (cpfair/quran-tajweed): **F1 99.8 %**; and recited (Ḥafṣ by way of
  al-Shāṭibiyya): madd of 2, 4 and 6 counts, ghunna, idghām, iqlāb, ikhfāʾ.
- **The page** (`/arabic`): read or recite, the text coloured by its tajweed with the letter being
  said lit, presets from greetings to al-Fātiḥa, al-Ikhlāṣ and al-Qiyāma, and the letters laid
  out by makhraj from throat to lips with their ṣifāt.
- **Judged from outside**: Whisper (large-v3-turbo) on minimal pairs and sentences, the Qurʾān
  lines transcribed without an error, spoken or recited.
- **Video**: `bun tools/clip.ts out.mp4 "<vowelled Arabic>"` renders a square clip of the 3D head
  reciting, with its sound.

## Run it

Needs Rust with the `wasm32-unknown-unknown` target, and Bun.

```sh
bun install
bun run dev        # builds the wasm, then Vite on 127.0.0.1:5317
bun run test       # wasm build, cargo test --workspace, bun test tests
```

The AudioWorklet needs a secure context, so open it on localhost or over HTTPS (for example
`tailscale serve`). Vercel builds it with `vercel.json`.

Offline renders and measurement:

```sh
cargo run --release -p chimera-cli -- render speech|text|arabic|song|notes|beatbox|creature|vowels …
cargo run --release -p chimera-cli -- diagnose       # nasal leakage, formants, bandwidths, pitch
cargo run --release -p chimera-cli -- compare csd    # against a real singer (docs/csd-comparison.md)
cargo run --release -p chimera-cli -- tajweed score  # tajweed against the whole Qurʾān (docs/arabic.md)
bun tools/spectrogram.ts in.wav out.png
```

## Docs

- [docs/architecture.md](docs/architecture.md): the acoustic model (FDTD airway, sources,
  anatomy, gestures).
- [docs/rust-engine.md](docs/rust-engine.md): the engine's real-time and determinism contract,
  and the browser boundary.
- [docs/ui-redesign.md](docs/ui-redesign.md) and [docs/mockups/](docs/mockups/): the vector
  display look.
- [docs/csd-comparison.md](docs/csd-comparison.md): scoring the voice against the Children's
  Song Dataset.
- [docs/hill-climb/](docs/hill-climb/): the rules and log for tuning the voice one measured
  step at a time.
- [docs/arabic.md](docs/arabic.md): Arabic, its Whisper judge and results, and the plan for
  tajweed and Qurʾān recitation.

Agents: see [AGENTS.md](AGENTS.md).

## License

[MIT](LICENSE). Bundled third-party files keep their own licenses: the CMU Pronouncing
Dictionary ([public/data/CMUDICT-LICENSE.txt](public/data/CMUDICT-LICENSE.txt)) and the fonts
([public/fonts/](public/fonts/), Amiri Quran in [AMIRI-OFL.txt](public/fonts/AMIRI-OFL.txt)).
