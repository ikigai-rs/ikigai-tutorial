#!/usr/bin/env bash
# Regenerate the images of the gonk Book's chapter "The Dashboard" (ledger #2973). The one command:
#
#   ./scripts/capture-dashboard.sh [--out <dir>] [--chrome <path>]
#
# It starts a scratch gonk in a temporary directory, on a port of its own, and never reads or writes
# a real one. The gonk and the cli are the book's pins, as `scripts/test-transcripts.sh` installs
# them; run that once first. The browser is the machine's Google Chrome, or the one `--chrome` names.
# Commit what it writes: the images are a record of one run, and only their dates differ between runs.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

rev="$(tr -d '[:space:]' < books/gonk/gonk.rev)"
cli="$(tr -d '[:space:]' < books/gonk/ikigai-cli.version)"
gonk="$PWD/target/gonk/$rev/bin/ikigai-gonk"
ikigai="$PWD/target/ikigai-cli/$cli+quic/bin/ikigai"
for binary in "$gonk" "$ikigai"; do
    if [ ! -x "$binary" ]; then
        echo "no $binary: ./scripts/test-transcripts.sh installs the book's pins" >&2
        exit 2
    fi
done

out="$PWD/books/gonk/src/ledger/dashboard/img"
chrome=()
while [ $# -gt 0 ]; do
    case "$1" in
        --out) out="$(mkdir -p "$2" && cd "$2" && pwd)"; shift 2 ;;
        --chrome) chrome=(--chrome "$2"); shift 2 ;;
        *) echo "usage: $0 [--out <dir>] [--chrome <path>]" >&2; exit 2 ;;
    esac
done

seed="$PWD/books/gonk/src/ledger/dashboard/seed.py"
cd a11y
[ -d node_modules ] || npm ci
node capture-dashboard.mjs --gonk "$gonk" --ikigai "$ikigai" --seed "$seed" --out "$out" "${chrome[@]}"
