#!/usr/bin/env bash
# One-time setup for tools/asr/arabic_judge.py (Apple silicon): a Python env with mlx-whisper in
# ~/.cache/chimera-asr-ar. The model (whisper-large-v3-turbo, ~1.6 GB) downloads into the Hugging
# Face cache on first use.
set -euo pipefail
dir="$HOME/.cache/chimera-asr-ar"
mkdir -p "$dir"
[ -x "$dir/.venv/bin/python" ] || uv venv -q --python 3.12 "$dir/.venv"
uv pip install -q --python "$dir/.venv/bin/python" mlx-whisper==0.4.2
echo "run: cargo build --release -p chimera-cli && $dir/.venv/bin/python tools/asr/arabic_judge.py <outdir> [--against earlier/arabic.json] [--control]"
