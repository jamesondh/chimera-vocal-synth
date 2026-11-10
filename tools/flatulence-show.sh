#!/bin/sh
# Flatulence: a physical model. The Bestiary's flatulence creature plays each of its calls, then
# three held notes with physical courses (tools/flatulence-show.json), as a creature show:
# public/shows/flatulence/, opened at ?show=flatulence. Usage: tools/flatulence-show.sh
set -eu
cd "$(dirname "$0")/.."
cargo build --release -q -p chimera-cli
out=public/shows/flatulence
mkdir -p "$out"
target/release/chimera render creature-show tools/flatulence-show.json "$out/flatulence" --tele "$out/flatulence.ctele"
cat > "$out/show.json" <<J
{ "title": "Flatulence: a physical model", "audio": "flatulence.wav",
  "creature": "flatulence", "tele": "flatulence.ctele" }
J
echo "installed $out (?show=flatulence)"
