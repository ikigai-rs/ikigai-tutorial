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
#         ⚠ Each revision builds in its OWN target directory (`target/gonk/<rev>/build`). With
#         one build directory shared across revisions, `cargo install --git --rev <new>` reported
#         the new revision "Fresh" and installed the OLD binary (seen moving 5050879 → 9b182ff,
#         2026-10-06): the pin moved and the check went on testing the previous gonk.
#
#   ikigai is `ikigai-cli` from crates.io, pinned to ONE version in `books/gonk/ikigai-cli.version`
#         (the same one-file shape: the book prints its install line from it, CI keys its cache
#         on it). Choose the release whose transports gonk's lock takes (`ikigai-ipc`, `-wire`,
#         `-web` share the cli's version), so the client a page runs speaks the wire the pinned
#         gonk serves. Installed `--locked` into `target/ikigai-cli/<version>/`.
#
# ⚠ Every page runs in a SCRATCH home (`HOME` and `XDG_CONFIG_HOME` point into a temporary
# directory), so nothing here reads or writes a real gonk's store, socket or grants; and a
# command naming port 1060, a real gonk's, is refused before it runs.
#
# The ikigai book's transcripts are REPL blocks (`ikigai> …`), not `console` blocks, and are
# replayed by `crates/book-urns/tests/book_transcripts.rs`, which is `#[ignore]`d because it
# needs an `ikigai` on PATH. This script supplies one and runs it (ledger #1023):
#
#   ikigai (for the ikigai Book) is `ikigai-cli` pinned SEPARATELY, in
#         `books/ikigai/ikigai-cli.version`: the gonk Book's pin follows gonk's lock, and the
#         ikigai Book's follows the release its pages were last replayed against, so they move
#         for different reasons. Default features (no page needs QUIC). Installed `--locked`
#         into `target/ikigai-book-cli/<version>/`, a root of its own so neither book's cache
#         key can restore or save the other's binary, and put FIRST on PATH for the test.
#
# The same binary then runs `crates/book-urns/tests/book_urns.rs`'s CLI-vocabulary probe
# (ledger #1031), `#[ignore]`d for the same reason: it asks that `ikigai` for every exact name in
# `books/ikigai/cli-vocabulary.txt`, the book's written-down claim about the CLI, so the claim is
# checked against the pinned release on every commit and moving the pin re-checks it.
#
# Every check runs whatever the ones before it say, and the script fails if any does.
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
        --root "$root" --target-dir "$root/build" ikigai-gonk
fi

cli="$(tr -d '[:space:]' < books/gonk/ikigai-cli.version)"
case "$cli" in
    [0-9]*.[0-9]*.[0-9]*) ;;
    *) echo "books/gonk/ikigai-cli.version is not a version: '$cli'" >&2; exit 2 ;;
esac
# With the `quic` feature, which is not a default: "A QUIC client" connects to gonk's QUIC door
# with `ikigai --connect quic://…`, and a cli built without it refuses `quic://` targets. The
# feature is part of the directory name, so a build without it is never mistaken for one with.
cli_root="target/ikigai-cli/$cli+quic"
if [ ! -x "$cli_root/bin/ikigai" ]; then
    echo "installing ikigai-cli $cli (features: quic) into $cli_root (once per version)"
    cargo install --locked ikigai-cli --version "=$cli" --features quic \
        --root "$cli_root" --target-dir "$cli_root/build"
fi

book_cli="$(tr -d '[:space:]' < books/ikigai/ikigai-cli.version)"
case "$book_cli" in
    [0-9]*.[0-9]*.[0-9]*) ;;
    *) echo "books/ikigai/ikigai-cli.version is not a version: '$book_cli'" >&2; exit 2 ;;
esac
book_cli_root="target/ikigai-book-cli/$book_cli"
if [ ! -x "$book_cli_root/bin/ikigai" ]; then
    echo "installing ikigai-cli $book_cli into $book_cli_root (once per version)"
    cargo install --locked ikigai-cli --version "=$book_cli" \
        --root "$book_cli_root" --target-dir "$book_cli_root/build"
fi

status=0

echo "── gonk (ikigai-gonk $rev, ikigai-cli $cli) ──────────────────"
cargo run --quiet -p book-transcripts -- books/gonk --strict --path "$root/bin" --path "$cli_root/bin" \
    || status=1

# The test runs `ikigai` by name, so the pinned one goes first on PATH; `--version` is printed
# so the log says which binary the pages were replayed against.
echo "── ikigai (ikigai-cli $book_cli) ──────────────────"
PATH="$PWD/$book_cli_root/bin:$PATH"
ikigai --version
cargo test -p book-urns --test book_transcripts -- --ignored --nocapture || status=1

echo "── ikigai CLI vocabulary (ikigai-cli $book_cli) ──────────────────"
cargo test -p book-urns --test book_urns -- --ignored --nocapture || status=1

exit "$status"
