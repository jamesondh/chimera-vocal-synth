#!/usr/bin/env bash
# Fetch the tajweed reference for `chimera tajweed score`: cpfair/quran-tajweed's annotations
# (riwāyat Ḥafṣ, CC BY 4.0) and the copy of the Tanzil.net Uthmani text they index into (Tanzil's
# terms of use: verbatim copies only). Neither goes in git.
#   tools/fetch-tajweed.sh [dir]     (default ~/datasets/quran-tajweed)
set -euo pipefail
dir="${1:-$HOME/datasets/quran-tajweed}"
mkdir -p "$dir"
curl -fsSL -o "$dir/tajweed.json" https://raw.githubusercontent.com/cpfair/quran-tajweed/master/output/tajweed.hafs.uthmani-pause-sajdah.json
curl -fsSL -o "$dir/quran-uthmani.txt" https://github.com/cpfair/quran-tajweed/files/7281388/quran-uthmani.txt
echo "fetched to $dir: $(wc -l < "$dir/quran-uthmani.txt") lines of text, $(wc -c < "$dir/tajweed.json") bytes of annotations"
