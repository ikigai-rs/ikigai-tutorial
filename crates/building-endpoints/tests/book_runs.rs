//! What `cargo run -p building-endpoints` prints is what every page says it prints
//! (ledger #1072).
//!
//! Three chapters of Building endpoints end on this host from a shell, one flag each:
//! `--strings` ("The graph face"), `--markdown` ("Transreption") and `--banner`
//! ("Configuration", whose fence first writes the config file it reads; the check runs that
//! setup too, in a scratch config home). The scanner and the rules for how a page writes a run
//! are `book_transcripts::runs`; this test supplies the binary, which only a test of this
//! package can (Cargo names it in `CARGO_BIN_EXE_building`).

use book_transcripts::runs;

/// The pages that show this program's output, relative to the repository root. The scan finds
/// a new copy by itself; list it here too, so losing it later is noticed.
const EXPECTED_PAGES: &[&str] = &[
    "books/ikigai/src/building/graph-face.md",
    "books/ikigai/src/building/transreption.md",
    "books/ikigai/src/getting-started/configuration.md",
];

#[test]
fn every_page_shows_what_the_building_binary_prints() {
    runs::check(
        "building-endpoints",
        env!("CARGO_BIN_EXE_building"),
        EXPECTED_PAGES,
    );
}
