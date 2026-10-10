//! What `cargo run -p loadable-module` prints is what every page says it prints (ledger #165).
//!
//! The demo's output is shown to the reader in three places: the repository README, Part II's
//! opening ("What you will build") and "A module, end to end". Nothing compiles a ```` ```text ````
//! block, so a stale one would sit there looking authoritative; `mdbook test` runs Rust blocks,
//! and the REPL replay (`crates/book-urns/tests/book_transcripts.rs`) runs `ikigai> ` blocks.
//! Until this file, the only thing that kept these three lines true was somebody running the
//! binary and looking (the 2026-09-12 rename changed all three copies by hand).
//!
//! This is the same check those make, for a program instead of a REPL: run the demo binary
//! (Cargo builds it for this test and names it in `CARGO_BIN_EXE_module-demo`, so no nested
//! `cargo run` and no binary on PATH), and diff its stdout against every copy. The scanner,
//! and the rules for how a page writes a run, are `book_transcripts::runs`, shared with the
//! other programs the books run (ledger #1072); this file supplies the binary.
//!
//! **One copy, where mdbook can share it.** The book's two chapters `{{#include}}`
//! `module-demo.stdout`, the file beside this crate's `Cargo.toml`, so the book holds the output
//! once. The README is rendered by GitHub, which has no includes, so it keeps a literal copy, and
//! this test is what keeps that copy in step.

use std::path::Path;
use std::process::Command;

use book_transcripts::runs;

/// The pages that show the demo's output, relative to the repository root. Adding a copy
/// anywhere is fine (the scan finds it); add it here too, so losing it later is noticed.
const EXPECTED_PAGES: &[&str] = &[
    "README.md",
    "books/ikigai/src/modules/end-to-end.md",
    "books/ikigai/src/modules/index.md",
];

/// The demo's stdout, by running the binary Cargo built for this test.
fn demo_stdout() -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_module-demo"))
        .output()
        .expect("the demo binary runs");
    assert!(
        out.status.success(),
        "module-demo failed ({}): {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("the demo prints UTF-8")
}

#[test]
fn the_shared_copy_is_what_the_demo_prints() {
    let shared = Path::new(env!("CARGO_MANIFEST_DIR")).join("module-demo.stdout");
    let text = std::fs::read_to_string(&shared).expect("module-demo.stdout is readable");
    assert_eq!(
        text,
        demo_stdout(),
        "crates/loadable-module/module-demo.stdout is not what the demo prints. The book \
         includes this file: regenerate it with\n    cargo run -q -p loadable-module > \
         crates/loadable-module/module-demo.stdout\nand update the README's literal copy to match"
    );
}

#[test]
fn every_page_shows_what_the_demo_prints() {
    runs::check(
        "loadable-module",
        env!("CARGO_BIN_EXE_module-demo"),
        EXPECTED_PAGES,
    );
}
