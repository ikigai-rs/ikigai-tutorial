#!/usr/bin/env bash
# Serve (or build) the book LOCALLY with its unpublished chapters linked in.
#
# A chapter that is written but not yet public lives in `books/ikigai/src/` like any
# other, but is left OUT of `SUMMARY.md`, so `mdbook` neither builds nor deploys it.
# This script copies the book to a temporary directory, appends the drafts listed in
# `books/ikigai/drafts.txt` to that copy's SUMMARY under a "Drafts" part, and runs
# mdbook there — the working tree and the deployed site are untouched.
#
#   ./scripts/serve-with-drafts.sh            # mdbook serve, opens a browser
#   ./scripts/serve-with-drafts.sh build      # mdbook build only; output under books/ikigai/book/with-drafts/
#
# drafts.txt: one chapter per line — `Title | relative/path.md` (relative to src/).
# Blank lines and lines starting with `#` are ignored.
set -euo pipefail

here="$(cd "$(dirname "$0")/.." && pwd)"
book="$here/books/ikigai"
drafts="$book/drafts.txt"
mode="${1:-serve}"

if [ ! -f "$drafts" ]; then
    echo "no $drafts — nothing is in draft" >&2
    exit 1
fi

# The copy lives BESIDE the real book, at the same depth, because chapters include code by
# relative path (`{{#include ../../../../crates/…}}`) and a copy in /tmp cannot reach it.
# `books/.drafts.*` is ignored by git.
tmp="$(mktemp -d "$here/books/.drafts.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT
cp -R "$book/." "$tmp/"
rm -rf "$tmp/book"

partfile="$tmp/drafts-part.md"
{
    printf '\n# Drafts (not yet public)\n\n'
    while IFS='|' read -r title path; do
        title="$(printf '%s' "${title:-}" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
        path="$(printf '%s' "${path:-}" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
        case "$title" in ''|'#'*) continue ;; esac
        if [ -z "$path" ] || [ ! -f "$tmp/src/$path" ]; then
            echo "draft chapter not found: src/$path (from $drafts)" >&2
            exit 1
        fi
        printf -- '- [%s](%s)\n' "$title" "$path"
    done < "$drafts"
} > "$partfile"

# mdbook refuses a numbered part AFTER the suffix chapters (the `---` block at the end of
# SUMMARY.md: "Where to go next", the glossary), so the drafts part goes in front of that
# separator; a summary with no suffix block gets it appended.
summary="$tmp/src/SUMMARY.md"
if grep -qx -- '---' "$summary"; then
    awk -v partfile="$partfile" 'BEGIN { done = 0 }
        /^---$/ && !done { while ((getline l < partfile) > 0) print l; print ""; done = 1 }
        { print }' "$summary" > "$summary.new" && mv "$summary.new" "$summary"
else
    cat "$partfile" >> "$summary"
fi

case "$mode" in
    serve) exec mdbook serve "$tmp" --open ;;
    build)
        mdbook build "$tmp"
        # Keep the output past the trap, INSIDE the ignored build dir of the real book, so
        # it is never committed and a real `mdbook build` sweeps it away.
        out="$book/book/with-drafts"
        rm -rf "$out"
        mkdir -p "$book/book"
        mv "$tmp/book" "$out"
        echo "built with drafts: $out/index.html"
        ;;
    *) echo "usage: $0 [serve|build]" >&2; exit 2 ;;
esac
