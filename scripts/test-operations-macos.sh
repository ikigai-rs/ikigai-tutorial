#!/usr/bin/env bash
# macOS-only prerequisites for Running ikigai's planned installation/unattended listings.
# No launchctl, system installation, or changes to the user's configuration.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
if [ "$(uname -s)" != Darwin ]; then
    echo 'test-operations-macos: requires macOS' >&2
    exit 2
fi
version="$(tr -d '[:space:]' < books/operations/ikigai-cli.version)"
root="target/ikigai-ops-cli/$version+quic+web"
if [ ! -x "$root/bin/ikigai" ]; then
    cargo install --locked ikigai-cli --version "=$version" --features quic,web \
        --root "$root" --target-dir target/operations-macos-build
fi
"$root/bin/ikigai" --version
cargo test -p book-transcripts
cargo run --quiet -p book-transcripts -- crates/book-transcripts/tests/fixtures/macos \
    --strict --path "$root/bin"
cargo run --quiet -p book-transcripts -- crates/book-transcripts/tests/fixtures/serving \
    --strict --path "$root/bin"
