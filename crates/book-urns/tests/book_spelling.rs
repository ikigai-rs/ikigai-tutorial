//! The book, and the code it teaches, are written in American English.
//!
//! Not a style preference review can hold: the words drift in from briefs, from quoted
//! sources and from habit, and a relapse is invisible in a diff that is about something
//! else. So this fails on a short denylist of spellings that are British and nothing
//! else — no word on it has an American reading, so a hit is never a false one.
//!
//! ⚠ **The matcher and the marker are ikigai-core's** (`crates/ikigai-core/tests/
//! american_spelling.rs`, ikigai-rs/ikigai-core PR #131), kept the same on purpose so
//! the two guards read the same. What differs is only what is scanned, and one short
//! list of words this repository has had to refuse ([`LOCAL_EXACT`]).
//!
//! Scanned: the books' pages, scripts and styles (`books/ikigai/{src,css,js}`, and
//! `books/gonk/src` with that book's `DESIGN.md`), every
//! crate's sources, READMEs, templates and static files, the root `README.md`, and
//! `docs/`, `scripts/`, `a11y/` and `.github/`. The scripts and styles are in scope
//! because that is where half of the first sweep's hits were (`a11y.css`, `run.js`);
//! `center` is an American word, so a CSS property never matches. Skipped: build output
//! (`target`), installed packages (`node_modules`) and vendored third-party files
//! (`books/ikigai/src/vendor/`), which stay byte for byte upstream's. Words are split on
//! anything that is not a letter, so a `snake_case` name is checked part by part and
//! `aria-labelledby` is `labelledby`, which is not on the list.
//!
//! **A direct quote keeps its own spelling.** Put `spelling: quote` anywhere on the
//! line — in Markdown an HTML comment (`<!-- spelling: quote (the paper's own title) -->`)
//! renders as nothing — and the line is skipped.
//!
//! This file is excluded from the scan, since it has to spell the denylist out.

use std::fs;
use std::path::{Path, PathBuf};

/// Any word that begins with one of these is British: behaviour(al), colour(ful),
/// honour(ed), neighbour(hood), favour(ite), flavour(s).
const OUR_STEMS: [&str; 6] = [
    "behaviour",
    "colour",
    "honour",
    "neighbour",
    "favour",
    "flavour",
];

/// `-ise` stems that are British only with one of `ISE_SUFFIXES` after them. Stem and
/// suffix together, so `realism` and `formalism` never match.
const ISE_STEMS: [&str; 11] = [
    "realis",
    "serialis",
    "normalis",
    "initialis",
    "recognis",
    "organis",
    "summaris",
    "authoris",
    "optimis",
    "memois",
    "categoris",
];
const ISE_SUFFIXES: [&str; 9] = [
    "e", "ed", "es", "ing", "ation", "ations", "er", "ers", "ably",
];

/// Whole words — ikigai-core's list, unchanged.
const EXACT: [&str; 17] = [
    "modelled",
    "modelling",
    "labelled",
    "labelling",
    "travelled",
    "travelling",
    "cancelled",
    "cancelling",
    "artefact",
    "artefacts",
    "catalogue",
    "catalogues",
    "centre",
    "centres",
    "analyse",
    "analysed",
    "analysing",
];

/// Whole words this repository has had to refuse beyond core's list. `noughts`: the
/// game in the book is tic-tac-toe, and its British name appears once, as a marked note.
/// The rest were found in this tree by the first sweep (or named in its brief).
const LOCAL_EXACT: [&str; 4] = ["noughts", "licence", "enrolment", "whilst"];

const QUOTE_MARKER: &str = "spelling: quote";

/// Where we write, relative to the repository root. A file is scanned as itself.
const ROOTS: [&str; 11] = [
    "README.md",
    "books/ikigai/src",
    // The gonk Book's own pages and its design note. Its css/ and js/ are symlinks to the
    // ikigai book's, scanned above.
    "books/gonk/src",
    "books/gonk/DESIGN.md",
    "books/ikigai/css",
    "books/ikigai/js",
    "crates",
    "docs",
    "scripts",
    "a11y",
    ".github",
];

