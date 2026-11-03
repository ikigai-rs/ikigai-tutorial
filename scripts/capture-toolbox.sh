#!/usr/bin/env bash
# Regenerate the images of The ikigai Book's chapter "Checking a resource" (ledger #2980). The one
# command:
#
#   ./scripts/capture-toolbox.sh [--out <dir>] [--chrome <path>]
#
# It starts two scratch hosts on loopback ports of their own, each the chapter's crate over a toolbox
# build (`crates/check-resource/examples/toolbox_host.rs`), one with a caller who holds the scope and
# one with a caller who does not, and never reads or writes a gonk. The build is the book's pin,
# built by `scripts/build-toolbox.sh` the first time. The browser is the machine's Google Chrome, or
# the one `--chrome` names. Commit what it writes: the images are a record of one run.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

out="$PWD/books/ikigai/src/building/check/img"
chrome=()
while [ $# -gt 0 ]; do
    case "$1" in
        --out) out="$(mkdir -p "$2" && cd "$2" && pwd)"; shift 2 ;;
        --chrome) chrome=(--chrome "$2"); shift 2 ;;
        *) echo "usage: $0 [--out <dir>] [--chrome <path>]" >&2; exit 2 ;;
    esac
done

rev="$(tr -d '[:space:]' < books/ikigai/toolbox.rev)"
dist="$PWD/target/toolbox-dist/$rev"
[ -s "$dist/engine.wasm" ] || ./scripts/build-toolbox.sh "$dist"

cargo build --locked --quiet -p check-resource --example toolbox_host
target_dir="$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])')"

cd a11y
[ -d node_modules ] || npm ci
node capture-toolbox.mjs --host "$target_dir/debug/examples/toolbox_host" --dist "$dist" --out "$out" "${chrome[@]}"
