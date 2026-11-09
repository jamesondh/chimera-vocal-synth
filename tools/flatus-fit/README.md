# flatus-fit: the flatulence creature against real recordings

Measures recorded flatulence and the creature's renders with the same level-free features, and
fits the creature's physical controls to each recorded event. Results and method:
docs/flatulence.md ("Validation against recordings") and docs/flatulence-validation-log.md.

Needs Python 3 with numpy, scipy (matplotlib for `compare.py`), and ffmpeg. The recordings are
downloaded data: keep them outside the repository and run every script with `python3 -I`.

```sh
cargo build --release -p chimera --example flatus_render
D=/somewhere/outside/the/repo
python3 -I tools/flatus-fit/fetch.py tools/flatus-fit/manifest.json $D/raw $D/wav
python3 -I tools/flatus-fit/events.py tools/flatus-fit/manifest.json $D/wav $D/events.json $D/clips
python3 -I tools/flatus-fit/library.py $D/lib.json $D/tmp                       # ~6 min
python3 -I tools/flatus-fit/fit.py $D/events.json $D/lib.json $D/fit.json $D/tmp   # ~15 min
python3 -I tools/flatus-fit/report.py $D/fit.json [$D/fit-before.json …]
python3 -I tools/flatus-fit/compare.py $D/fit.json $D/clips $D/compare.png $D/tmp [event …]
```

- `manifest.json`: every recording considered, its URL, author, license and why it is in or
  out. Freesound files are its public HQ previews (128 kb/s MP3).
- `features.py`: event segmentation, the features, the distance and the feature scales.
- `sim.py`: drives `examples/flatus_render` (one process; `FLATUS_RENDER` picks another
  binary, e.g. a build of an earlier commit).
- `library.py`: renders a grid of the controls (push held until the gas is gone).
- `fit.py`: per event, the nearest library renders, then Nelder–Mead over the controls and the
  push's timing. Pass `scale=<file>` (a JSON list) to reuse the feature scales of an earlier run;
  they come from the real events only, so they are the same for every model version.
- `report.py`: the per-event table, the counts of close events and reached recordings, and the
  features that miss most.
