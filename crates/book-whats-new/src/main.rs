//! `book-whats-new`: write every book's What's new page, or check a change carried an entry.
//!
//! ```text
//! book-whats-new generate [--check] [ROOT]        write books/*/src/whats-new.md (or compare)
//! book-whats-new check-diff --base REV [--head REV|WORKTREE] [ROOT]
//! ```
//!
//! ROOT is the repository (default `.`); every directory under `ROOT/books/` with a `book.toml` is
//! a book, so a new book is covered by being one. `check-diff` reads git (`git diff` and `git show`
//! between two revisions) and nothing else does: the page is made from `changes.toml` alone. `--head
//! WORKTREE` compares the working tree (untracked files included) to `--base`, for a gate run before
//! the commit. A file a published page pulls in with `{{#include}}` counts as part of that page.

use book_whats_new::{
    books, expected_page, missing_entry_message, new_or_updated, parse_changes, prose_files,
    Change, Summary, ENTRIES_FILE, PAGE,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("generate") => generate(&args[1..]),
        Some("check-diff") => check_diff(&args[1..]),
        _ => Err("usage: book-whats-new generate [--check] [ROOT]\n       \
             book-whats-new check-diff --base REV [--head REV|WORKTREE] [ROOT]"
            .to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn root_arg(rest: &[String]) -> PathBuf {
    rest.first()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn generate(args: &[String]) -> Result<(), String> {
    let check = args.iter().any(|a| a == "--check");
    let rest: Vec<String> = args.iter().filter(|a| *a != "--check").cloned().collect();
    let root = root_arg(&rest);
    let mut problems = Vec::new();
    for book in books(&root)? {
        let page_path = root.join("books").join(&book).join("src").join(PAGE);
        match expected_page(&root, &book) {
            Err(e) => problems.push(e),
            Ok(page) if check => {
                if std::fs::read_to_string(&page_path).ok().as_deref() != Some(page.as_str()) {
                    problems.push(format!(
                        "books/{book}/src/{PAGE} does not match books/{book}/{ENTRIES_FILE}: run \
                         `cargo run -p book-whats-new -- generate` and commit the result"
                    ));
                }
            }
            Ok(page) => {
                if std::fs::read_to_string(&page_path).ok().as_deref() != Some(page.as_str()) {
                    std::fs::write(&page_path, page)
                        .map_err(|e| format!("{}: {e}", page_path.display()))?;
                    println!("wrote books/{book}/src/{PAGE}");
                }
            }
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("\n"))
    }
}

fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|e| format!("git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The `--head` that means the working tree rather than a revision.
const WORKTREE: &str = "WORKTREE";

/// A file at a revision (or in the working tree), or `None` when it is not there.
fn show(root: &Path, rev: &str, path: &str) -> Result<Option<String>, String> {
    if rev == WORKTREE {
        return match std::fs::read_to_string(root.join(path)) {
            Ok(t) => Ok(Some(t)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("{path}: {e}")),
        };
    }
    let spec = format!("{rev}:{path}");
    let exists = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["cat-file", "-e", &spec])
        .output()
        .map_err(|e| format!("git: {e}"))?
        .status
        .success();
    if !exists {
        return Ok(None);
    }
    git(root, &["show", &spec]).map(Some)
}

fn check_diff(args: &[String]) -> Result<(), String> {
    let mut base = None;
    let mut head = "HEAD".to_string();
    let mut rest = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--base" => base = it.next().cloned(),
            "--head" => head = it.next().cloned().ok_or("--head needs a revision")?,
            _ => rest.push(a.clone()),
        }
    }
    let base = base.ok_or("check-diff needs --base REV")?;
    let root = root_arg(&rest);

    // Every changed file in the repository: an included file lives outside `books/`.
    let mut all_changed: BTreeSet<String> = if head == WORKTREE {
        let mut set: BTreeSet<String> =
            git(&root, &["diff", "--name-only", "--no-renames", &base])?
                .lines()
                .map(String::from)
                .collect();
        set.extend(
            git(&root, &["ls-files", "--others", "--exclude-standard"])?
                .lines()
                .map(String::from),
        );
        set
    } else {
        git(
            &root,
            &["diff", "--name-only", "--no-renames", &base, &head],
        )?
        .lines()
        .map(String::from)
        .collect()
    };
    all_changed.retain(|f| !f.is_empty());
    let changed = all_changed
        .iter()
        .filter(|f| f.starts_with("books/"))
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    // Per book, the files under its src/, relative to src/.
    let mut by_book: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for line in changed.lines() {
        let mut parts = line.splitn(4, '/');
        if let (Some("books"), Some(book), Some("src"), Some(rel)) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        {
            by_book
                .entry(book.to_string())
                .or_default()
                .push(rel.to_string());
        }
    }

    // A book whose only change is an included file has no entry in `by_book` yet.
    for book in books(&root)? {
        by_book.entry(book).or_default();
    }
    let mut problems = Vec::new();
    for (book, files) in &mut by_book {
        let book_dir = format!("books/{book}");
        let summary_at = |rev: &str| -> Result<Summary, String> {
            Ok(show(&root, rev, &format!("{book_dir}/src/SUMMARY.md"))?
                .map(|t| Summary::parse(&t))
                .unwrap_or_default())
        };
        let head_summary = summary_at(&head)?;
        let mut published: BTreeSet<String> = head_summary.paths();
        published.extend(summary_at(&base)?.paths());
        // A page that pulls in a changed file changed too.
        let mut via: Vec<String> = Vec::new();
        for page in &published {
            if !page.ends_with(".md") || files.contains(page) {
                continue;
            }
            let page_path = format!("{book_dir}/src/{page}");
            let mut includes = BTreeSet::new();
            for rev in [&head, &base] {
                if let Some(text) = show(&root, rev, &page_path)? {
                    includes.extend(included_files(&page_path, &text));
                }
            }
            if let Some(hit) = includes.iter().find(|i| all_changed.contains(*i)) {
                via.push(format!("{page_path} includes {hit}"));
                files.push(page.clone());
            }
        }
        let prose = prose_files(files, &published);
        let entries_path = format!("{book_dir}/{ENTRIES_FILE}");
        let head_entries: Vec<Change> = match show(&root, &head, &entries_path)? {
            None if prose.is_empty() => continue,
            None => {
                problems.push(format!(
                    "{book_dir}: its pages changed and it has no {ENTRIES_FILE}\n\n{}",
                    missing_entry_message(&book_dir, &prose, &head_summary)
                ));
                continue;
            }
            Some(text) => parse_changes(&text).map_err(|e| format!("{entries_path}: {e}"))?,
        };
        // A base that does not parse (or has no file) holds no entries to compare against.
        let base_entries: Vec<Change> = show(&root, &base, &entries_path)?
            .and_then(|t| parse_changes(&t).ok())
            .unwrap_or_default();
        let mut recorded: Vec<&Change> = base_entries.iter().collect();
        for (index, entry) in head_entries.iter().enumerate() {
            // A summary already in the record stays exempt, including when its page paths or
            // other metadata are updated. Only newly introduced summary text gets this gate.
            if let Some(position) = recorded
                .iter()
                .position(|old| old.date == entry.date && old.summary == entry.summary)
            {
                recorded.remove(position);
                continue;
            }
            let (words, sentences) = summary_counts(&entry.summary);
            if words > MAX_SUMMARY_WORDS || sentences > MAX_SUMMARY_SENTENCES {
                problems.push(format!(
                    "{entries_path}: entry {} ({}): summary has {words} words and {sentences} sentences; \
                     new summaries allow at most {MAX_SUMMARY_WORDS} words and {MAX_SUMMARY_SENTENCES} sentences",
                    index + 1, entry.date
                ));
            }
        }
        if prose.is_empty() {
            println!("{book_dir}: no page changed");
            continue;
        }
        let fresh = new_or_updated(&base_entries, &head_entries);
        if fresh.is_empty() {
            let mut message = missing_entry_message(&book_dir, &prose, &head_summary);
            if !via.is_empty() {
                message.push_str(&format!(
                    "\n\nPages changed through {{{{#include}}}}:\n    {}",
                    via.join("\n    ")
                ));
            }
            problems.push(message);
        } else {
            println!(
                "{book_dir}: {} page file(s) changed, {} entr{} added or updated",
                prose.len(),
                fresh.len(),
                if fresh.len() == 1 { "y" } else { "ies" }
            );
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("\n\n"))
    }
}

/// Limits for summary text introduced by a diff, never the historical record.
const MAX_SUMMARY_WORDS: usize = 60;
const MAX_SUMMARY_SENTENCES: usize = 2;

/// Words are whitespace-separated. A sentence ends at a whitespace-separated token ending in
/// `.`, `!`, or `?` (possibly followed by closing quotes, parentheses, or Markdown delimiters).
/// Punctuation runs count once; internal dots in versions, paths, and URLs do not end sentences.
/// A final unpunctuated fragment counts too. This is a deterministic prose gate, not a language
/// parser: abbreviations ending in a period also count as sentence endings.
fn summary_counts(summary: &str) -> (usize, usize) {
    let mut words = 0;
    let mut sentences = 0;
    let mut fragment = false;
    for word in summary.split_whitespace() {
        words += 1;
        fragment = true;
        let end = word.trim_end_matches(['\"', '\'', '”', '’', ')', ']', '*', '_', '`']);
        if end.ends_with(['.', '!', '?']) {
            sentences += 1;
            fragment = false;
        }
    }
    (words, sentences + usize::from(fragment))
}

/// The repository-relative files a page pulls in with `{{#include}}` (also `rustdoc_include` and
/// `playground`), without the `:anchor` or line range.
fn included_files(page_path: &str, text: &str) -> BTreeSet<String> {
    let dir: Vec<&str> =
        page_path.split('/').collect::<Vec<_>>()[..page_path.split('/').count() - 1].to_vec();
    let mut out = BTreeSet::new();
    let mut rest = text;
    while let Some(i) = rest.find("{{#") {
        rest = &rest[i + 3..];
        let Some(end) = rest.find("}}") else { break };
        let body = &rest[..end];
        rest = &rest[end..];
        let mut words = body.split_whitespace();
        if !matches!(
            words.next(),
            Some("include" | "rustdoc_include" | "playground")
        ) {
            continue;
        }
        let Some(arg) = words.next() else { continue };
        let file = arg.split(':').next().unwrap_or(arg);
        let mut parts = dir.clone();
        for c in file.split('/') {
            match c {
                "" | "." => {}
                ".." => {
                    parts.pop();
                }
                c => parts.push(c),
            }
        }
        out.insert(parts.join("/"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::summary_counts;

    #[test]
    fn summary_counting_handles_prose_punctuation_and_technical_names() {
        assert_eq!(summary_counts("One?! Two..."), (2, 2));
        assert_eq!(summary_counts("One. An unfinished second"), (4, 2));
        assert_eq!(
            summary_counts("Read `src/main.rs` with 0.2.6 at https://example.org."),
            (6, 1)
        );
        assert_eq!(summary_counts("“First!” (Second?)"), (2, 2));
        assert_eq!(summary_counts("One.\n\nTwo."), (2, 2));
    }
}
