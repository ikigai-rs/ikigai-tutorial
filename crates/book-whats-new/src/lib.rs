//! The books' "What's new" pages.
//!
//! Every book under `books/` keeps a `changes.toml` beside its `book.toml`: one entry per change a
//! reader would notice, newest first, each a date, a short summary written for the reader, and the
//! pages it concerns (as `SUMMARY.md` names them). This crate turns that file into the book's
//! `src/whats-new.md` ([`render`]) and decides whether a change to a book's pages came with an
//! entry ([`prose_files`], [`new_or_updated`]). The git history the first entries were written from
//! is only their source: nothing on the page names a commit, and nothing here reads git except the
//! `check-diff` command in `main.rs`.
//!
//! An entry, in full:
//!
//! ```
//! let text = r#"
//! [[change]]
//! date = "2026-10-15"
//! summary = "A new page lists what changed in the book, most recent first."
//! pages = ["whats-new.md", "introduction.md"]
//! removed = []            # titles of pages that no longer exist, shown without a link
//! maintenance = false     # true: kept in the file, left off the page
//! commit = "68ab188"      # where it came from; for tooling, never shown
//! "#;
//! let changes = book_whats_new::parse_changes(text).unwrap();
//! assert_eq!(changes[0].date, "2026-10-15");
//! assert_eq!(changes[0].pages, ["whats-new.md", "introduction.md"]);
//! assert!(!changes[0].maintenance);
//! ```

use std::collections::BTreeSet;
use std::path::Path;

/// The entries file, beside a book's `book.toml`.
pub const ENTRIES_FILE: &str = "changes.toml";
/// The generated page, under a book's `src/`.
pub const PAGE: &str = "whats-new.md";

/// One entry of a book's `changes.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    /// `YYYY-MM-DD`.
    pub date: String,
    /// One or two sentences for a reader. Whitespace runs (a multi-line TOML string) render as one
    /// space.
    pub summary: String,
    /// Paths relative to the book's `src/`, each one `SUMMARY.md` lists. Linked, titled from it.
    pub pages: Vec<String>,
    /// Titles of pages that no longer exist. Listed without a link.
    pub removed: Vec<String>,
    /// Upkeep with no news in it (a version move, a link or typo fix): kept, but not shown.
    pub maintenance: bool,
    /// The commit an entry was written from. Tooling only; never rendered.
    pub commit: Option<String>,
}

const FIELDS: &[&str] = &[
    "date",
    "summary",
    "pages",
    "removed",
    "maintenance",
    "commit",
];

/// Parse a `changes.toml`. Every mistake names the entry (by position and date) and the field.
pub fn parse_changes(text: &str) -> Result<Vec<Change>, String> {
    let table: toml::Table = toml::from_str(text).map_err(|e| format!("not TOML: {e}"))?;
    for key in table.keys() {
        if key != "change" {
            return Err(format!(
                "unknown top-level key `{key}`: every entry is a `[[change]]` table"
            ));
        }
    }
    let Some(list) = table.get("change") else {
        return Ok(Vec::new());
    };
    let list = list
        .as_array()
        .ok_or("`change` must be an array of tables, written `[[change]]`")?;
    let mut out = Vec::with_capacity(list.len());
    for (i, item) in list.iter().enumerate() {
        let n = i + 1;
        let t = item
            .as_table()
            .ok_or_else(|| format!("entry {n}: must be a table, written `[[change]]`"))?;
        let date = t.get("date").and_then(toml::Value::as_str);
        let at = format!("entry {n} ({})", date.unwrap_or("no date"));
        for key in t.keys() {
            if !FIELDS.contains(&key.as_str()) {
                return Err(format!(
                    "{at}: unknown field `{key}` (the fields are {})",
                    FIELDS.join(", ")
                ));
            }
        }
        let date = date
            .ok_or_else(|| format!("{at}: `date` is required, a string \"YYYY-MM-DD\""))?
            .to_string();
        let summary = t
            .get("summary")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| format!("{at}: `summary` is required, a string"))?
            .to_string();
        let strings = |key: &str| -> Result<Vec<String>, String> {
            match t.get(key) {
                None => Ok(Vec::new()),
                Some(v) => v
                    .as_array()
                    .and_then(|a| {
                        a.iter()
                            .map(|s| s.as_str().map(str::to_string))
                            .collect::<Option<Vec<_>>>()
                    })
                    .ok_or_else(|| format!("{at}: `{key}` must be a list of strings")),
            }
        };
        let pages = strings("pages")?;
        let removed = strings("removed")?;
        let maintenance = match t.get("maintenance") {
            None => false,
            Some(v) => v
                .as_bool()
                .ok_or_else(|| format!("{at}: `maintenance` must be true or false"))?,
        };
        let commit = match t.get("commit") {
            None => None,
            Some(v) => Some(
                v.as_str()
                    .ok_or_else(|| format!("{at}: `commit` must be a string"))?
                    .to_string(),
            ),
        };
        out.push(Change {
            date,
            summary,
            pages,
            removed,
            maintenance,
            commit,
        });
    }
    Ok(out)
}

