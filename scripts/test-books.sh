#!/usr/bin/env bash
# Build every book and run its examples.
#
# `mdbook test` needs `-L <deps>` to find the crates the examples use, and rustc refuses
# with E0464 when that directory holds more than one candidate for a crate. A shared
# `target/` accumulates them: `cargo clippy` leaves `.rmeta` beside `cargo build`'s
# `.rlib`, and a restored CI cache can carry stale ones from an older dependency set.
#
# So the books build into a target directory of their own, wiped first. It costs one
# compile of the workspace and removes an entire class of confusing failure — the error
# rustc gives says nothing about books, and the first person to hit it will not guess.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

TARGET="${BOOK_TARGET_DIR:-target/books}"
rm -rf "$TARGET"
CARGO_TARGET_DIR="$TARGET" cargo build

status=0
for book in books/*/; do
    name="$(basename "$book")"
    echo "── $name ─────────────────────────────────"
    mdbook build "$book"
    if ! mdbook test "$book" -L "$TARGET/debug/deps"; then
        echo "FAILED: $name"
        status=1
    fi
done

exit "$status"
