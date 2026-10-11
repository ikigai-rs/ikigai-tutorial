//! What `cargo run -p script-authority` prints is what every page says it prints (ledger #1072).
//!
//! The scanner and the rules for how a page writes a run are `book_transcripts::runs`; this
//! test supplies the binary, which only a test of this package can (Cargo names it in
//! `CARGO_BIN_EXE_authority`).

use book_transcripts::runs;

/// The pages that show this program's output, relative to the repository root. The scan finds
/// a new copy by itself; list it here too, so losing it later is noticed.
const EXPECTED_PAGES: &[&str] = &["books/ikigai/src/code/authority.md"];

#[test]
fn every_page_shows_what_the_authority_binary_prints() {
    runs::check(
        "script-authority",
        env!("CARGO_BIN_EXE_authority"),
        EXPECTED_PAGES,
    );
}