/// A book's `SUMMARY.md`, as the pages it links: `(path, title)` in order.
#[derive(Debug, Clone, Default)]
pub struct Summary {
    pub pages: Vec<(String, String)>,
}

impl Summary {
    /// Every line of the form `[Title](path)` or `- [Title](path)` (the forms mdbook reads). A
    /// draft (`[Title]()`) has no page and is skipped.
    pub fn parse(text: &str) -> Self {
        let mut pages = Vec::new();
        for line in text.lines() {
            let line = line.trim_start();
            let line = line
                .strip_prefix("- ")
                .or_else(|| line.strip_prefix("* "))
                .unwrap_or(line);
            let Some(rest) = line.strip_prefix('[') else {
                continue;
            };
            let Some(close) = rest.find("](") else {
                continue;
            };
            let title = &rest[..close];
            let after = &rest[close + 2..];
            let Some(end) = after.find(')') else {
                continue;
            };
            let path = after[..end].trim();
            let path = path.strip_prefix("./").unwrap_or(path);
            if !path.is_empty() {
                pages.push((path.to_string(), title.trim().to_string()));
            }
        }
        Summary { pages }
    }

    pub fn title(&self, path: &str) -> Option<&str> {
        self.pages
            .iter()
            .find(|(p, _)| p == path)
            .map(|(_, t)| t.as_str())
    }

    pub fn paths(&self) -> BTreeSet<String> {
        self.pages.iter().map(|(p, _)| p.clone()).collect()
    }
}

/// `YYYY-MM-DD`, with a month and a day that exist.
pub fn is_iso_date(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let num = |r: std::ops::Range<usize>| -> Option<u32> {
        let part = &s[r];
        part.bytes()
            .all(|c| c.is_ascii_digit())
            .then(|| part.parse().ok())
            .flatten()
    };
    let (Some(y), Some(m), Some(d)) = (num(0..4), num(5..7), num(8..10)) else {
        return false;
    };
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let days = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days).contains(&d)
}

/// Everything wrong with a book's entries against its `SUMMARY.md`, one message each.
pub fn validate(changes: &[Change], summary: &Summary) -> Vec<String> {
    let mut errors = Vec::new();
    if summary.title(PAGE).is_none() {
        errors.push(format!(
            "SUMMARY.md does not list `{PAGE}`: add `[What's new]({PAGE})` near the top"
        ));
    }
    for (i, c) in changes.iter().enumerate() {
        let at = format!("entry {} ({})", i + 1, c.date);
        if !is_iso_date(&c.date) {
            errors.push(format!(
                "{at}: `date` must be a real date written YYYY-MM-DD"
            ));
        }
        if i > 0 && c.date > changes[i - 1].date {
            errors.push(format!(
                "{at}: newer than the entry above it ({}); entries are newest first",
                changes[i - 1].date
            ));
        }
        if c.summary.trim().is_empty() {
            errors.push(format!("{at}: `summary` is empty"));
        }
        let mut seen = BTreeSet::new();
        for p in &c.pages {
            if !seen.insert(p) {
                errors.push(format!("{at}: `{p}` is listed twice"));
            }
            if p == PAGE {
                continue;
            }
            if summary.title(p).is_none() {
                errors.push(format!(
                    "{at}: `{p}` is not a page SUMMARY.md lists. If it moved, write its new path; \
                     if it is gone, move its title to `removed`"
                ));
            }
        }
        if !c.maintenance && c.pages.is_empty() && c.removed.is_empty() {
            errors.push(format!(
                "{at}: an entry the page shows names at least one page (`pages`) or a removed \
                 page (`removed`); upkeep with nothing to point at is `maintenance = true`"
            ));
        }
    }
    errors
}

