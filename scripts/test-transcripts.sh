#!/usr/bin/env bash
# Run every shell transcript in the books that are checked this way, against the binaries
# they name, and fail on any that does not say what its command prints (ledger #165).
#
# What runs where is `crates/book-transcripts` (its docs say what a block declares and what
# "matches" means). This script supplies the binaries:
#
#   gonk  is pinned to ONE git revision, in `books/gonk/gonk.rev` — the file the book prints
#         its install line from and CI keys its cache on, so moving gonk is a one-line change.
#         It is installed here, `--locked`, into `target/gonk/<rev>/` (ignored by git) unless
#         that build is already there. The first install compiles RocksDB: minutes, once.
#
# ⚠ Every page runs in a SCRATCH home (`HOME` and `XDG_CONFIG_HOME` point into a temporary
# directory), so nothing here reads or writes a real gonk's store, socket or grants; and a
# command naming port 1060, a real gonk's, is refused before it runs.
#
# The ikigai book is not checked this way yet: its REPL transcripts are replayed by
# `crates/book-urns/tests/book_transcripts.rs` against an installed `ikigai`, `#[ignore]`d
# because CI has none. Adopting this check is a pin of `ikigai-cli` beside gonk's and one more
# line below; its `ikigai> ` blocks would become `console` blocks of `$ ikigai -c …`.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

rev="$(tr -d '[:space:]' < books/gonk/gonk.rev)"
case "$rev" in
    *[!0-9a-f]*|'') echo "books/gonk/gonk.rev is not a git revision: '$rev'" >&2; exit 2 ;;
esac
if [ "${#rev}" -ne 40 ]; then
    echo "books/gonk/gonk.rev must be a full 40-character revision, not '$rev'" >&2
    exit 2
fi

root="target/gonk/$rev"
if [ ! -x "$root/bin/ikigai-gonk" ]; then
    echo "installing ikigai-gonk at $rev into $root (once per revision)"
    cargo install --locked --git https://github.com/ikigai-rs/ikigai-gonk --rev "$rev" \
        --root "$root" --target-dir target/gonk/build ikigai-gonk
fi

echo "── gonk (ikigai-gonk $rev) ──────────────────"
cargo run --quiet -p book-transcripts -- books/gonk --strict --path "$root/bin"
