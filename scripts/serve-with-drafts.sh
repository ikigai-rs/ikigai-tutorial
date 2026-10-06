#!/usr/bin/env bash
# Build and serve the book LOCALLY with its unpublished chapters linked in AND its in-page
# kernel built, so a review can actually press Run.
#
# A chapter that is written but not yet public lives in `books/ikigai/src/` like any
# other, but is left OUT of `SUMMARY.md`, so `mdbook` neither builds nor deploys it.
# This script copies the book to a temporary directory, appends the drafts listed in
# `books/<book>/drafts.txt` (`books/ikigai/drafts.txt` by default) to that copy's SUMMARY under a "Drafts" part, builds it, then
# builds the in-page kernel (`crates/book-wasm`) into `wasm/` beside it the way pages.yml
# does — the working tree and the deployed site are untouched.
#
#   ./scripts/serve-with-drafts.sh                 # build (book + kernel), serve on 127.0.0.1:3000
#   ./scripts/serve-with-drafts.sh serve 8080      # the same, on another port
#   ./scripts/serve-with-drafts.sh build           # build only, into target/drafts-book/
#   ./scripts/serve-with-drafts.sh build DIR       # build only, into DIR (replaced)
#   … --no-kernel                                  # skip the kernel: prose only, Run disabled
#   … --book gonk                                  # another book (default: ikigai)
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
# anything: the `wasm32-unknown-unknown` standard library IN THE RUSTC THE BUILD WILL USE
# (`$RUSTC`, else the first `rustc` on PATH — which is not always rustup's; see the check),
# and `wasm-bindgen-cli` at EXACTLY the version `crates/book-wasm/Cargo.toml` pins for the
# `wasm-bindgen` crate (read from that manifest here) — a mismatch is not a build error, it
# is a page whose kernel silently fails to load.
#
# drafts.txt: one chapter per line — `Title | relative/path.md` (relative to src/).
# Blank lines and lines starting with `#` are ignored. A book other than the default may have
# none (The gonk Book does not, yet): it is then built with its kernel and no Drafts part, which
# is still the only local way to press its Run buttons.
set -euo pipefail

here="$(cd "$(dirname "$0")/.." && pwd)"
name=ikigai

usage() {
    sed -n '12,17p' "$0" | sed 's/^# \{0,1\}//'
}

mode=serve
kernel=1
port=3000
out=""
positional=()
want_book=0
for arg in "$@"; do
    if [ "$want_book" = 1 ]; then
        name="$arg"
        want_book=0
        continue
    fi
    case "$arg" in
        --book) want_book=1 ;;
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
book="$here/books/$name"
drafts="$book/drafts.txt"
if [ ! -f "$book/book.toml" ]; then
    echo "no book called $name (no $book/book.toml)" >&2
    exit 2
