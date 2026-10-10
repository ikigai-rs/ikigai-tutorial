//! What `cargo run -p hello-camel` prints is what every page says it prints (ledger #1072).
//!
//! Part I's host is the first program a reader runs: the README and "Running it" show both of
//! its forms, and "What resolution buys you" shows its catalog. The scanner and the rules for
//! how a page writes a run are `book_transcripts::runs`; this test supplies the binary, which
//! only a test of this package can (Cargo names it in `CARGO_BIN_EXE_tutorial`).

use book_transcripts::runs;

/// The pages that show this program's output, relative to the repository root. The scan finds
/// a new copy by itself; list it here too, so losing it later is noticed.
const EXPECTED_PAGES: &[&str] = &[
    "README.md",
    "books/ikigai/src/getting-started/payoff.md",
    "books/ikigai/src/getting-started/running-it.md",
];

#[test]
fn every_page_shows_what_the_tutorial_binary_prints() {
    runs::check(
        "hello-camel",
        env!("CARGO_BIN_EXE_tutorial"),
        EXPECTED_PAGES,
    );
}
