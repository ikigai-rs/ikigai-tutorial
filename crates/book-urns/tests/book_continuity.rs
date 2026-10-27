//! Serial-reader continuity, the part of it a parser can decide (ledger #1623).
//!
//! A reader who goes front to back arrives at each chapter carrying what the chapters before it
//! left behind: a server still running, a directory, a name. A chapter that ignores that state
//! still passes every other check here, because each check runs a page on its own, in a scratch
//! directory of its own. So the policy is written down in each book's `DESIGN.md` and the README,
//! and two parts of it are checked here, in every book under `books/`, in `SUMMARY.md` order:
//!
//! * **A long-running piece names the one before it.** A page whose transcripts start something
//!   that keeps running (a `<!-- transcript: serve -->` block: a gonk, a model stub) must, before
//!   its first such block, LINK the nearest earlier page that started one, which is what a serial
//!   reader still has running ("if the gonk from [that chapter] is still running, stop it first").
//!   Reorder the chapters and the link names the wrong one, which fails here.
//! * **"The next chapter" is the next chapter.** A link whose text says the next, previous or
//!   last chapter (or page, or part), or that phrase followed by `, [Title](target)`, must point
//!   at the neighboring page in `SUMMARY.md`. An unlinked "the next chapter" is not checked,
//!   because there is nothing to check it against; the fix there is to link it or name it.
//!
//! The ikigai Book's servers (`ikigai serve`, the tic-tac-toe host) start in `bash` and `sh`
//! blocks that no runner executes; each carries `<!-- transcript: serve -->` so this check sees it.
//!
//! What it does not see: a long-running piece in a block that carries no such declaration, a term
//! used before it is defined, a cleanup that breaks a later chapter. Those stay with the review the
//! policy asks for.

use std::path::{Path, PathBuf};

use book_transcripts::{declared, scan_page, Kind};
use book_urns::summary_chapters;

fn books() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../books");
    let mut out: Vec<PathBuf> = std::fs::read_dir(&root)
        .expect("books/")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            // The library (`books/library`) is one page that lists the books, not a book read in
            // order: it has no chapters for a reader to arrive at in sequence.
            p.join("src/SUMMARY.md").is_file() && !name.starts_with('.') && name != "library"
        })
        .collect();
    out.sort();
    out
}

/// `target` as written in `file`, relative to the book's `src`, without an anchor; `None` for a
/// link that leaves the book (a URL, or a path that climbs out of `src`).
fn resolve(file: &str, target: &str) -> Option<String> {
    if target.contains("://") || target.starts_with('#') {
        return None;
    }
    let target = target.split('#').next().unwrap_or("");
    let mut parts: Vec<&str> = file.split('/').collect();
    parts.pop();
    for segment in target.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            other => parts.push(other),
        }
    }
    Some(parts.join("/"))
}

/// Every `[text](target)` in `prose`, as (text, target, the text that follows the link).
fn links(prose: &str) -> Vec<(String, String, usize)> {
    let mut out = Vec::new();
    let bytes = prose.as_bytes();
    let mut i = 0;
    while let Some(open) = prose[i..].find('[') {
        let start = i + open;
        let Some(mid) = prose[start..].find("](") else {
            break;
        };
        let mid = start + mid;
        let Some(close) = prose[mid + 2..].find(')') else {
            break;
        };
        let close = mid + 2 + close;
        // A link's text holds no `[` of its own in these books; restart inside one that does.
        if let Some(inner) = prose[start + 1..mid].rfind('[') {
            i = start + 1 + inner;
            continue;
        }
        out.push((
            prose[start + 1..mid].replace('\n', " "),
            prose[mid + 2..close].trim().to_string(),
            close + 1,
        ));
        i = close + 1;
        let _ = bytes;
    }
    out
}

