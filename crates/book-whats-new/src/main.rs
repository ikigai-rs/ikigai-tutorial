//! `book-whats-new`: write every book's What's new page, or check a change carried an entry.
//!
//! ```text
//! book-whats-new generate [--check] [ROOT]        write books/*/src/whats-new.md (or compare)
//! book-whats-new check-diff --base REV [--head REV] [ROOT]
//! ```
//!
//! ROOT is the repository (default `.`); every directory under `ROOT/books/` with a `book.toml` is
//! a book, so a new book is covered by being one. `check-diff` reads git (`git diff` and `git show`
//! between two revisions) and nothing else does: the page is made from `changes.toml` alone.

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
             book-whats-new check-diff --base REV [--head REV] [ROOT]"
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

/// A file at a revision, or `None` when the revision does not hold it.
fn show(root: &Path, rev: &str, path: &str) -> Result<Option<String>, String> {
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

    let changed = git(
        &root,
        &[
            "diff",
            "--name-only",
            "--no-renames",
            &base,
            &head,
            "--",
            "books",
        ],
    )?;
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

    let mut problems = Vec::new();
    for (book, files) in &by_book {
        let book_dir = format!("books/{book}");
        let summary_at = |rev: &str| -> Result<Summary, String> {
            Ok(show(&root, rev, &format!("{book_dir}/src/SUMMARY.md"))?
                .map(|t| Summary::parse(&t))
                .unwrap_or_default())
        };
        let head_summary = summary_at(&head)?;
        let mut published: BTreeSet<String> = head_summary.paths();
        published.extend(summary_at(&base)?.paths());
        let prose = prose_files(files, &published);
        if prose.is_empty() {
            println!("{book_dir}: no page changed");
            continue;
        }
        let entries_path = format!("{book_dir}/{ENTRIES_FILE}");
        let head_entries: Vec<Change> = match show(&root, &head, &entries_path)? {
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
        let fresh = new_or_updated(&base_entries, &head_entries);
        if fresh.is_empty() {
            problems.push(missing_entry_message(&book_dir, &prose, &head_summary));
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
