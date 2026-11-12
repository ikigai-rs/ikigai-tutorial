#!/usr/bin/env bash
# Build every book and run its examples.
#
# `mdbook test` needs `-L <dirs>` to find the crates the examples use (the build below reports
# which directories), and rustc refuses with E0464 when they hold more than one candidate for
# a crate. A shared
# `target/` accumulates them: `cargo clippy` leaves `.rmeta` beside `cargo build`'s
# `.rlib`, and a restored CI cache can carry stale ones from an older dependency set.
#
# So the books build into a target directory of their own, wiped first. It costs one
# compile of the workspace and removes an entire class of confusing failure — the error
# rustc gives says nothing about books, and the first person to hit it will not guess.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

# The browser and native matchers read the same cases. Also exercise the routing
# chapter's config edits with the platform's sed (GNU on Linux, BSD on macOS).
# And the vendored DOMPurify is held to its digest (a11y/vendor.test.mjs).
node --test a11y/transcript-matcher.test.mjs a11y/routing-commands.test.mjs a11y/vendor.test.mjs

TARGET="${BOOK_TARGET_DIR:-target/books}"
rm -rf "$TARGET"
mkdir -p "$TARGET"

# Where the examples' crates are is asked of cargo, never assumed. Cargo 1.100 moved its
# intermediate artifacts out of `debug/deps/` into one directory per unit (rust-lang/cargo#17354),
# and says plainly that the build directory's layout "is not a stable interface"; tools are to
# read its JSON messages instead. `-L "$TARGET/debug/deps"` then named a directory that no longer
# existed, and every example with an `extern crate` failed with E0463 (can't find crate).
#
# So the build reports its artifacts as JSON (diagnostics still rendered to stderr as before), and
# the search path is the directory of every library file it names, minus `debug/` itself. That
# directory holds only the UPLIFTED copies of the workspace's own libraries (`libfoo.rlib`, no
# hash), whose hashed twins sit in the unit's own directory beside their `.rmeta`; searching both
# would give rustc two candidates for one crate (E0464). On cargo before 1.100 this list is exactly
# `debug/deps`, so the script means the same thing on either side of the change.
CARGO_TARGET_DIR="$TARGET" cargo build --message-format=json-render-diagnostics > "$TARGET/artifacts.jsonl"
LIBRARY_PATH="$(node -e '
const fs = require("fs"), path = require("path");
const [file, uplift] = [process.argv[1], fs.realpathSync(process.argv[2])];
const dirs = new Set();
for (const line of fs.readFileSync(file, "utf8").split("\n")) {
    if (!line.startsWith("{")) continue;
    const m = JSON.parse(line);
    if (m.reason !== "compiler-artifact") continue;
    for (const f of m.filenames) {
        if (!/\.(rlib|rmeta|so|dylib|dll)$/.test(f)) continue;
        const dir = fs.realpathSync(path.dirname(f));
        if (dir !== uplift) dirs.add(dir);
    }
}
if (dirs.size === 0) { console.error("test-books: the build named no library artifacts"); process.exit(1); }
// mdbook takes the search path as ONE comma-separated list, so a comma in a directory would split it.
const bad = [...dirs].find((d) => d.includes(","));
if (bad) { console.error(`test-books: a library directory has a comma in it: ${bad}`); process.exit(1); }
console.log([...dirs].sort().join(","));
' "$TARGET/artifacts.jsonl" "$TARGET/debug")"

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
    if ! mdbook test "$book" -L "$LIBRARY_PATH"; then
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
    # narrower set, and before cargo 1.100 it put them into the same `deps/`, leaving a second
    # `ikigai_core` rlib there: the next book's `mdbook test` then failed with E0464. With one
    # book it never showed, because nothing ran after the last contrast check; the second book is
    # what exposed it. (The search path above now lists only this build's own units, but a second
    # compile is still a waste.)
    if ! "$TARGET/debug/book-a11y" "$book"; then
        echo "FAILED: $name (contrast)"
        status=1
    fi
done

exit "$status"