/// The page's prose: every line outside a fence, with fenced lines kept as empty lines so that
/// line numbers still count.
fn prose(text: &str) -> String {
    let mut fence = false;
    text.lines()
        .map(|l| {
            if l.trim_start().starts_with("```") {
                fence = !fence;
                return "";
            }
            if fence {
                ""
            } else {
                l
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Debug, PartialEq, Eq)]
enum Direction {
    Next,
    Previous,
}

/// The adjacency a link's own text, or the words just before it, claims.
fn claimed(text: &str, before: &str) -> Option<Direction> {
    let says = |s: &str| -> Option<Direction> {
        let s = s.to_lowercase().replace('\n', " ");
        for noun in ["chapter", "page", "part"] {
            if s.contains(&format!("next {noun}")) {
                return Some(Direction::Next);
            }
            if s.contains(&format!("previous {noun}")) || s.contains(&format!("last {noun}")) {
                return Some(Direction::Previous);
            }
        }
        None
    };
    if let Some(d) = says(text) {
        return Some(d);
    }
    // "is the next chapter, [Hub and spoke](…)": the phrase ends right before the link.
    let tail = before.trim_end();
    let tail = tail.strip_suffix(',').or_else(|| tail.strip_suffix(':'))?;
    let words: Vec<&str> = tail.split_whitespace().collect();
    let last_two = words[words.len().saturating_sub(2)..].join(" ");
    says(&last_two)
}

/// Each adjacency claim in one page that names a page other than its neighbor.
fn wrong_neighbors(file: &str, text: &str, order: &[String]) -> Vec<String> {
    let Some(at) = order.iter().position(|p| p == file) else {
        return Vec::new();
    };
    let prose = prose(text);
    let mut out = Vec::new();
    for (link_text, target, _) in links(&prose) {
        let pos = prose.find(&format!("[{link_text}]")).unwrap_or(0);
        let before = &prose[pos.saturating_sub(40)..pos];
        let Some(direction) = claimed(&link_text, before) else {
            continue;
        };
        let Some(target) = resolve(file, &target) else {
            continue;
        };
        let neighbor = match direction {
            Direction::Next => order.get(at + 1),
            Direction::Previous => at.checked_sub(1).and_then(|i| order.get(i)),
        };
        if neighbor != Some(&target) {
            out.push(format!(
                "{file}: [{link_text}]({target}) says {direction:?}, but that is {}",
                neighbor.map_or("nothing".to_string(), |n| n.clone())
            ));
        }
    }
    out
}

/// The 1-based line of the page's first `serve` block, if it has one.
///
/// A `console` block the transcript runner reads counts, and so does a fence of any other kind
/// (the ikigai Book's `bash` and `sh` blocks, which no runner executes) that carries the
/// `<!-- transcript: serve -->` declaration (or a manual one whose reason begins "a server"): the declaration is how that book says "this starts
/// something that stays up".
fn first_serve(file: &str, text: &str) -> Option<usize> {
    let scanned = scan_page(file, text)
        .into_iter()
        .find(|b| b.kind == Kind::Serve)
        .map(|b| b.line);
    let lines: Vec<&str> = text.lines().collect();
    let mut declared_at = None;
    let mut i = 0;
    while i < lines.len() {
        if !lines[i].trim_start().starts_with("```") {
            i += 1;
            continue;
        }
        let starts_server = match declared(&lines[i.saturating_sub(5)..i]) {
            Kind::Serve => true,
            // A `cargo run` block must be declared manual, and a block has one declaration:
            // a manual reason that says "a server" is how such a block says it stays up.
            Kind::Manual(reason) => reason.starts_with("a server"),
            _ => false,
        };
        if starts_server {
            declared_at = Some(i + 2);
            break;
        }
        // Skip to the closing fence.
        i += 1;
        while i < lines.len() && !lines[i].trim_start().starts_with("```") {
            i += 1;
        }
        i += 1;
    }
    match (scanned, declared_at) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    }
}

/// Whether the prose above `line` links `previous`.
fn links_before(file: &str, text: &str, line: usize, previous: &str) -> bool {
    let above: String = text
        .lines()
        .take(line.saturating_sub(1))
        .collect::<Vec<_>>()
        .join("\n");
    links(&prose(&above))
        .iter()
        .any(|(_, target, _)| resolve(file, target).as_deref() == Some(previous))
}

/// Each page that starts a long-running piece without naming the page whose piece a serial
/// reader still has running.
fn unacknowledged_servers(pages: &[(String, String)]) -> Vec<String> {
    let mut out = Vec::new();
    let mut last: Option<&str> = None;
    for (file, text) in pages {
        let Some(line) = first_serve(file, text) else {
            continue;
        };
        if let Some(previous) = last {
            if !links_before(file, text, line, previous) {
                out.push(format!(
                    "{file}:{line}: starts a long-running piece, and a reader arriving in order still \
                     has the one {previous} started; link it above this block (\"if the gonk from \
                     [that chapter](…) is still running, stop it first\")"
                ));
            }
        }
        last = Some(file);
    }
    out
}

/// How many list entries `SUMMARY.md` holds, linked or draft (`- [Title]()`): what a parser
/// that is reading the outline at all must find.
fn summary_entries(summary: &str) -> usize {
    summary
        .lines()
        .map(str::trim_start)
        .filter(|l| l.starts_with("- [") || l.starts_with("* ["))
        .count()
}

fn book_pages(book: &Path) -> (Vec<String>, Vec<(String, String)>) {
    let src = book.join("src");
    let summary = std::fs::read_to_string(src.join("SUMMARY.md")).expect("SUMMARY.md");
    let order: Vec<String> = summary_chapters(&summary)
        .into_iter()
        .map(|c| c.path)
        .collect();
    // The floor is on what the parser READ, drafts included: a book whose outline is written ahead
    // of its chapters (Running ikigai, as its arcs land) links few pages yet, and a parser that is
    // not reading the outline finds no entries at all.
    let entries = summary_entries(&summary);
    assert!(
        entries > 3 && !order.is_empty(),
        "{}: SUMMARY.md parsed to {} chapters ({entries} entries, drafts included)",
        book.display(),
        order.len()
    );
    let pages = order
        .iter()
        .map(|p| {
            let text = std::fs::read_to_string(src.join(p)).unwrap_or_default();
            (p.clone(), text)
        })
        .collect();
    (order, pages)
}

#[test]
fn a_chapter_that_starts_a_server_names_the_one_a_reader_still_has_running() {
    let mut faults = Vec::new();
    let mut serving = 0;
    for book in books() {
        let (_, pages) = book_pages(&book);
        serving += pages
            .iter()
            .filter(|(f, t)| first_serve(f, t).is_some())
            .count();
        faults.extend(unacknowledged_servers(&pages));
    }
    assert!(
        serving > 10,
        "only {serving} pages start a server — the scan is not reading the books"
    );
    assert!(faults.is_empty(), "\n{}", faults.join("\n"));
}

#[test]
fn a_link_that_says_next_or_previous_points_at_the_neighbor() {
    let mut faults = Vec::new();
    for book in books() {
        let (order, pages) = book_pages(&book);
        for (file, text) in &pages {
            faults.extend(wrong_neighbors(file, text, &order));
        }
    }
    assert!(faults.is_empty(), "\n{}", faults.join("\n"));
}

#[cfg(test)]
mod rules {
    use super::*;

    fn order() -> Vec<String> {
        ["a/one.md", "a/two.md", "b/three.md"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn a_next_link_to_the_page_after_the_next_is_refused() {
        let text = "Coordinating is the next chapter, [Three](../b/three.md).\n";
        assert_eq!(wrong_neighbors("a/one.md", text, &order()).len(), 1);
        assert!(wrong_neighbors("a/two.md", text, &order()).is_empty());
    }

    #[test]
    fn the_phrase_inside_the_link_text_is_read_too() {
        let ok = "The [previous chapter](one.md) did it.\n";
        assert!(wrong_neighbors("a/two.md", ok, &order()).is_empty());
        let wrong = "The [last chapter](one.md) did it.\n";
        assert_eq!(wrong_neighbors("b/three.md", wrong, &order()).len(), 1);
    }

    #[test]
    fn an_unlinked_phrase_and_a_fenced_one_are_not_claims() {
        let text = "That is the next chapter.\n```text\n[next chapter](x.md)\n```\n";
        assert!(wrong_neighbors("a/one.md", text, &order()).is_empty());
    }

    fn serve(prose: &str) -> String {
        format!("{prose}\n\n<!-- transcript: serve -->\n```console\n$ gonk\n```\n")
    }

    #[test]
    fn a_bash_block_declared_serve_counts_as_a_server() {
        let page =
            "Start it.\n\n<!-- transcript: serve -->\n```bash\nikigai serve /tmp/k.sock &\n```\n";
        assert_eq!(first_serve("a.md", page), Some(5));
        let undeclared = "Start it.\n\n```bash\nikigai serve /tmp/k.sock &\n```\n";
        assert_eq!(first_serve("a.md", undeclared), None);
    }

    #[test]
    fn a_second_server_must_link_the_page_that_started_the_first() {
        let pages = vec![
            ("a/one.md".to_string(), serve("Start one.")),
            ("a/two.md".to_string(), "No server here.\n".to_string()),
            ("b/three.md".to_string(), serve("Start another.")),
        ];
        assert_eq!(unacknowledged_servers(&pages).len(), 1);

        let pages = vec![
            ("a/one.md".to_string(), serve("Start one.")),
            (
                "b/three.md".to_string(),
                serve("If the gonk from [One](../a/one.md) is still running, stop it."),
            ),
        ];
        assert!(unacknowledged_servers(&pages).is_empty());
    }
}