fi
out="${out:-$here/target/drafts-book}"
# An absolute path, so the `mv` below and the server agree whatever the caller's cwd.
case "$out" in /*) ;; *) out="$PWD/$out" ;; esac

# The default book always has a drafts list, so its absence there is a broken checkout rather
# than a book with nothing in draft.
if [ ! -f "$drafts" ] && [ "$name" = ikigai ]; then
    echo "no $drafts — nothing is in draft" >&2
    exit 1
fi

# ── Check the kernel's toolchain FIRST, before minutes of building ────────────────────
#
# ASK THE COMPILER THE BUILD WILL USE, NOT RUSTUP. cargo compiles with `$RUSTC` when it is
# set, else with the first `rustc` on PATH — and that need not be rustup's. On a Mac with
# Homebrew's Rust ahead of `~/.cargo/bin` on PATH, rustup's toolchain has the wasm32
# target (so `rustup target list --installed` says yes), while the build runs Homebrew's
# rustc, which ships no wasm32 standard library: E0463 "can't find crate for `core`" on
# every crate, minutes later, telling the reader to run a command they already ran
# (ledger #590). So the check asks that rustc where its wasm32 libraries live and looks for
# `core` there, which means the same thing under rustup, Homebrew or a distro Rust.
wasm_target=wasm32-unknown-unknown
wasm_libdir=""

# Does compiler "$1" have a wasm32 `core` to link against? Sets `wasm_libdir` to where it
# looked. Asked from the repository root, so a toolchain override there applies as it will
# for the build. (`--print target-libdir` prints the path whether or not it exists.)
has_wasm_core() {
    wasm_libdir="$(cd "$here" && "$1" --print target-libdir --target "$wasm_target" 2>/dev/null)" ||
        return 1
    [ -n "$wasm_libdir" ] || return 1
    for rlib in "$wasm_libdir"/libcore-*.rlib; do
        [ -e "$rlib" ] && return 0
    done
    return 1
}

sysroot_of() {
    (cd "$here" && "$1" --print sysroot 2>/dev/null) || true
}

if [ "$kernel" = 1 ]; then
    pin="$(sed -n 's/^wasm-bindgen *= *"=\{0,1\}\([0-9][0-9.]*\)".*/\1/p' "$here/crates/book-wasm/Cargo.toml")"
    if [ -z "$pin" ]; then
        echo "could not read the wasm-bindgen pin from crates/book-wasm/Cargo.toml" >&2
        exit 1
    fi
    found=()    # what was found and why it is not enough, printed first
    missing=()  # install commands

    if [ -n "${RUSTC:-}" ]; then
        rustc_cmd="$RUSTC"
        rustc_via="\$RUSTC, which cargo uses in place of the rustc on PATH"
    else
        rustc_cmd=rustc
        rustc_via="the first rustc on PATH"
    fi
    rustc_path="$(command -v "$rustc_cmd" 2>/dev/null || true)"
    rustup_rustc=""
    if command -v rustup >/dev/null 2>&1; then
        rustup_rustc="$(cd "$here" && rustup which rustc 2>/dev/null || true)"
    fi

    if [ -z "$rustc_path" ]; then
        found+=("No rustc found (looked for $rustc_via: $rustc_cmd).")
        if [ -z "$rustup_rustc" ]; then
            missing+=("Rust, through rustup: https://rustup.rs")
        fi
        missing+=("rustup target add $wasm_target")
    elif ! has_wasm_core "$rustc_path"; then
        version="$(cd "$here" && "$rustc_path" --version 2>/dev/null || echo 'version unknown')"
        found+=("The build compiles with $rustc_path ($version),")
        found+=("$rustc_via. It has no $wasm_target standard library:")
        found+=("no libcore in ${wasm_libdir:-its target-libdir (rustc would not say where)}.")
        if [ -n "$rustup_rustc" ] &&
            [ "$(sysroot_of "$rustup_rustc")" != "$(sysroot_of "$rustc_path")" ]; then
            # rustup is installed, and the build's rustc is not the one it would choose.
            if has_wasm_core "$rustup_rustc"; then
                rustup_has="rustup's toolchain HAS the target, so \`rustup target add\` changes nothing"
            else
                rustup_has="rustup's toolchain lacks the target as well"
                missing+=("rustup target add $wasm_target")
            fi
            if [ -n "${RUSTC:-}" ]; then
                found+=("")
                found+=("That is not rustup's rustc ($rustup_rustc), and $rustup_has.")
                found+=("Unset RUSTC, or point it at a rustc that has the target.")
            else
                bin="$(dirname "$(command -v rustup)")"
                case "$bin" in "${HOME:-/nonexistent}"/*) bin="\$HOME${bin#"$HOME"}" ;; esac
                found+=("")
                found+=("That is not rustup's rustc ($rustup_rustc): another Rust — Homebrew's,")
                found+=("for example — comes before rustup's $bin on PATH and shadows it,")
                found+=("and $rustup_has.")
                found+=("Put $bin first on PATH, in fish:")
                found+=("    fish_add_path --move $bin")
                found+=("in bash or zsh (in the shell's startup file):")
                found+=("    export PATH=\"$bin:\$PATH\"")
                found+=("or remove the other Rust. For this one run only, in any of them:")
                found+=("    env PATH=\"$bin:\$PATH\" ./scripts/serve-with-drafts.sh")
            fi
        elif [ -n "$rustup_rustc" ]; then
            # rustup is in charge; the target is simply not installed.
            missing+=("rustup target add $wasm_target")
        else
            found+=("rustup is not installed, and a Rust that did not come from rustup may not")
            found+=("ship a wasm32 standard library at all (Homebrew's does not). Install rustup,")
            found+=("put its bin directory first on PATH, then add the target.")
            missing+=("Rust, through rustup: https://rustup.rs")
            missing+=("rustup target add $wasm_target")
        fi
    fi

    have="$(wasm-bindgen --version 2>/dev/null | sed -n 's/^wasm-bindgen \([0-9][0-9.]*\).*/\1/p' || true)"
    if [ "$have" != "$pin" ]; then
        if [ -n "$have" ]; then
            found+=("wasm-bindgen is $have; the book's kernel is pinned to $pin.")
        fi
        missing+=("cargo install --locked wasm-bindgen-cli --version $pin")
    fi

    if [ "${#found[@]}" -gt 0 ] || [ "${#missing[@]}" -gt 0 ]; then
        echo "The in-page kernel cannot be built here yet." >&2
        if [ "${#found[@]}" -gt 0 ]; then
            echo >&2
            printf '%s\n' "${found[@]}" >&2
        fi
        if [ "${#missing[@]}" -gt 0 ]; then
            echo >&2
            echo "Install:" >&2
            printf '    %s\n' "${missing[@]}" >&2
        fi
        echo >&2
        echo "Or pass --no-kernel to read the prose only (every Run disabled)." >&2
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

# The book's own list came with the copy; a book with none gets no Drafts part at all.
drafts="$tmp/drafts.txt"
entries="$tmp/drafts-entries.md"
: > "$entries"
if [ -f "$drafts" ]; then
    while IFS='|' read -r title path; do
        title="$(printf '%s' "${title:-}" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
        path="$(printf '%s' "${path:-}" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
        case "$title" in ''|'#'*) continue ;; esac
        if [ -z "$path" ] || [ ! -f "$tmp/src/$path" ]; then
            echo "draft chapter not found: src/$path (from $book/drafts.txt)" >&2
            exit 1
        fi
        printf -- '- [%s](%s)\n' "$title" "$path"
    done < "$drafts" > "$entries"
fi

if [ -s "$entries" ]; then
    partfile="$tmp/drafts-part.md"
    {
        printf '\n# Drafts (not yet public)\n\n'
        cat "$entries"
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
fi

mdbook build "$tmp"
rm -rf "$out"
mkdir -p "$(dirname "$out")"
mv "$tmp/book" "$out"
rm -rf "$tmp"

# ── The in-page kernel, exactly as pages.yml builds it ───────────────────────────────
if [ "$kernel" = 1 ]; then
    "$here/scripts/build-kernels.sh" "$name" "$out/wasm"
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