fn one_line(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The page: an introduction, then the shown entries newest first, grouped by month (`## YYYY-MM`).
/// Entries marked `maintenance` are left off. Call [`validate`] first: a page that is not in
/// `SUMMARY.md` is rendered by its path, unlinked, rather than refused here.
///
/// ```
/// use book_whats_new::{parse_changes, render, Summary};
/// let summary = Summary::parse("[Introduction](introduction.md)\n[What's new](whats-new.md)\n- [Hello](hello.md)\n");
/// let changes = parse_changes(r#"
/// [[change]]
/// date = "2026-10-02"
/// summary = "A new chapter."
/// pages = ["hello.md"]
///
/// [[change]]
/// date = "2026-10-01"
/// summary = "A version moved."
/// maintenance = true
///
/// [[change]]
/// date = "2026-09-30"
/// summary = "An old chapter went away."
/// removed = ["Goodbye"]
/// "#).unwrap();
/// let page = render("books/demo", "The Demo Book", &changes, &summary);
/// assert!(page.starts_with("# What's new\n"));
/// assert!(page.contains("## 2026-10\n\n- **2026-10-02**: A new chapter.\n\n  Pages: [Hello](hello.md)\n"));
/// assert!(page.contains("## 2026-09\n\n- **2026-09-30**: An old chapter went away.\n\n  No longer in the book: Goodbye\n"));
/// assert!(!page.contains("A version moved"));
/// ```
pub fn render(book_dir: &str, book_title: &str, changes: &[Change], summary: &Summary) -> String {
    let mut out = String::new();
    out.push_str("# What's new\n\n");
    out.push_str(&format!(
        "<!-- Generated by crates/book-whats-new from {book_dir}/{ENTRIES_FILE}: edit that file, \
         not this page.\n     ./scripts/test-books.sh writes it, and a test fails when the two \
         disagree. -->\n\n"
    ));
    out.push_str(&format!(
        "What has changed in {book_title}, most recent first: the date, what was added or \
         changed, and the pages to read for it. Smaller upkeep (moving to a newer release when \
         nothing a reader does changes, a corrected link, a typo) is not listed.\n"
    ));
    let shown: Vec<&Change> = changes.iter().filter(|c| !c.maintenance).collect();
    if shown.is_empty() {
        out.push_str("\nNothing is listed yet.\n");
        return out;
    }
    let mut month = "";
    for c in shown {
        let m = c.date.get(..7).unwrap_or(&c.date);
        if m != month {
            out.push_str(&format!("\n## {m}\n"));
            month = m;
        }
        out.push_str(&format!("\n- **{}**: {}\n", c.date, one_line(&c.summary)));
        let links: Vec<String> = c
            .pages
            .iter()
            .map(|p| match summary.title(p) {
                Some(t) => format!("[{t}]({p})"),
                None => format!("`{p}`"),
            })
            .collect();
        if !links.is_empty() {
            out.push_str(&format!("\n  Pages: {}\n", links.join(", ")));
        }
        if !c.removed.is_empty() {
            out.push_str(&format!(
                "\n  No longer in the book: {}\n",
                c.removed.join(", ")
            ));
        }
    }
    out
}

/// A `book.toml`'s `[book] title`.
pub fn book_title(book_toml: &str) -> Option<String> {
    let table: toml::Table = toml::from_str(book_toml).ok()?;
    table
        .get("book")?
        .get("title")?
        .as_str()
        .map(str::to_string)
}

/// Of the files a change touched under a book's `src/` (paths relative to it), the ones that are
/// prose a reader sees: `SUMMARY.md` itself, and any `.md` page `published` lists, except the
/// generated page. A draft that no `SUMMARY.md` lists is not prose a reader sees, until the change
/// that lists it, which touches `SUMMARY.md`.
///
/// ```
/// use std::collections::BTreeSet;
/// let published: BTreeSet<String> = ["intro.md".to_string()].into();
/// let changed = ["intro.md", "drafts/idea.md", "whats-new.md", "vendor/htmx.js", "SUMMARY.md"]
///     .map(String::from);
/// assert_eq!(book_whats_new::prose_files(&changed, &published), ["intro.md", "SUMMARY.md"]);
/// ```
pub fn prose_files<'a>(changed: &'a [String], published: &BTreeSet<String>) -> Vec<&'a str> {
    changed
        .iter()
        .map(String::as_str)
        .filter(|f| f.ends_with(".md") && *f != PAGE)
        .filter(|f| *f == "SUMMARY.md" || published.contains(*f))
        .collect()
}