const EXTENSIONS: [&str; 11] = [
    "md", "rs", "css", "js", "mjs", "html", "toml", "yml", "yaml", "sh", "txt",
];

const SKIP_DIRS: [&str; 3] = ["target", "node_modules", "vendor"];

fn is_british(word: &str) -> bool {
    let w = word.to_ascii_lowercase();
    OUR_STEMS.iter().any(|s| w.starts_with(s))
        || EXACT.contains(&w.as_str())
        || LOCAL_EXACT.contains(&w.as_str())
        || ISE_STEMS.iter().any(|s| {
            w.strip_prefix(s)
                .is_some_and(|rest| ISE_SUFFIXES.contains(&rest))
        })
}

/// Every British word in `text`, as `file:line: word`, skipping lines marked as quotes.
fn hits_in(file: &str, text: &str) -> Vec<String> {
    let mut hits = Vec::new();
    for (n, line) in text.lines().enumerate() {
        if line.contains(QUOTE_MARKER) {
            continue;
        }
        for word in line.split(|c: char| !c.is_ascii_alphabetic()) {
            if !word.is_empty() && is_british(word) {
                hits.push(format!("{file}:{}: {word}", n + 1));
            }
        }
    }
    hits
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the workspace root is two levels up from this crate")
}

fn collect(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_dir() {
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| SKIP_DIRS.contains(&n))
        {
            return;
        }
        let mut entries: Vec<PathBuf> = fs::read_dir(path)
            .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
            .map(|e| e.expect("a directory entry").path())
            .collect();
        entries.sort();
        for entry in entries {
            collect(&entry, out);
        }
    } else if path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| EXTENSIONS.contains(&e))
    {
        out.push(path.to_path_buf());
    }
}

#[test]
fn the_denylist_matches_what_it_should_and_nothing_else() {
    for british in [
        "behaviour",
        "Behavioural",
        "colours",
        "honoured",
        "neighbourhood",
        "realised",
        "Realisation",
        "serialisers",
        "modelled",
        "artefact",
        "Centre",
        "labelled",
        "noughts",
        "licence",
    ] {
        assert!(is_british(british), "{british} should be refused");
    }
    for fine in [
        "behavior",
        "realism",
        "formalism",
        "realize",
        "serialize",
        "analysis",
        "center",
        "drawCentredString",
        "modeled",
        "otherwise",
        "promise",
        "labelledby",
        "license",
        "enrolled",
    ] {
        assert!(!is_british(fine), "{fine} is not British-only");
    }
}

#[test]
fn a_quote_marker_skips_its_line_and_only_its_line() {
    let text = "the colour <!-- spelling: quote (their words) -->\nthe colour_pair\n";
    assert_eq!(hits_in("f.md", text), ["f.md:2: colour"]);
}

#[test]
fn the_book_and_its_crates_are_written_in_american_english() {
    let root = repo_root();
    let mut files = Vec::new();
    for r in ROOTS {
        collect(&root.join(r), &mut files);
    }
    assert!(
        files
            .iter()
            .any(|f| f.ends_with("books/ikigai/src/SUMMARY.md")),
        "the scan does not reach the book's pages — it would pass vacuously"
    );
    let this = Path::new(file!())
        .file_name()
        .expect("this test has a file name")
        .to_owned();
    let mut hits = Vec::new();
    for path in &files {
        if path.ends_with(Path::new("tests").join(&this)) {
            continue;
        }
        let text =
            fs::read_to_string(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
        let rel = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .display()
            .to_string();
        hits.extend(hits_in(&rel, &text));
    }
    assert!(
        hits.is_empty(),
        "{} British spelling(s) — write American (behavior, color, center, labeled, \
         tic-tac-toe), or mark a direct quote with `{QUOTE_MARKER}` on its line:\n  {}",
        hits.len(),
        hits.join("\n  ")
    );
}
