//! The book, and the code it teaches, are written in American English.
//!
//! Not a style preference that review can hold: the words drift in from briefs, from
//! quoted sources and from habit, and a relapse is invisible in a diff that is about
//! something else. So this fails on a short DENYLIST of British spellings in the book's
//! prose, its page scripts and styles, and every crate's sources and READMEs.
//!
//! The denylist is deliberately short and unambiguous: only words that have no American
//! meaning at all, so it never fires on a false positive. `grey` and `judgement` are
//! accepted American variants and are not on it; `labelledby` is a different token from
//! `labelled` (it is an HTML attribute, `aria-labelledby`) and never matches.
//!
//! ## The escape, for a quote
//!
//! A direct quote keeps its author's spelling. Say so where it stands, in the same
//! directive style as `urn-gate:`, naming the word and the reason:
//!
//! ```text
//! <!-- spelling: allow colour — a quoted source's own words. -->
//! // spelling: allow colour — the upstream error text, matched verbatim.
//! ```
//!
//! A directive covers its own line and the block that follows it, up to the next blank
//! line — in Markdown, the paragraph it sits directly above. Same teeth both ways: a
//! directive with no reason is refused, and so is one that allows nothing, because a
//! stale exemption is how a gate rots.

use std::path::{Path, PathBuf};

/// Words with no American meaning. Keep it that way: a word with a legitimate American
/// use belongs in review, not here.
const DENYLIST: &[&str] = &[
    "analyse",
    "analysed",
    "artefact",
    "artefacts",
    "behaviour",
    "behaviours",
    "cancelled",
    "cancelling",
    "centre",
    "centred",
    "centres",
    "colour",
    "coloured",
    "colours",
    "enrolment",
    "favour",
    "favourite",
    "honour",
    "honoured",
    "honouring",
    "honours",
    "labelled",
    "labelling",
    "licence",
    "modelled",
    "modelling",
    "neighbour",
    "neighbours",
    "noughts",
    "normalise",
    "normalised",
    "organise",
    "organised",
    "realise",
    "realised",
    "recognise",
    "recognised",
    "serialise",
    "serialised",
    "travelled",
    "travelling",
    "whilst",
];

const DIRECTIVE: &str = "spelling: allow";

/// Where we write. Relative to the repository root; each is walked for [`EXTENSIONS`].
const ROOTS: &[&str] = &[
    "README.md",
    "books/ikigai/src",
    "books/ikigai/css",
    "books/ikigai/js",
    "crates",
    "docs",
    "scripts",
    "a11y",
    ".github",
];

const EXTENSIONS: &[&str] = &[
    "md", "rs", "css", "js", "mjs", "html", "toml", "yml", "yaml", "sh", "txt",
];

/// Directories that are not ours to spell: build output, installed packages, and vendored
/// third-party files (`books/ikigai/src/vendor/`), which stay byte for byte upstream's.
const SKIP_DIRS: &[&str] = &["target", "node_modules", "vendor"];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the workspace root is two levels up from this crate")
}

