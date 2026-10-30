//! `book-whats-new check-diff` against a scratch git repository: a change to a book's pages with no
//! entry is refused, the same change with one passes, a draft and the generated page need none, and
//! a `maintenance = true` entry is the escape hatch. Hermetic: CI's checkout is shallow, so nothing
//! here reads this repository's own history.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.name=book",
            "-c",
            "user.email=book@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .args(args)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .expect("git");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn write(dir: &Path, rel: &str, text: &str) {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

const SUMMARY: &str = "# Summary\n\n[Introduction](introduction.md)\n[What's new](whats-new.md)\n\n- [Hello](hello.md)\n";
const ONE_ENTRY: &str =
    "[[change]]\ndate = \"2026-10-01\"\nsummary = \"The book begins.\"\npages = [\"hello.md\"]\n";

/// A repository with one book, committed: the base every case starts from.
fn scratch() -> Scratch {
    let dir = std::env::temp_dir().join(format!(
        "book-whats-new-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    git(&dir, &["init", "-q"]);
    write(&dir, "books/demo/book.toml", "[book]\ntitle = \"Demo\"\n");
    write(&dir, "books/demo/src/SUMMARY.md", SUMMARY);
    write(&dir, "books/demo/src/introduction.md", "# Introduction\n");
    write(&dir, "books/demo/src/hello.md", "# Hello\n");
    write(&dir, "books/demo/changes.toml", ONE_ENTRY);
    commit(&dir);
    Scratch(dir)
}

fn commit(dir: &Path) {
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "x"]);
}

