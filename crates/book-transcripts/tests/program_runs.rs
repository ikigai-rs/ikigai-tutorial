//! Every `cargo run` a page tells a reader to type is checked, or says why it is not
//! (ledger #1072).
//!
//! Each program's check lives in that program's crate, because only there does Cargo hand a
//! test the program's binary (`book_transcripts::runs` says why, and how a page is written).
//! This is the other half, and it needs no binary: scan every page, and fail on a run whose
//! package nothing checks and whose fence does not declare `<!-- transcript: manual — why -->`.
//! Without it a new chapter's run block would be skipped by every check there is, silently.

use book_transcripts::runs::{repo_root, run_blocks, unchecked};
use book_transcripts::Kind;

#[test]
fn every_cargo_run_on_a_page_is_checked_or_declared_manual() {
    let root = repo_root();
    let problems = unchecked(&root);
    assert!(problems.is_empty(), "\n{}", problems.join("\n\n"));
    for block in run_blocks(&root).expect("every page scans") {
        if let Kind::Manual(reason) = &block.kind {
            eprintln!("manual  {}:{}  {reason}", block.page, block.line);
        }
    }
}
