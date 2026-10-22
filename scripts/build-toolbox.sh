#!/usr/bin/env bash
# Build the RDF Toolbox (ikigai-toolbox's `build.sh` output) at the revision the ikigai book
# pins, and publish it beside the book: `books/ikigai/book/toolbox/`.
#
#   ./scripts/build-toolbox.sh            # into books/ikigai/book/toolbox (after test-books.sh)
#   ./scripts/build-toolbox.sh <dir>      # into <dir> (serve-with-drafts)
#
# "Checking a resource" opens the toolbox in place: the book's in-page kernel binds
# `urn:iki:toolbox:dist:{path}` to these files (crates/book-wasm, `dist_space`), so
# `urn:toolbox:check:{iri}` composes in the page from the build's own template, and the page's
# scripts and its 8.3 MB engine load from here, only when a reader opens a check.
#
# The pin is `books/ikigai/toolbox.rev`: an ikigai-toolbox COMMIT (a tag can move), the one
# whose crate the book links (`ikigai-toolbox` 0.1.0 is tag v0.1.0). It is cloned once into
# `~/.cache/ikigai-tutorial/toolbox/<rev>/` (see below) with its submodule (turtle-lsp, pinned
# by ikigai-toolbox), and its `build.sh` runs there every time: it refuses a submodule that is not at its pin, builds the
# engine with upstream's toolchain (Rust 1.99.0, from turtle-lsp's rust-toolchain.toml) into
# `target/engine-<commit>` inside the clone, which pages.yml caches by the submodule commit,
# applies ikigai-toolbox's patches and writes MANIFEST.jsonld. Nothing is downloaded from
# upstream's site, and nothing here is conditional: a build that fails stops the books job.
#
# Needs: git, rustup (the toolchain and the wasm32-unknown-unknown target for it are installed
# here when missing), node (build.sh writes the manifest with it).
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."
here="$(pwd)"

rev="$(tr -d '[:space:]' < books/ikigai/toolbox.rev)"
case "$rev" in
    *[!0-9a-f]* | "") echo "books/ikigai/toolbox.rev is not a commit: '$rev'" >&2; exit 1 ;;
esac
[ "${#rev}" -eq 40 ] || { echo "books/ikigai/toolbox.rev must be a full 40-hex commit" >&2; exit 1; }

out="${1:-books/ikigai/book/toolbox}"
if [ "$#" -eq 0 ] && [ ! -f books/ikigai/book/index.html ]; then
    echo "books/ikigai/book is not built — run ./scripts/test-books.sh first" >&2
    exit 1
fi

# OUTSIDE this repository: cargo looks upward from the engine's manifest for a workspace, and a
# clone under this repo's `target/` would find this workspace's Cargo.toml and refuse to build
# ("current package believes it's in a workspace when it's not"). Locally `target/` may be a
# symlink elsewhere, which hides that; CI's is not.
clone="${XDG_CACHE_HOME:-$HOME/.cache}/ikigai-tutorial/toolbox/$rev"
if [ ! -d "$clone/.git" ]; then
    rm -rf "$clone"
    mkdir -p "$(dirname "$clone")"
    git clone --quiet https://github.com/ikigai-rs/ikigai-toolbox "$clone"
fi
git -C "$clone" -c advice.detachedHead=false checkout --quiet "$rev"
git -C "$clone" submodule update --init --quiet
sub="$clone/vendor/turtle-lsp"

# A fresh clone stamps every file with the current time, so cargo would see the engine's path
# crates as changed and rebuild them even when pages.yml restored their build from the cache.
# The cache is keyed by the submodule commit, so these ARE the cached sources: give them that
# commit's own time, which predates every cached fingerprint (ikigai-toolbox's ci.yml does the
# same). `touch -d` with an ISO time reads the same in GNU and BSD touch.
when="$(TZ=UTC git -C "$sub" log -1 --format=%cd --date=format-local:%Y-%m-%dT%H:%M:%S)"
(cd "$sub" && git ls-files -z | TZ=UTC xargs -0 touch -h -d "$when")

# Upstream's toolchain, and its wasm32 target (rustup reads rust-toolchain.toml in that tree).
(cd "$sub" && rustup toolchain install --no-self-update >/dev/null && rustup target add wasm32-unknown-unknown >/dev/null)

started=$SECONDS
(cd "$clone" && ./build.sh >/dev/null)
echo "  ikigai-toolbox ${rev:0:12}: build.sh in $((SECONDS - started))s"

# Exactly the files the toolbox serves (its `FILES` allowlist), each one checked, so a renamed or
# missing file fails here rather than as a panel that never loads.
files=(toolbox.html page.template.html app.js samples.js worker.js engine.js style.css tokens.css
    engine.wasm LICENSE.turtle-lsp MANIFEST.jsonld)
rm -rf "$out"
mkdir -p "$out"
for f in "${files[@]}"; do
    test -s "$clone/dist/$f" || { echo "the toolbox build has no $f" >&2; exit 1; }
    cp "$clone/dist/$f" "$out/$f"
done
wasm_bytes="$(wc -c < "$out/engine.wasm" | tr -d ' ')"
echo "  toolbox → $out  (engine.wasm $wasm_bytes bytes)"
cd "$here"
