#!/usr/bin/env bash
# Build each book's in-page kernel and bind it into the built book, beside its pages.
#
#   ./scripts/build-kernels.sh                  # every book that has one, into books/<name>/book/wasm
#   ./scripts/build-kernels.sh <book> <dir>     # one book's kernel, into <dir> (serve-with-drafts)
#
# A book's cells are made runnable by `js/run.js`, ONE script both books load (the gonk book's
# is a symlink to the ikigai book's). It imports `wasm/book_wasm.js` beside the book's root, so
# every kernel is bound under that name (`--out-name book_wasm`) whatever its crate is called.
# For The ikigai Book that is the name wasm-bindgen would choose anyway, so its output is the
# same bytes it always was.
#
# The one table of which crate is which book's kernel is `kernel_of` below. Each crate is built
# in a cargo invocation of its own, so one kernel's features can never unify into another's.
#
# Needs the `wasm32-unknown-unknown` target and `wasm-bindgen-cli` at the version the kernels
# pin (`=0.2.108`; pages.yml installs it, `serve-with-drafts.sh` checks it).
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

kernel_of() {
    case "$1" in
        ikigai) echo book-wasm ;;
        gonk) echo gonk-book-wasm ;;
        *) echo "" ;;
    esac
}

build() {
    local book="$1" out="$2" crate lib
    crate="$(kernel_of "$book")"
    if [ -z "$crate" ]; then
        echo "books/$book has no in-page kernel (add it to kernel_of in $0)" >&2
        return 1
    fi
    lib="${crate//-/_}"
    cargo build --release -p "$crate" --lib --target wasm32-unknown-unknown
    wasm-bindgen --target web --out-name book_wasm --out-dir "$out" \
        "target/wasm32-unknown-unknown/release/$lib.wasm"
    test -s "$out/book_wasm_bg.wasm"
    echo "  $crate → $out"
}

if [ "$#" -eq 2 ]; then
    build "$1" "$2"
    exit 0
fi
if [ "$#" -ne 0 ]; then
    sed -n '4,5p' "$0" | sed 's/^# \{0,1\}//' >&2
    exit 2
fi
for dir in books/*/; do
    name="$(basename "$dir")"
    [ -n "$(kernel_of "$name")" ] || continue
    if [ ! -d "$dir/book" ]; then
        echo "$dir/book is not built — run ./scripts/test-books.sh first" >&2
        exit 1
    fi
    build "$name" "$dir/book/wasm"
done
