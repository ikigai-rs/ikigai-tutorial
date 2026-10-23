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

# Each book's "What's new" page, from its changes.toml, BEFORE any `mdbook build`, so the page
# every build (and the deploy) publishes is the one the entries say. The page is committed too:
# `crates/book-whats-new/tests/pages.rs` fails when the committed page and the entries disagree,
# so a page written here and not committed turns `cargo test` red rather than going stale.
"$TARGET/debug/book-whats-new" generate

# The library's page, from books/library/library.toml and each book's book.toml, for the same
# reason and with the same check (`crates/book-library/tests/page.rs`), which also fails when a
# book under books/ has no entry in the library.
"$TARGET/debug/book-library" generate

status=0
for book in books/*/; do
    name="$(basename "$book")"
    echo "── $name ─────────────────────────────────"
    mdbook build "$book"
    if ! mdbook test "$book" -L "$TARGET/debug/deps"; then
        echo "FAILED: $name"
        status=1
    fi
    # The contrast gate reads the palette mdbook JUST generated, plus the book's own
    # additional-css, in cascade order — so it has to run here, after the build, rather
    # than as a `cargo test`. A table of hex values in a test file would go stale the
    # first time mdbook changed a theme, and stay green while doing it.
    #
    # The binary the workspace build above made, run directly — NOT `cargo run -p book-a11y`.
    # That re-resolves features for book-a11y ALONE, rebuilds shared dependencies with that
    # narrower set into the same `deps/`, and leaves a second `ikigai_core` rlib there: the
    # next book's `mdbook test` then fails with E0464. With one book it never showed, because
    # nothing ran after the last contrast check; the second book is what exposed it.
    if ! "$TARGET/debug/book-a11y" "$book"; then
        echo "FAILED: $name (contrast)"
        status=1
    fi
done

exit "$status"