fn collect(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_dir() {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if SKIP_DIRS.contains(&name) {
            return;
        }
        let mut entries: Vec<_> = std::fs::read_dir(path)
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

/// One `spelling: allow` directive, and whether any line it covers used it.
#[derive(Debug)]
struct Allow {
    line: usize,
    word: String,
    used: bool,
}

/// The denylisted words on one line, lowercased, as whole alphabetic tokens — so
/// `colour_pair` and `Colour` both match and `labelledby` does not.
fn denied_words(line: &str) -> Vec<String> {
    line.split(|c: char| !c.is_ascii_alphabetic())
        .map(str::to_ascii_lowercase)
        .filter(|w| DENYLIST.contains(&w.as_str()))
        .collect()
}

/// Every problem in one file: a denylisted word no directive covers, a directive with no
/// reason, a directive that allows nothing.
fn check(file: &str, text: &str) -> Vec<String> {
    let mut problems = Vec::new();
    // The directives in force: those since the last blank line.
    let mut active: Vec<Allow> = Vec::new();
    let retire = |active: &mut Vec<Allow>, problems: &mut Vec<String>| {
        for allow in active.drain(..) {
            if !allow.used {
                problems.push(format!(
                    "{file}:{}: `spelling: allow {}` covers no use of it — remove the \
                     directive",
                    allow.line, allow.word
                ));
            }
        }
    };
    for (index, raw) in text.lines().enumerate() {
        let line_no = index + 1;
        if raw.trim().is_empty() {
            retire(&mut active, &mut problems);
            continue;
        }
        // The directive's own words are a declaration, not prose: scan only what is
        // before it.
        let prose = match raw.find(DIRECTIVE) {
            Some(at) => {
                let rest = raw[at + DIRECTIVE.len()..].replace("-->", " ");
                let mut words = rest.split_whitespace();
                let word = words.next().unwrap_or("").to_ascii_lowercase();
                let reason = words.collect::<Vec<_>>().join(" ");
                let reason = reason.trim_start_matches(['—', '-', ':']).trim();
                if word.is_empty() || reason.is_empty() {
                    problems.push(format!(
                        "{file}:{line_no}: a `spelling: allow` directive needs a word and a \
                         reason (`spelling: allow colour — a quoted source's own words`)"
                    ));
                } else {
                    active.push(Allow {
                        line: line_no,
                        word,
                        used: false,
                    });
                }
                &raw[..at]
            }
            None => raw,
        };
        for word in denied_words(prose) {
            match active.iter_mut().find(|a| a.word == word) {
                Some(allow) => allow.used = true,
                None => problems.push(format!(
                    "{file}:{line_no}: British spelling `{word}` — the book is written in \
                     American English (a direct quote takes a `spelling: allow` directive)"
                )),
            }
        }
    }
    retire(&mut active, &mut problems);
    problems
}

#[test]
fn the_book_and_its_crates_are_written_in_american_english() {
    let root = repo_root();
    let this_file = Path::new(file!())
        .file_name()
        .expect("this test has a file name")
        .to_owned();
    let mut files = Vec::new();
    for r in ROOTS {
        collect(&root.join(r), &mut files);
    }
    assert!(
        files
            .iter()
            .any(|f| f.ends_with("books/ikigai/src/SUMMARY.md")),
        "the scan must reach the book's pages; the roots are wrong"
    );
    let mut problems = Vec::new();
    for path in &files {
        // This file names the denylist, so it is the one file that cannot be scanned.
        if path.file_name() == Some(this_file.as_os_str()) {
            continue;
        }
        let text = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
        let shown = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .display()
            .to_string();
        problems.extend(check(&shown, &text));
    }
    assert!(
        problems.is_empty(),
        "{} spelling problem(s):\n  {}",
        problems.len(),
        problems.join("\n  ")
    );
}

#[test]
fn a_british_spelling_is_refused_and_an_american_one_is_not() {
    assert_eq!(
        check("f.md", "the color of the center\n"),
        Vec::<String>::new()
    );
    let problems = check("f.md", "the colour of the Centre\n");
    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(problems[0].starts_with("f.md:1:"));
}

#[test]
fn a_token_is_matched_whole_so_an_attribute_name_is_not_a_word() {
    assert!(check("f.js", "el.setAttribute(\"aria-labelledby\", id);\n").is_empty());
    assert_eq!(check("f.rs", "let colour_pair = 1;\n").len(), 1);
}

#[test]
fn a_directive_covers_the_paragraph_after_it_and_no_further() {
    let text = "<!-- spelling: allow colour — quoted. -->\nfirst line\nthe colour, quoted\n\n\
                a later colour\n";
    let problems = check("f.md", text);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].starts_with("f.md:5:"), "{problems:?}");
}

#[test]
fn a_directive_on_the_same_line_covers_that_line() {
    assert!(check(
        "f.rs",
        "let s = \"colour\"; // spelling: allow colour — upstream text\n"
    )
    .is_empty());
}

#[test]
fn a_directive_without_a_reason_is_refused() {
    let problems = check("f.md", "<!-- spelling: allow colour -->\nthe colour\n");
    assert!(
        problems
            .iter()
            .any(|p| p.contains("needs a word and a reason")),
        "{problems:?}"
    );
}

#[test]
fn a_directive_that_allows_nothing_is_refused() {
    let problems = check(
        "f.md",
        "<!-- spelling: allow colour — quoted. -->\nthe color\n",
    );
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains("covers no use"), "{problems:?}");
}
