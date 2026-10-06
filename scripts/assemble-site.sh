#!/usr/bin/env bash
# Put every built book where it is published, in ONE directory: the Pages artifact.
#
#   ./scripts/assemble-site.sh [DIR]     # default target/site; replaced
#
# Each book's place comes from its own `book.toml`: `site-url` is where it is served, and the
# shortest one is the site's root (`/ikigai-tutorial/`, The ikigai Book). Every other book goes
# under the root at the rest of its URL — The gonk Book's `/ikigai-tutorial/gonk/` lands at
# `gonk/`. So the layout is stated once per book, in the file mdbook already reads for it, and
# a third book needs no edit here.
#
# Run after the books are built (`./scripts/test-books.sh`) and their kernels are bound into
# them (pages.yml), because it copies `books/<name>/book/` as it is. The root book's files are
# copied first and byte for byte; a nested book whose place the root already uses is refused,
# rather than one overwriting the other.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."
out="${1:-target/site}"

site_url() {
    sed -n 's/^site-url *= *"\([^"]*\)".*/\1/p' "$1/book.toml"
}

# A book with no site-url has no place; say so here, where an exit stops the script (one
# inside the process substitution below would end only its subshell).
for book in books/*/; do
    if [ -f "$book/book.toml" ] && [ -z "$(site_url "${book%/}")" ]; then
        echo "${book}book.toml has no site-url, so it has no place in the site" >&2
        exit 1
    fi
done

# Every book, shortest site-url first: the root, then what nests under it.
books=()
while IFS= read -r line; do
    books+=("${line#* }")
done < <(for book in books/*/; do
    book="${book%/}"
    [ -f "$book/book.toml" ] || continue
    url="$(site_url "$book")"
    printf '%d %s\n' "${#url}" "$book"
done | sort -n)

root_url="$(site_url "${books[0]}")"
rm -rf "$out"
mkdir -p "$out"
for book in "${books[@]}"; do
    url="$(site_url "$book")"
    case "$url" in
        "$root_url"*) rel="${url#"$root_url"}" ;;
        *) echo "$book serves at $url, outside the site root $root_url" >&2; exit 1 ;;
    esac
    if [ ! -f "$book/book/index.html" ]; then
        echo "$book/book is not built — run ./scripts/test-books.sh first" >&2
        exit 1
    fi
    dest="$out/$rel"
    if [ -n "$rel" ] && [ -e "$dest" ]; then
        echo "$book belongs at ${rel}, which the root book already uses" >&2
        exit 1
    fi
    mkdir -p "$dest"
    cp -R "$book/book/." "$dest"
    echo "  $book/book → ${dest%/}  (${url})"
done
