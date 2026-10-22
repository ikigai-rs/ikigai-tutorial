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
#   the starter is ikigai-gonk-starter, pinned to ONE git revision in `books/gonk/starter.rev`
#         (the same one-file shape as gonk.rev). Each "Ways of working" page ends by generating a
#         repository with the starter's `tools/new.py --process <profile>`, and it gets the
#         starter as a reader does: `git clone https://github.com/ikigai-rs/ikigai-gonk-starter`
#         then `git checkout` of the pinned revision, which the page names with
#         `{{#include ../../starter.rev}}` (the runner expands it as mdbook renders it). It is
#         cloned ONCE here, into `target/gonk-starter/<rev>/`, and the pages' `git clone` of that
#         URL is pointed at that local clone by git's own `url.<base>.insteadOf`, passed in the
#         environment (`GIT_CONFIG_COUNT`, so no config file is written in any home): eight pages
#         make no network fetch, and every one checks out exactly the pinned commit. Without the
#         redirect (the runner run by hand) the same page clones from GitHub and checks out the
#         same commit, so the redirect changes where the bytes come from, never which bytes.
#
#   model-stub is this workspace's own `crates/model-stub`: a deterministic `urn:llm:` peer on a
#         Unix socket, which "Scripts and sentences in gonk" and the "Ways of working" pages start
#         and mount so a scratch gonk drafts, explains and reviews without a real model or the
#         network. Built here and copied alone into
#         `target/model-stub/bin/`, so the page's PATH gains that one binary and not the rest of
#         `target/debug`.
#
# ⚠ Every page runs in a SCRATCH home (`HOME` and `XDG_CONFIG_HOME` point into a temporary
# directory), so nothing here reads or writes a real gonk's store, socket or grants; and a
# command naming port 1062 or 1060, ports a live service on this machine uses, is refused before
# it runs.
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

cargo build --quiet -p model-stub
stub_root="target/model-stub/bin"
mkdir -p "$stub_root"
cp target/debug/model-stub "$stub_root/"

starter_rev="$(tr -d '[:space:]' < books/gonk/starter.rev)"
case "$starter_rev" in
    *[!0-9a-f]*|'') echo "books/gonk/starter.rev is not a git revision: '$starter_rev'" >&2; exit 2 ;;
esac
if [ "${#starter_rev}" -ne 40 ]; then
    echo "books/gonk/starter.rev must be a full 40-character revision, not '$starter_rev'" >&2
    exit 2
fi
starter_url="https://github.com/ikigai-rs/ikigai-gonk-starter"
starter="target/gonk-starter/$starter_rev"
if [ ! -d "$starter/.git" ]; then
    echo "cloning ikigai-gonk-starter at $starter_rev into $starter (once per revision)"
    rm -rf "$starter.partial"
    git clone --quiet --no-checkout "$starter_url" "$starter.partial"
    git -C "$starter.partial" checkout --quiet --detach "$starter_rev"
    mv "$starter.partial" "$starter"
fi
# The cache's HEAD is a BRANCH at the pin, never a detached commit, and this runs every time so
# an older cache is repaired too. A reader's `git clone` from GitHub checks out a branch; a clone
# of a repository whose HEAD is detached prints git's detached-HEAD advice, and `--quiet` does
# not silence it. That advice then lands in the page's output as if the reader would see it. It
# stayed hidden while the starter's `main` was the pinned commit, because clone then names the
# matching branch; the starter's `main` moving past the pin is what exposed it.
git -C "$starter" checkout --quiet -B main "$starter_rev"

status=0

echo "── gonk (ikigai-gonk $rev, ikigai-cli $cli, starter $starter_rev) ──────────────────"
GIT_CONFIG_COUNT=1 \
    GIT_CONFIG_KEY_0="url.$PWD/$starter.insteadOf" GIT_CONFIG_VALUE_0="$starter_url" \
    cargo run --quiet -p book-transcripts -- books/gonk --strict --path "$root/bin" --path "$cli_root/bin" \
    --path "$stub_root" \
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
