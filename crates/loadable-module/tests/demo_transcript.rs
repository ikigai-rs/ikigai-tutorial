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
//! `cargo run` and no binary on PATH), and diff its stdout against every copy.
//!
//! **One copy, where mdbook can share it.** The book's two chapters `{{#include}}`
//! `module-demo.stdout`, the file beside this crate's `Cargo.toml`, so the book holds the output
//! once. The README is rendered by GitHub, which has no includes, so it keeps a literal copy, and
//! this test is what keeps that copy in step.
//!
//! **Every copy, found rather than listed.** The scan reads the README and every book's `src/`,
//! and treats any shell fence that runs this crate as a promise of an output fence right after
//! it. An include in that output fence is resolved (relative to the page, as mdbook does), so a
//! chapter that goes back to a literal copy is still checked. A copy the scan cannot check
//! fails rather than being skipped: a run with no output fence after it, or one sharing its
//! fence with other commands (whose outputs would then share one block, and which line belongs
//! to which run is not something a test can know). `EXPECTED_PAGES` is the guard on the scan
//! itself: if a page stops being found, by a renamed fence language or a moved file, the test
//! says so instead of passing on fewer copies.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The pages that show the demo's output, relative to the repository root. Adding a copy
/// anywhere is fine (the scan finds it); add it here too, so losing it later is noticed.
const EXPECTED_PAGES: &[&str] = &[
    "README.md",
    "books/ikigai/src/modules/end-to-end.md",
    "books/ikigai/src/modules/index.md",
];

/// The command a reader is told to type.
const COMMAND: &str = "cargo run -p loadable-module";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the repository root is two levels up")
}

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

/// One fenced block: its info string, the line its opening fence is on, and its body lines.
struct Fence {
    info: String,
    line: usize,
    body: Vec<String>,
    /// Whether anything but blank lines and HTML comments sits between the previous fence's
    /// close and this fence's open.
    prose_before: bool,
}

/// Every fenced block in a page, in order. A fence opens with three or more backticks and
/// closes with at least as many and nothing else, so a four-backtick block can quote a
/// three-backtick one without ending early.
fn fences(text: &str) -> Vec<Fence> {
    let mut out = Vec::new();
    let mut open: Option<(usize, Fence)> = None;
    let mut prose_since_close = false;
    let mut in_comment = false;
    for (i, raw) in text.lines().enumerate() {
        let trimmed = raw.trim_start();
        let ticks = trimmed.chars().take_while(|&c| c == '`').count();
        match open.take() {
            None if ticks >= 3 => {
                open = Some((
                    ticks,
                    Fence {
                        info: trimmed[ticks..].trim().to_string(),
                        line: i + 1,
                        body: Vec::new(),
                        prose_before: prose_since_close,
                    },
                ));
            }
            None => {
                // An HTML comment is a note to the editor, not something between the two
                // blocks a reader sees.
                let mut rest = trimmed;
                if in_comment || rest.starts_with("<!--") {
                    match rest.find("-->") {
                        Some(end) => {
                            in_comment = false;
                            rest = rest[end + 3..].trim();
                        }
                        None => {
                            in_comment = true;
                            rest = "";
                        }
                    }
                }
                if !rest.is_empty() {
                    prose_since_close = true;
                }
            }
            Some((n, fence)) if ticks >= n && trimmed[ticks..].trim().is_empty() => {
                out.push(fence);
                prose_since_close = false;
            }
            Some((n, mut fence)) => {
                fence.body.push(raw.to_string());
                open = Some((n, fence));
            }
        }
    }
    out
}

fn is_shell(info: &str) -> bool {
    matches!(
        info.split_whitespace().next(),
        Some("bash" | "sh" | "shell" | "zsh")
    )
}

fn is_output(info: &str) -> bool {
    info.is_empty() || info == "text"
}

/// The lines a reader sees in an output fence: each `{{#include PATH}}` line replaced by the
/// file it names, relative to the page, the way mdbook does it.
fn rendered(page: &Path, body: &[String]) -> Vec<String> {
    let mut lines = Vec::new();
    for line in body {
        let trimmed = line.trim();
        if let Some(path) = trimmed
            .strip_prefix("{{#include ")
            .and_then(|rest| rest.strip_suffix("}}"))
        {
            assert!(
                !path.contains(':'),
                "{}: `{trimmed}` includes an anchor or line range; this check resolves a whole \
                 file only",
                page.display()
            );
            let file = page
                .parent()
                .expect("a page has a directory")
                .join(path.trim());
            let text = std::fs::read_to_string(&file)
                .unwrap_or_else(|e| panic!("{}: {trimmed}: {e}", page.display()));
            lines.extend(text.lines().map(str::to_string));
        } else {
            lines.push(line.clone());
        }
    }
    lines
}

