//! Every REPL transcript in the book is replayed against a real `ikigai`.
//!
//! A ```` ```text ```` block whose lines start with `ikigai> ` is a transcript: the book
//! saying "type this, see that". Nothing compiles one, so one was written to fit its
//! prose — the front door's "Cut a thread" showed `sink urn:kernel:cut urn:iki:fn:toUpper`
//! followed by `not cached`, and the real CLI says `cached`: a pure function declares no
//! golden thread, so a cut has nothing of its own to invalidate. The prose was wrong and
//! the transcript agreed with it, which is the failure mode this file exists for.
//!
//! The replay sends every `ikigai> ` line of a block to ONE `ikigai --plain` process, as
//! repeated `-c` arguments, so the cache carries across the lines the way it does in a
//! REPL session, and diffs the output against the block's non-prompt lines. Two kinds of
//! noise are normalized away on both sides — the `[computed]` / `[cached]` /
//! `[N computed · M cached]` verdict lines, and durations like `0ms` — because a verdict
//! depends on what the reader ran before, and the prose beside a block, not the block,
//! is where a verdict is a claim. A line that is exactly `…` in a block means "any lines
//! here"; the segments around it must appear in order.
//!
//! Two declarations, in an HTML comment on the lines above a fence. A block that is the
//! NEXT part of the same session — the front door's sections 5 to 8 are one REPL sitting,
//! split for the prose — says so, and is replayed with everything before it prepended:
//!
//! ```text
//! <!-- transcript: continues -->
//! ```
//!
//! That is not a convenience. Replayed alone, "cut the thread, then probe" says
//! `not cached` because a fresh process has nothing cached — which is exactly what the
//! wrong block claimed, so isolation would have passed the bug. A block that cannot be
//! replayed at all declares itself with a reason:
//!
//! ```text
//! <!-- transcript: manual — the capability line prints this machine's home directory -->
//! ```
//!
//! Like the CLI vocabulary probe, this needs a binary and is `#[ignore]`d rather than
//! silently skipped:
//!
//!     cargo install ikigai-cli --locked
//!     cargo test -p book-urns --test book_transcripts -- --ignored --nocapture

use std::path::{Path, PathBuf};
use std::process::Command;

fn book_src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../books/ikigai/src")
        .canonicalize()
        .expect("the book source is two levels up")
}

/// One transcript block.
#[derive(Debug)]
struct Transcript {
    file: String,
    line: usize,
    commands: Vec<String>,
    /// The line of every block merged into this transcript (`continues`).
    spans: Vec<usize>,
    /// The block's non-prompt lines, in order — what the reader is told to expect.
    expected: Vec<String>,
    /// `Some(reason)` when the prose declared the block manual.
    manual: Option<String>,
}

fn collect_markdown(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("readable") {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            collect_markdown(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(path);
        }
    }
}

/// Every transcript in the book.
fn transcripts() -> Vec<Transcript> {
    let root = book_src();
    let mut files = Vec::new();
    collect_markdown(&root, &mut files);
    files.sort();

    let mut out: Vec<Transcript> = Vec::new();
    for path in files {
        let file = path
            .strip_prefix(&root)
            .expect("under root")
            .to_string_lossy()
            .into_owned();
        let text = std::fs::read_to_string(&path).expect("readable");
        let lines: Vec<&str> = text.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            if lines[i].trim_start() != "```text" {
                i += 1;
                continue;
            }
            let start = i + 1;
            let mut end = start;
            while end < lines.len() && !lines[end].trim_start().starts_with("```") {
                end += 1;
            }
            let block: Vec<&str> = lines[start..end].to_vec();
            if !block.iter().any(|l| l.starts_with("ikigai> ")) {
                i = end + 1;
                continue;
            }
            // A declaration is an HTML comment within the five lines above the fence —
            // close enough to be read as belonging to the block.
            let above = &lines[i.saturating_sub(5)..i];
            let manual = above.iter().find_map(|l| {
                l.split_once("transcript: manual").map(|(_, reason)| {
                    reason
                        .trim()
                        .trim_start_matches(['—', '-', ':'])
                        .trim()
                        .trim_end_matches("-->")
                        .trim()
                        .to_string()
                })
            });
            let continues = above.iter().any(|l| l.contains("transcript: continues"));
            let commands: Vec<String> = block
                .iter()
                .filter_map(|l| l.strip_prefix("ikigai> "))
                .map(str::to_string)
                .collect();
            let expected: Vec<String> = block
                .iter()
                .filter(|l| !l.starts_with("ikigai> "))
                .map(|l| l.to_string())
                .collect();
            match (continues, out.last_mut()) {
                // The same sitting: the earlier block's commands come first, and its
                // output is expected first. The merged transcript keeps the first
                // block's line, and the report names every block it spans.
                (true, Some(previous)) if previous.file == file => {
                    previous.commands.extend(commands);
                    previous.expected.extend(expected);
                    previous.spans.push(start);
                }
                _ => out.push(Transcript {
                    file: file.clone(),
                    line: start,
                    spans: vec![start],
                    commands,
                    expected,
                    manual,
                }),
            }
            i = end + 1;
        }
    }
    out
}