fn check(dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_book-whats-new"))
        .args(["check-diff", "--base", "HEAD~1"])
        .arg(dir)
        .output()
        .expect("book-whats-new")
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

#[test]
fn a_page_change_without_an_entry_is_refused_and_told_what_to_add() {
    let s = scratch();
    write(&s.0, "books/demo/src/hello.md", "# Hello\n\nMore.\n");
    commit(&s.0);
    let out = check(&s.0);
    assert!(!out.status.success(), "it passed: {}", stderr(&out));
    let e = stderr(&out);
    assert!(e.contains("books/demo/src/hello.md"), "{e}");
    assert!(e.contains("pages = [\"hello.md\"]"), "{e}");
    assert!(e.contains("maintenance = true"), "{e}");
}

#[test]
fn the_same_change_with_a_new_entry_passes() {
    let s = scratch();
    write(&s.0, "books/demo/src/hello.md", "# Hello\n\nMore.\n");
    write(
        &s.0,
        "books/demo/changes.toml",
        &format!("[[change]]\ndate = \"2026-10-02\"\nsummary = \"More.\"\npages = [\"hello.md\"]\n\n{ONE_ENTRY}"),
    );
    commit(&s.0);
    let out = check(&s.0);
    assert!(out.status.success(), "{}", stderr(&out));
}

#[test]
fn an_updated_entry_counts_as_one() {
    let s = scratch();
    write(&s.0, "books/demo/src/hello.md", "# Hello\n\nFixed.\n");
    write(
        &s.0,
        "books/demo/changes.toml",
        &ONE_ENTRY.replace("begins.", "begins, corrected."),
    );
    commit(&s.0);
    assert!(check(&s.0).status.success());
}

#[test]
fn a_maintenance_entry_is_the_escape_hatch() {
    let s = scratch();
    write(
        &s.0,
        "books/demo/src/hello.md",
        "# Hello\n\nA typo fixed.\n",
    );
    write(
        &s.0,
        "books/demo/changes.toml",
        &format!("[[change]]\ndate = \"2026-10-02\"\nsummary = \"A typo.\"\nmaintenance = true\n\n{ONE_ENTRY}"),
    );
    commit(&s.0);
    let out = check(&s.0);
    assert!(out.status.success(), "{}", stderr(&out));
}

#[test]
fn a_draft_and_the_generated_page_need_no_entry() {
    let s = scratch();
    write(&s.0, "books/demo/src/drafts/idea.md", "# An idea\n");
    write(&s.0, "books/demo/src/whats-new.md", "# What's new\n");
    commit(&s.0);
    let out = check(&s.0);
    assert!(out.status.success(), "{}", stderr(&out));
}

#[test]
fn listing_a_draft_in_summary_is_a_page_change() {
    let s = scratch();
    write(&s.0, "books/demo/src/drafts/idea.md", "# An idea\n");
    commit(&s.0);
    write(
        &s.0,
        "books/demo/src/SUMMARY.md",
        &format!("{SUMMARY}- [An idea](drafts/idea.md)\n"),
    );
    commit(&s.0);
    assert!(!check(&s.0).status.success());
}

#[test]
fn deleting_a_published_page_is_a_page_change() {
    let s = scratch();
    std::fs::remove_file(s.0.join("books/demo/src/hello.md")).unwrap();
    commit(&s.0);
    let out = check(&s.0);
    assert!(!out.status.success(), "{}", stderr(&out));
    assert!(stderr(&out).contains("books/demo/src/hello.md"));
}

#[test]
fn a_new_book_without_an_entries_file_is_refused() {
    let s = scratch();
    write(
        &s.0,
        "books/second/book.toml",
        "[book]\ntitle = \"Second\"\n",
    );
    write(
        &s.0,
        "books/second/src/SUMMARY.md",
        "[Introduction](introduction.md)\n",
    );
    write(&s.0, "books/second/src/introduction.md", "# Introduction\n");
    commit(&s.0);
    let out = check(&s.0);
    assert!(!out.status.success());
    assert!(
        stderr(&out).contains("books/second: its pages changed and it has no changes.toml"),
        "{}",
        stderr(&out)
    );
}

fn check_worktree(dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_book-whats-new"))
        .args(["check-diff", "--base", "HEAD", "--head", "WORKTREE"])
        .arg(dir)
        .output()
        .expect("book-whats-new")
}

fn with_include(s: &Scratch) {
    write(
        &s.0,
        "crates/demo/src/lib.rs",
        "// ANCHOR: a
fn a() {}
// ANCHOR_END: a
",
    );
    write(
        &s.0,
        "books/demo/src/hello.md",
        "# Hello\n\n```rust\n{{#include ../../../crates/demo/src/lib.rs:a}}\n```\n",
    );
    commit(&s.0);
}

#[test]
fn a_change_to_an_included_file_is_a_page_change() {
    let s = scratch();
    with_include(&s);
    write(
        &s.0,
        "crates/demo/src/lib.rs",
        "// ANCHOR: a\nfn b() {}\n// ANCHOR_END: a\n",
    );
    commit(&s.0);
    let out = check(&s.0);
    assert!(!out.status.success(), "it passed: {}", stderr(&out));
    let e = stderr(&out);
    assert!(e.contains("books/demo/src/hello.md"), "{e}");
    assert!(e.contains("crates/demo/src/lib.rs"), "{e}");
}

#[test]
fn a_change_to_an_unrelated_file_is_not() {
    let s = scratch();
    with_include(&s);
    write(&s.0, "crates/demo/src/other.rs", "fn c() {}\n");
    commit(&s.0);
    assert!(check(&s.0).status.success());
}

#[test]
fn worktree_mode_sees_uncommitted_changes() {
    let s = scratch();
    with_include(&s);
    assert!(check_worktree(&s.0).status.success());
    write(&s.0, "books/demo/src/hello.md", "# Hello\n\nMore.\n");
    let out = check_worktree(&s.0);
    assert!(!out.status.success(), "it passed: {}", stderr(&out));
    write(
        &s.0,
        "books/demo/changes.toml",
        &format!("[[change]]\ndate = \"2026-10-02\"\nsummary = \"More.\"\npages = [\"hello.md\"]\n\n{ONE_ENTRY}"),
    );
    let out = check_worktree(&s.0);
    assert!(out.status.success(), "{}", stderr(&out));
}

#[test]
fn worktree_mode_sees_an_uncommitted_included_file() {
    let s = scratch();
    with_include(&s);
    write(&s.0, "crates/demo/src/lib.rs", "fn z() {}\n");
    assert!(!check_worktree(&s.0).status.success());
}

#[test]
fn added_summaries_obey_word_and_sentence_limits_even_without_page_changes() {
    for (summary, expected) in [
        (format!("{}end.", "word ".repeat(60)), Some("61 words")),
        ("One. Two! Three?".to_string(), Some("3 sentences")),
        (format!("{}end. Second sentence!", "word ".repeat(57)), None),
    ] {
        let s = scratch();
        write(
            &s.0,
            "books/demo/changes.toml",
            &format!("[[change]]\ndate = \"2026-10-02\"\nsummary = \"{summary}\"\npages = [\"hello.md\"]\n\n{ONE_ENTRY}"),
        );
        commit(&s.0);
        let out = check(&s.0);
        if let Some(count) = expected {
            assert!(!out.status.success(), "accepted {count}");
            let e = stderr(&out);
            assert!(
                e.contains("books/demo/changes.toml: entry 1 (2026-10-02)"),
                "{e}"
            );
            assert!(e.contains(count), "{e}");
        } else {
            assert!(out.status.success(), "{}", stderr(&out));
        }
    }
}

#[test]
fn an_unchanged_old_long_summary_is_not_checked() {
    let s = scratch();
    let old = ONE_ENTRY.replace("The book begins.", &"Old sentence. ".repeat(40));
    write(&s.0, "books/demo/changes.toml", &old);
    commit(&s.0);
    write(&s.0, "books/demo/src/hello.md", "# Hello\n\nMore.\n");
    write(
        &s.0,
        "books/demo/changes.toml",
        &format!("[[change]]\ndate = \"2026-10-02\"\nsummary = \"More.\"\npages = [\"hello.md\"]\n\n{old}"),
    );
    commit(&s.0);
    let out = check(&s.0);
    assert!(out.status.success(), "{}", stderr(&out));
}

#[test]
fn an_old_long_summary_can_have_its_page_paths_updated() {
    let s = scratch();
    let old = ONE_ENTRY.replace("The book begins.", &"Old sentence. ".repeat(40));
    write(&s.0, "books/demo/changes.toml", &old);
    commit(&s.0);
    write(&s.0, "books/demo/src/hello.md", "# Hello\n\nCorrected.\n");
    write(
        &s.0,
        "books/demo/changes.toml",
        &old.replace("[\"hello.md\"]", "[\"hello.md\", \"introduction.md\"]"),
    );
    commit(&s.0);
    let out = check(&s.0);
    assert!(out.status.success(), "{}", stderr(&out));
}

#[test]
fn worktree_mode_refuses_a_long_new_entry_with_a_page_change() {
    let s = scratch();
    write(&s.0, "books/demo/src/hello.md", "# Hello\n\nMore.\n");
    write(
        &s.0,
        "books/demo/changes.toml",
        &format!("[[change]]\ndate = \"2026-10-02\"\nsummary = \"{}\"\npages = [\"hello.md\"]\n\n{ONE_ENTRY}", "word ".repeat(61)),
    );
    let out = check_worktree(&s.0);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("61 words"), "{}", stderr(&out));
}