/// The README and every page under `books/*/src/`. Dot-entries are skipped: they are local
/// state, not pages (`serve-with-drafts.sh` leaves `books/.drafts.*` behind, an editor its
/// undo files), and git ignores them, so CI would never see what a local run found there.
fn markdown_pages(root: &Path) -> Vec<PathBuf> {
    fn visible(p: &Path) -> bool {
        p.file_name()
            .is_some_and(|n| !n.to_string_lossy().starts_with('.'))
    }
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("readable") {
            let p = entry.expect("entry").path();
            if !visible(&p) {
                continue;
            }
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|e| e == "md") {
                out.push(p);
            }
        }
    }
    let mut pages = vec![root.join("README.md")];
    for book in std::fs::read_dir(root.join("books")).expect("books/") {
        let book = book.expect("entry").path();
        let src = book.join("src");
        if visible(&book) && src.is_dir() {
            walk(&src, &mut pages);
        }
    }
    pages.sort();
    pages
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
    let root = repo_root();
    let actual: Vec<String> = demo_stdout().lines().map(str::to_string).collect();
    let mut found = Vec::new();
    let mut problems = Vec::new();

    for page in markdown_pages(&root) {
        let rel = page
            .strip_prefix(&root)
            .expect("under the root")
            .to_string_lossy()
            .into_owned();
        let text = std::fs::read_to_string(&page).expect("readable page");
        let fences = fences(&text);
        for (i, fence) in fences.iter().enumerate() {
            if !is_shell(&fence.info) {
                continue;
            }
            let commands: Vec<&str> = fence
                .body
                .iter()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .collect();
            let runs = commands.iter().filter(|c| {
                let words: Vec<&str> = c.split_whitespace().collect();
                words.starts_with(&["cargo", "run"])
                    && words.windows(2).any(|w| w == ["-p", "loadable-module"])
            });
            let runs: Vec<&&str> = runs.collect();
            if runs.is_empty() {
                continue;
            }
            let at = format!("{rel}:{}", fence.line);
            if commands.len() != 1 || *runs[0] != COMMAND {
                problems.push(format!(
                    "{at}: give `{COMMAND}` a shell fence of its own, exactly that line, so the \
                     output fence after it is its output alone (found {commands:?})"
                ));
                continue;
            }
            match fences.get(i + 1) {
                Some(next) if is_output(&next.info) && !next.prose_before => {
                    let shown = rendered(&page, &next.body);
                    if shown != actual {
                        problems.push(format!(
                            "{rel}:{}: the page shows\n{}\nbut the demo prints\n{}",
                            next.line,
                            indent(&shown),
                            indent(&actual)
                        ));
                    }
                    found.push(rel.clone());
                }
                _ => problems.push(format!(
                    "{at}: `{COMMAND}` is not followed by its output (a ```text or bare fence, \
                     with nothing but blank lines between them)"
                )),
            }
        }
    }

    for page in EXPECTED_PAGES {
        if !found.iter().any(|f| f == page) {
            problems.push(format!(
                "{page}: expected a checked copy of the demo's output here and the scan found \
                 none; if it moved, update EXPECTED_PAGES"
            ));
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n\n"));
    eprintln!(
        "{} cop{} of `{COMMAND}`'s output checked: {}",
        found.len(),
        if found.len() == 1 { "y" } else { "ies" },
        found.join(", ")
    );
}

fn indent(lines: &[String]) -> String {
    lines
        .iter()
        .map(|l| format!("    | {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The scanner's own rules, on text small enough to read.
#[test]
fn a_fence_closes_only_on_its_own_length() {
    let page = "intro\n\n````markdown\n```bash\ncargo run -p loadable-module\n```\n````\n\n```text\nx\n```\n";
    let f = fences(page);
    assert_eq!(f.len(), 2, "the inner ``` fence is body, not a fence");
    assert_eq!(f[0].info, "markdown");
    assert_eq!(f[0].body.len(), 3);
    assert!(f[0].prose_before);
    assert_eq!(f[1].info, "text");
    assert!(!f[1].prose_before, "only blank lines between the two");
}

#[test]
fn prose_between_a_run_and_a_block_breaks_the_pair_and_a_comment_does_not() {
    let f =
        fences("```bash\ncargo run -p loadable-module\n```\n\nSome prose.\n\n```text\nx\n```\n");
    assert!(f[1].prose_before);
    let f = fences("```bash\nx\n```\n\n<!-- one\n     two -->\n```text\nx\n```\n");
    assert!(
        !f[1].prose_before,
        "a comment, even over two lines, is not prose"
    );
    let f = fences("```bash\nx\n```\n<!-- a note --> then prose\n```text\nx\n```\n");
    assert!(
        f[1].prose_before,
        "prose after a closed comment is still prose"
    );
}
