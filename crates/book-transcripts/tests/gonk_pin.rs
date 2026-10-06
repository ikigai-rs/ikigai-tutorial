//! gonk's revision lives in ONE file, and everything that names gonk reads it from there.
//!
//! `books/gonk/gonk.rev` is what `scripts/test-transcripts.sh` installs and CI keys its cache
//! on; the book prints its install line from it with `{{#include}}`. A second spelling of the
//! revision anywhere in the book is a pin that would not move when the file does.

use std::path::{Path, PathBuf};

fn book() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../books/gonk")
}

fn rev() -> String {
    std::fs::read_to_string(book().join("gonk.rev")).expect("books/gonk/gonk.rev")
}

#[test]
fn the_pin_is_one_full_revision_on_one_line() {
    let text = rev();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 1, "one line: {text:?}");
    let rev = lines[0];
    assert_eq!(rev.len(), 40, "a full revision, not an abbreviation: {rev}");
    assert!(rev
        .chars()
        .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
}

#[test]
fn the_book_installs_gonk_from_the_pin_and_spells_no_revision_itself() {
    let rev = rev();
    let rev = rev.trim();
    let mut includes = 0;
    for page in book_transcripts::pages(&book().join("src")).expect("the book's pages") {
        let text = std::fs::read_to_string(&page).expect("a readable page");
        includes += text.matches("--rev {{#include ../gonk.rev}}").count();
        assert!(
            !text.contains(rev) && !text.contains(&rev[..7]),
            "{} spells gonk's revision itself; include books/gonk/gonk.rev instead",
            page.display()
        );
    }
    assert!(
        includes >= 1,
        "no page installs gonk from books/gonk/gonk.rev"
    );
}

/// The `ikigai` the transcripts run is pinned the same way: one version in one file,
/// `books/gonk/ikigai-cli.version`, which the script installs and CI keys its cache on.
fn cli_version() -> String {
    std::fs::read_to_string(book().join("ikigai-cli.version"))
        .expect("books/gonk/ikigai-cli.version")
}

#[test]
fn the_cli_pin_is_one_exact_version_on_one_line() {
    let text = cli_version();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 1, "one line: {text:?}");
    let parts: Vec<&str> = lines[0].split('.').collect();
    assert_eq!(parts.len(), 3, "MAJOR.MINOR.PATCH, nothing else: {text:?}");
    assert!(
        parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit())),
        "an exact version, not a requirement: {text:?}"
    );
}

#[test]
fn the_book_installs_the_cli_from_the_pin() {
    let mut includes = 0;
    for page in book_transcripts::pages(&book().join("src")).expect("the book's pages") {
        let text = std::fs::read_to_string(&page).expect("a readable page");
        includes += text
            .matches(
                "cargo install --locked ikigai-cli --version {{#include ../ikigai-cli.version}}",
            )
            .count();
    }
    assert!(
        includes >= 1,
        "no page installs ikigai-cli from books/gonk/ikigai-cli.version"
    );
}