/// Drop the lines that legitimately differ between the book and a replay: cache verdicts
/// and the batch summary; blank lines; and rewrite durations.
fn normalize(lines: impl IntoIterator<Item = String>) -> Vec<String> {
    lines
        .into_iter()
        .map(|l| l.trim_end().to_string())
        .filter(|l| !l.is_empty())
        .filter(|l| !(l.starts_with('[') && l.ends_with(']')))
        .filter(|l| !l.starts_with("— batch:"))
        // A trace node carries its verdict in-line: `· computed ·` / `· cached ·`.
        .map(|l| {
            l.replace("· computed ·", "· <verdict> ·")
                .replace("· cached ·", "· <verdict> ·")
        })
        .map(|l| {
            // `0ms` → `<ms>`, wherever it appears.
            let mut out = String::new();
            let mut rest = l.as_str();
            while let Some(at) = rest.find("ms") {
                let (head, tail) = rest.split_at(at);
                let digits = head
                    .chars()
                    .rev()
                    .take_while(|c| c.is_ascii_digit())
                    .count();
                if digits > 0 && head[..head.len() - digits].ends_with(' ') {
                    out.push_str(&head[..head.len() - digits]);
                    out.push_str("<ms>");
                } else {
                    out.push_str(head);
                    out.push_str("ms");
                }
                rest = &tail[2..];
            }
            out.push_str(rest);
            out
        })
        .collect()
}

/// Does `actual` match `expected`, where an expected line of `…` matches any run of lines?
fn matches(expected: &[String], actual: &[String]) -> bool {
    let segments: Vec<Vec<&String>> = expected
        .split(|l| l.trim() == "…")
        .map(|s| s.iter().collect())
        .collect();
    let mut pos = 0usize;
    for (n, segment) in segments.iter().enumerate() {
        if segment.is_empty() {
            continue;
        }
        let first = n == 0 && !expected.first().is_some_and(|l| l.trim() == "…");
        // Find the segment as a contiguous run in `actual` at or after `pos`.
        let found = (pos..=actual.len().saturating_sub(segment.len())).find(|&at| {
            actual.len() >= at + segment.len()
                && segment
                    .iter()
                    .zip(&actual[at..at + segment.len()])
                    .all(|(e, a)| e.trim() == a.trim())
        });
        match found {
            Some(at) if !first || at == pos => pos = at + segment.len(),
            _ => return false,
        }
    }
    // With no trailing `…`, nothing may follow.
    if !expected.last().is_some_and(|l| l.trim() == "…") && pos != actual.len() {
        return false;
    }
    true
}

#[test]
#[ignore = "needs an `ikigai` binary on PATH; replays every REPL transcript in the book against it"]
fn every_transcript_in_the_book_replays_against_the_installed_cli() {
    let found = transcripts();
    assert!(
        found.len() >= 3,
        "only {} transcripts found — the scanner is not reading the book",
        found.len()
    );

    let mut failures = Vec::new();
    for t in &found {
        if let Some(reason) = &t.manual {
            println!("skip {}:{} — manual: {reason}", t.file, t.line);
            continue;
        }
        let mut cmd = Command::new("ikigai");
        cmd.arg("--plain");
        for c in &t.commands {
            cmd.arg("-c").arg(c);
        }
        let output = match cmd.output() {
            Ok(o) => o,
            Err(e) => panic!(
                "no `ikigai` on PATH ({e}); install it with `cargo install ikigai-cli --locked`"
            ),
        };
        let mut actual: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::to_string)
            .collect();
        actual.extend(
            String::from_utf8_lossy(&output.stderr)
                .lines()
                .map(str::to_string),
        );
        let actual = normalize(actual);
        let expected = normalize(t.expected.iter().cloned());
        let ok = matches(&expected, &actual);
        let where_ = t
            .spans
            .iter()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .join("+");
        println!(
            "{} {}:{} ({} command{})",
            if ok { "ok  " } else { "MISS" },
            t.file,
            where_,
            t.commands.len(),
            if t.commands.len() == 1 { "" } else { "s" }
        );
        if !ok {
            failures.push(format!(
                "{}:{}\n    commands:\n      {}\n    the book says:\n      {}\n    the CLI says:\n      {}",
                t.file,
                t.line,
                t.commands.join("\n      "),
                expected.join("\n      "),
                actual.join("\n      ")
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} transcript{} do not replay:\n  {}\n\nFix the block to say what the CLI says — \
         or, if it genuinely cannot be replayed, declare it above the fence: \
         <!-- transcript: manual — why -->",
        failures.len(),
        if failures.len() == 1 { "" } else { "s" },
        failures.join("\n  ")
    );
}

#[cfg(test)]
mod unit {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn verdict_lines_and_durations_are_normalized_away() {
        let n = normalize(s(&[
            "A B",
            "[computed]",
            "x · cached · main · 12ms   → 3b",
            "",
            "— batch: 2 commands",
        ]));
        assert_eq!(n, s(&["A B", "x · <verdict> · main · <ms>   → 3b"]));
    }

    #[test]
    fn a_continued_block_is_one_session_with_the_block_before_it() {
        let found = transcripts();
        let front_door = found
            .iter()
            .filter(|t| t.file == "front-door/cli.md")
            .collect::<Vec<_>>();
        assert!(
            front_door.iter().any(|t| t.spans.len() >= 3),
            "the front door's sections 5–8 should merge into one sitting: {:?}",
            front_door.iter().map(|t| &t.spans).collect::<Vec<_>>()
        );
    }

    #[test]
    fn an_ellipsis_matches_any_run_of_lines() {
        assert!(matches(&s(&["a", "…", "d"]), &s(&["a", "b", "c", "d"])));
        assert!(matches(&s(&["a", "…"]), &s(&["a", "b"])));
        assert!(!matches(&s(&["a", "d"]), &s(&["a", "b", "d"])));
        assert!(
            !matches(&s(&["a"]), &s(&["a", "b"])),
            "nothing may follow without a trailing …"
        );
    }

    #[test]
    fn the_scanner_finds_the_books_transcripts_and_their_commands() {
        let found = transcripts();
        assert!(found.len() >= 3, "{}", found.len());
        assert!(found.iter().all(|t| !t.commands.is_empty()));
    }
}