/// The entries in `head` that `base` does not hold as they are: added, or changed in any field.
pub fn new_or_updated<'a>(base: &[Change], head: &'a [Change]) -> Vec<&'a Change> {
    head.iter().filter(|c| !base.contains(c)).collect()
}

/// What the check prints when a change to a book's pages carries no entry.
pub fn missing_entry_message(book_dir: &str, prose: &[&str], summary: &Summary) -> String {
    let files: Vec<String> = prose
        .iter()
        .map(|f| format!("    {book_dir}/src/{f}"))
        .collect();
    let pages: Vec<String> = prose
        .iter()
        .filter(|f| summary.title(f).is_some() && **f != PAGE)
        .map(|f| format!("\"{f}\""))
        .collect();
    format!(
        "{book_dir}: this change edits the book's pages but adds or updates no entry in \
         {book_dir}/{ENTRIES_FILE}:\n{}\n\n\
         Add an entry at the TOP of {book_dir}/{ENTRIES_FILE} (entries are newest first):\n\n\
         \x20   [[change]]\n\
         \x20   date = \"YYYY-MM-DD\"\n\
         \x20   summary = \"One or two sentences for a reader: what they will now find.\"\n\
         \x20   pages = [{}]\n\n\
         If nothing changed that a reader would notice as news (a version move, a link or typo \
         fix), add it with `maintenance = true`: it is kept in the file and left off the page.\n\
         Then run ./scripts/test-books.sh (or `cargo run -p book-whats-new -- generate`) to \
         rewrite {book_dir}/src/{PAGE}, and commit both.",
        files.join("\n"),
        pages.join(", ")
    )
}

