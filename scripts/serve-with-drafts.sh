#!/usr/bin/env bash
# Build and serve the book LOCALLY with its unpublished chapters linked in AND its in-page
# kernel built, so a review can actually press Run.
#
# A chapter that is written but not yet public lives in `books/ikigai/src/` like any
# other, but is left OUT of `SUMMARY.md`, so `mdbook` neither builds nor deploys it.
# This script copies the book to a temporary directory, appends the drafts listed in
# `books/ikigai/drafts.txt` to that copy's SUMMARY under a "Drafts" part, builds it, then
# builds the in-page kernel (`crates/book-wasm`) into `wasm/` beside it the way pages.yml
# does — the working tree and the deployed site are untouched.
#
#   ./scripts/serve-with-drafts.sh                 # build (book + kernel), serve on 127.0.0.1:3000
#   ./scripts/serve-with-drafts.sh serve 8080      # the same, on another port
#   ./scripts/serve-with-drafts.sh build           # build only, into target/drafts-book/
#   ./scripts/serve-with-drafts.sh build DIR       # build only, into DIR (replaced)
#   … --no-kernel                                  # skip the kernel: prose only, Run disabled
#
# WHY NOT `mdbook serve`. It never builds `crates/book-wasm`, so under it the kernel can
# never load and every Run is disabled — honest, but not reviewable. And it rebuilds (and
# wipes) its output directory on every change, which would delete a `wasm/` put there. So
# this builds once and serves the result statically with `python3 -m http.server`, bound
# to 127.0.0.1 (the wasm needs http; `file://` cannot load it). No live reload: edit, then
# run the script again.
#
# WHERE IT BUILDS. `target/drafts-book/` by default — deliberately NOT under
# `books/ikigai/book/`, which `./scripts/test-books.sh` and any `mdbook build` wipe: a
# server pointed there loses its kernel the next time the gates run, and the hub's did,
# exactly that way. `target/` is ignored by git, and `cargo clean` removes this too.
# CI uses the DIR form with `--no-kernel`: `a11y/no-kernel.mjs` checks the drafts' boards
# with the kernel absent, and the drafts must never land inside the directory pages.yml
# uploads.
#
# THE KERNEL NEEDS two things installed, and the script checks both before building
# anything: the `wasm32-unknown-unknown` target, and `wasm-bindgen-cli` at EXACTLY the
# version `crates/book-wasm/Cargo.toml` pins for the `wasm-bindgen` crate (read from that
# manifest here) — a mismatch is not a build error, it is a page whose kernel silently
# fails to load.
#
# drafts.txt: one chapter per line — `Title | relative/path.md` (relative to src/).
# Blank lines and lines starting with `#` are ignored.
set -euo pipefail

here="$(cd "$(dirname "$0")/.." && pwd)"
book="$here/books/ikigai"
drafts="$book/drafts.txt"

usage() {
    sed -n '12,16p' "$0" | sed 's/^# \{0,1\}//'
}

mode=serve
kernel=1
port=3000
out=""
positional=()
for arg in "$@"; do
    case "$arg" in
        serve|build) mode="$arg" ;;
        --no-kernel) kernel=0 ;;
        -h|--help) usage; exit 0 ;;
        -*) echo "unknown option: $arg" >&2; usage >&2; exit 2 ;;
        *) positional+=("$arg") ;;
    esac
done
if [ "${#positional[@]}" -gt 1 ]; then
    usage >&2
    exit 2
fi
if [ "${#positional[@]}" -eq 1 ]; then
    if [ "$mode" = serve ]; then
        port="${positional[0]}"
        case "$port" in ''|*[!0-9]*) echo "not a port: $port" >&2; exit 2 ;; esac
    else
        out="${positional[0]}"
    fi
fi
out="${out:-$here/target/drafts-book}"
# An absolute path, so the `mv` below and the server agree whatever the caller's cwd.
case "$out" in /*) ;; *) out="$PWD/$out" ;; esac

if [ ! -f "$drafts" ]; then
    echo "no $drafts — nothing is in draft" >&2
    exit 1
fi

# ── Check the kernel's toolchain FIRST, before minutes of building ────────────────────
if [ "$kernel" = 1 ]; then
    pin="$(sed -n 's/^wasm-bindgen *= *"=\{0,1\}\([0-9][0-9.]*\)".*/\1/p' "$here/crates/book-wasm/Cargo.toml")"
    if [ -z "$pin" ]; then
        echo "could not read the wasm-bindgen pin from crates/book-wasm/Cargo.toml" >&2
        exit 1
    fi
    missing=()
    if command -v rustup >/dev/null 2>&1 &&
        ! rustup target list --installed | grep -qx wasm32-unknown-unknown; then
        missing+=("rustup target add wasm32-unknown-unknown")
    fi
    have="$(wasm-bindgen --version 2>/dev/null | sed -n 's/^wasm-bindgen \([0-9][0-9.]*\).*/\1/p' || true)"
    if [ "$have" != "$pin" ]; then
        if [ -n "$have" ]; then
            echo "wasm-bindgen is $have; the book's kernel is pinned to $pin" >&2
        fi
        missing+=("cargo install --locked wasm-bindgen-cli --version $pin")
    fi
    if [ "${#missing[@]}" -gt 0 ]; then
        echo "The in-page kernel cannot be built here yet. Install:" >&2
        for cmd in "${missing[@]}"; do
            echo "    $cmd" >&2
        done
        echo "or pass --no-kernel to read the prose only (every Run disabled)." >&2
        exit 1
    fi
fi

# ── The book, with the drafts linked in ──────────────────────────────────────────────
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

mdbook build "$tmp"
rm -rf "$out"
mkdir -p "$(dirname "$out")"
mv "$tmp/book" "$out"
rm -rf "$tmp"

# ── The in-page kernel, exactly as pages.yml builds it ───────────────────────────────
if [ "$kernel" = 1 ]; then
    (cd "$here" && cargo build --release -p book-wasm --lib --target wasm32-unknown-unknown)
    wasm-bindgen --target web --out-dir "$out/wasm" \
        "$here/target/wasm32-unknown-unknown/release/book_wasm.wasm"
    if [ ! -s "$out/wasm/book_wasm_bg.wasm" ]; then
        echo "the kernel build produced no $out/wasm/book_wasm_bg.wasm" >&2
        exit 1
    fi
    echo "built with drafts and the in-page kernel: $out"
else
    echo "built with drafts, WITHOUT the in-page kernel (every Run disabled): $out"
fi

case "$mode" in
    build) ;;
    serve)
        echo "serving http://127.0.0.1:$port/ — Ctrl-C to stop; run the script again after an edit"
        exec python3 -m http.server "$port" --bind 127.0.0.1 --directory "$out"
        ;;
esac