/// Every book under `root/books/`: the directories holding a `book.toml`, sorted.
pub fn books(root: &Path) -> Result<Vec<String>, String> {
    let dir = root.join("books");
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        // A dot-directory is a working copy, not a book (`scripts/serve-with-drafts.sh` builds
        // the drafts in `books/.drafts.*`, which is gitignored).
        let hidden = entry.file_name().to_string_lossy().starts_with('.');
        if !hidden && entry.path().join("book.toml").is_file() {
            out.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    out.sort();
    Ok(out)
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// The page a book's files say it should have, or everything wrong with them.
pub fn expected_page(root: &Path, book: &str) -> Result<String, String> {
    let dir = root.join("books").join(book);
    let book_dir = format!("books/{book}");
    let entries = dir.join(ENTRIES_FILE);
    if !entries.is_file() {
        return Err(format!(
            "{book_dir} has no {ENTRIES_FILE}. Every book keeps one: start it with an entry for \
             the book's first pages (see crates/book-whats-new/src/lib.rs for the fields)."
        ));
    }
    let changes =
        parse_changes(&read(&entries)?).map_err(|e| format!("{book_dir}/{ENTRIES_FILE}: {e}"))?;
    let summary = Summary::parse(&read(&dir.join("src/SUMMARY.md"))?);
    let errors = validate(&changes, &summary);
    if !errors.is_empty() {
        return Err(errors
            .iter()
            .map(|e| format!("{book_dir}/{ENTRIES_FILE}: {e}"))
            .collect::<Vec<_>>()
            .join("\n"));
    }
    let title = book_title(&read(&dir.join("book.toml"))?).unwrap_or_else(|| book.to_string());
    Ok(render(&book_dir, &title, &changes, &summary))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn change(date: &str, pages: &[&str]) -> Change {
        Change {
            date: date.into(),
            summary: "s".into(),
            pages: pages.iter().map(|s| s.to_string()).collect(),
            removed: vec![],
            maintenance: false,
            commit: None,
        }
    }

    fn summary() -> Summary {
        Summary::parse(
            "# Summary\n\n[Introduction](introduction.md)\n[What's new](whats-new.md)\n\n# Part\n\n\
             - [Hello](part/hello.md)\n  - [Nested](./part/nested.md)\n- [Draft]()\n---\n",
        )
    }

    #[test]
    fn summary_reads_every_linked_page_and_skips_drafts() {
        let s = summary();
        let paths: Vec<&str> = s.pages.iter().map(|(p, _)| p.as_str()).collect();
        assert_eq!(
            paths,
            [
                "introduction.md",
                "whats-new.md",
                "part/hello.md",
                "part/nested.md"
            ]
        );
        assert_eq!(s.title("part/nested.md"), Some("Nested"));
    }

    #[test]
    fn dates_must_exist() {
        assert!(is_iso_date("2024-02-29"));
        assert!(!is_iso_date("2026-02-29"));
        assert!(!is_iso_date("2026-13-01"));
        assert!(!is_iso_date("2026-1-01"));
        assert!(!is_iso_date("2026-10-15T00"));
    }

    #[test]
    fn a_valid_book_has_no_errors() {
        let changes = vec![
            change("2026-10-02", &["part/hello.md"]),
            change("2026-10-01", &["introduction.md", "whats-new.md"]),
        ];
        assert_eq!(validate(&changes, &summary()), Vec::<String>::new());
    }

    #[test]
    fn entries_out_of_order_are_refused() {
        let changes = vec![
            change("2026-10-01", &["part/hello.md"]),
            change("2026-10-02", &["part/hello.md"]),
        ];
        let errors = validate(&changes, &summary());
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].contains("newest first"));
    }

    #[test]
    fn a_page_summary_does_not_list_is_refused_with_the_way_out() {
        let errors = validate(&[change("2026-10-01", &["part/moved.md"])], &summary());
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].contains("`part/moved.md`") && errors[0].contains("removed"));
    }

    #[test]
    fn a_shown_entry_points_somewhere() {
        let errors = validate(&[change("2026-10-01", &[])], &summary());
        assert_eq!(errors.len(), 1, "{errors:?}");
        let mut quiet = change("2026-10-01", &[]);
        quiet.maintenance = true;
        assert!(validate(&[quiet], &summary()).is_empty());
    }

    #[test]
    fn the_page_itself_must_be_in_summary() {
        let s = Summary::parse("[Introduction](introduction.md)\n");
        let errors = validate(&[], &s);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("[What's new](whats-new.md)"));
    }

    #[test]
    fn unknown_fields_and_wrong_types_are_named() {
        let e = parse_changes("[[change]]\ndate = \"2026-10-01\"\nsummary = \"x\"\npage = []\n")
            .unwrap_err();
        assert!(
            e.contains("entry 1 (2026-10-01)") && e.contains("`page`"),
            "{e}"
        );
        let e =
            parse_changes("[[change]]\ndate = \"2026-10-01\"\nsummary = \"x\"\npages = \"a.md\"\n")
                .unwrap_err();
        assert!(e.contains("`pages` must be a list"), "{e}");
        let e = parse_changes("[[change]]\nsummary = \"x\"\n").unwrap_err();
        assert!(e.contains("`date` is required"), "{e}");
        let e = parse_changes("[change]\ndate = \"2026-10-01\"\n").unwrap_err();
        assert!(e.contains("[[change]]"), "{e}");
    }

    #[test]
    fn a_multi_line_summary_renders_as_one_list_item() {
        let mut c = change("2026-10-01", &["part/hello.md"]);
        c.summary = "One\n  two\nthree.".into();
        let page = render("books/x", "X", &[c], &summary());
        assert!(
            page.contains("- **2026-10-01**: One two three.\n"),
            "{page}"
        );
    }

    #[test]
    fn an_entry_counts_when_added_or_changed_and_not_when_untouched() {
        let base = vec![change("2026-10-01", &["part/hello.md"])];
        assert!(new_or_updated(&base, &base).is_empty());
        let mut edited = base.clone();
        edited[0].summary = "better".into();
        assert_eq!(new_or_updated(&base, &edited).len(), 1);
        let mut added = base.clone();
        added.insert(0, change("2026-10-02", &[]));
        assert_eq!(new_or_updated(&base, &added).len(), 1);
        // Deleting an entry is not adding one.
        assert!(new_or_updated(&base, &[]).is_empty());
    }

    #[test]
    fn the_message_names_the_files_and_offers_their_pages() {
        let s = summary();
        let msg = missing_entry_message("books/x", &["part/hello.md", "SUMMARY.md"], &s);
        assert!(msg.contains("    books/x/src/part/hello.md\n"), "{msg}");
        assert!(msg.contains("pages = [\"part/hello.md\"]"), "{msg}");
        assert!(msg.contains("maintenance = true"), "{msg}");
    }
}
