//! Find every shell transcript a book prints, and decide whether a run matches it.
//!
//! A ```` ```console ```` fence whose lines start with `$ ` is a **transcript**: the book
//! saying "type this, see that". Nothing compiles one, so nothing noticed when one went stale
//! (ledger #165) — `mdbook test` runs Rust, and the ikigai book's REPL replay
//! (`crates/book-urns/tests/book_transcripts.rs`) needs a binary CI does not have. This crate
//! is the half that needs no binary: the scanner and the matcher, unit-tested here. The
//! runner (`src/main.rs`) runs what the scanner found, in a scratch home, against the binaries
//! it is pointed at, and asks the matcher.
//!
//! ## What a block declares
//!
//! An HTML comment within the five lines above the fence, so it is read as belonging to it:
//!
//! ```text
//! <!-- transcript: run -->        each `$ ` line runs to completion, in order (the default)
//! <!-- transcript: serve -->      ONE `$ ` line, started and left running for the rest of
//!                                 the page; what it prints at startup is the expected output
//! <!-- transcript: manual — why --> not run, and the reason is said where an editor reads it
//! <!-- transcript: file NAME -->  ANY fence (toml, markdown, json…): its lines are written to
//!                                 NAME in the page's scratch directory, at that point in the
//!                                 page, creating directories — the page saying "save this as
//!                                 NAME", which the check then does
//! ```
//!
//! A `file` block is how a page gives its commands an input (a config file, a fixture) without
//! a heredoc: each command is one `sh -c` line, so a multi-line file cannot be typed into one,
//! and the reader sees the file's whole content where they would copy it from. Its NAME must be
//! relative and stay inside the scratch directory (no `..`), or the block fails.
//!
//! Under `--strict` (how the gonk book is checked) a `console` block with a `$ ` line and no
//! declaration is a FAILURE, not a skip: a transcript nobody runs is the thing this exists to
//! stop, and "I forgot to declare it" must not look like "it passed".
//!
//! ## What a page is
//!
//! **One page is one session**: one scratch directory, which is also `HOME` and (under
//! `.config`) the config home, shared by every block on the page in page order — so a server a
//! `serve` block starts is the one the next block's `curl` reaches, and an item filed in one
//! block is listed in the next. Each `$ ` line is its own `sh -c`, so a `cd` or a shell variable
//! does not carry; the files do. That is a constraint on how a chapter is written, and it is
//! stated here rather than discovered.
//!
//! ## What "matches" means
//!
//! Blank lines and trailing whitespace are dropped on both sides. Then an expected line that is
//! exactly `…` matches any run of lines, and a `…` INSIDE a line matches any run of characters
//! on that line — which is how a transcript says "an id is minted here" or "a timestamp" without
//! the page lying about what the reader will see. Everything else matches exactly.

use std::fs;
use std::path::{Path, PathBuf};

/// The line prefix a transcript's commands carry.
pub const PROMPT: &str = "$ ";

/// What a block declares about itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    /// Run each command to completion, in order.
    Run,
    /// Start the one command and leave it running for the rest of the page.
    Serve,
    /// Not run, for the reason given.
    Manual(String),
    /// Write the block's lines to this path, relative to the page's scratch directory.
    File(String),
    /// Nothing declared. A skip normally, a failure under `--strict`.
    Undeclared,
}

/// One command of a block, and what the page says it prints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    /// The command, with `\` continuations joined into one line.
    pub text: String,
    /// The lines the page shows after it, up to the next command.
    pub expected: Vec<String>,
}

/// One transcript block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// The page, relative to the book's `src/`.
    pub file: String,
    /// The block's first line in that page, 1-based.
    pub line: usize,
    /// What it declares.
    pub kind: Kind,
    /// Its commands, in order.
    pub commands: Vec<Command>,
    /// Lines before the first command: output with nothing to have printed it.
    pub orphans: Vec<String>,
    /// A `file` block's lines, verbatim; empty for every other kind.
    pub content: Vec<String>,
}

/// Every transcript block in one page's markdown: each `console` fence with a `$ ` line, and
/// each fence of any language declared `file`, in page order.
pub fn scan_page(file: &str, text: &str) -> Vec<Block> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let fence = lines[i].trim();
        if !fence.starts_with("```") {
            i += 1;
            continue;
        }
        let start = i + 1;
        let mut end = start;
        while end < lines.len() && !lines[end].trim_start().starts_with("```") {
            end += 1;
        }
        let body = &lines[start..end];
        let above = &lines[i.saturating_sub(5)..i];
        let kind = declared(above);
        if let Kind::File(_) = kind {
            out.push(Block {
                file: file.to_string(),
                line: start + 1,
                kind,
                commands: Vec::new(),
                orphans: Vec::new(),
                content: body.iter().map(|l| l.to_string()).collect(),
            });
        } else if fence == "```console" && body.iter().any(|l| l.starts_with(PROMPT)) {
            out.push(Block {
                file: file.to_string(),
                line: start + 1,
                kind,
                commands: commands(body),
                orphans: body
                    .iter()
                    .take_while(|l| !l.starts_with(PROMPT))
                    .filter(|l| !l.trim().is_empty())
                    .map(|l| l.to_string())
                    .collect(),
                content: Vec::new(),
            });
        }
        i = end + 1;
    }
    out
}

/// Is `name` a path a `file` block may write: relative, and with no `..` to leave the scratch
/// directory by?
pub fn safe_file_name(name: &str) -> bool {
    let path = Path::new(name);
    !name.is_empty()
        && path.is_relative()
        && path
            .components()
            .all(|c| matches!(c, std::path::Component::Normal(_)))
}

/// The declaration in the lines above a fence, the nearest one winning. The search stops at
/// the end of an earlier fence: a declaration belongs to the first block after it, never to
/// the next one as well.
fn declared(above: &[&str]) -> Kind {
    for line in above.iter().rev() {
        if line.trim_start().starts_with("```") {
            break;
        }
        let Some((_, rest)) = line.split_once("transcript:") else {
            continue;
        };
        let rest = rest.trim().trim_end_matches("-->").trim();
        if let Some(name) = rest.strip_prefix("file ") {
            return Kind::File(name.trim().to_string());
        }
        if let Some(reason) = rest.strip_prefix("manual") {
            let reason = reason.trim().trim_start_matches(['—', '-', ':']).trim();
            return Kind::Manual(reason.to_string());
        }
        return match rest {
            "serve" => Kind::Serve,
            _ => Kind::Run,
        };
    }
    Kind::Undeclared
}

/// A block's commands, each with the output lines that follow it.
fn commands(body: &[&str]) -> Vec<Command> {
    let mut out: Vec<Command> = Vec::new();
    let mut i = 0;
    while i < body.len() {
        let line = body[i];
        if let Some(first) = line.strip_prefix(PROMPT) {
            let mut text = first.to_string();
            while text.ends_with('\\') && i + 1 < body.len() {
                text.pop();
                text.truncate(text.trim_end().len());
                i += 1;
                text.push(' ');
                text.push_str(body[i].trim());
            }
            out.push(Command {
                text: text.trim().to_string(),
                expected: Vec::new(),
            });
        } else if let Some(last) = out.last_mut() {
            last.expected.push(line.to_string());
        }
        i += 1;
    }
    out
}

/// Every markdown page under `dir`, sorted, so a report is stable.
pub fn pages(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    collect(dir, &mut out)?;
    out.sort();
    Ok(out)
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(path);
        }
    }
    Ok(())
}

/// Drop what legitimately differs and means nothing: blank lines and trailing whitespace.
pub fn normalize<I, S>(lines: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    lines
        .into_iter()
        .map(|l| l.as_ref().trim_end().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

/// The marker that matches anything: a whole line of it, any lines; inside a line, any text.
pub const ANY: &str = "…";

/// Does one actual line match one expected line, where `…` matches any run of characters?
pub fn line_matches(expected: &str, actual: &str) -> bool {
    let parts: Vec<&str> = expected.split(ANY).collect();
    if parts.len() == 1 {
        return expected == actual;
    }
    let (first, rest) = parts.split_first().expect("split yields one part at least");
    let Some(mut tail) = actual.strip_prefix(first) else {
        return false;
    };
    let (last, middle) = rest.split_last().expect("two parts at least");
    for part in middle {
        match tail.find(part) {
            Some(at) => tail = &tail[at + part.len()..],
            None => return false,
        }
    }
    tail.len() >= last.len() && tail.ends_with(last)
}

/// Does `actual` match `expected`, line by line, where a line that is exactly `…` matches any
/// run of lines (none included) and a `…` within a line matches any text on it?
pub fn matches(expected: &[String], actual: &[String]) -> bool {
    match expected.split_first() {
        None => actual.is_empty(),
        Some((head, rest)) if head.trim() == ANY => {
            (0..=actual.len()).any(|skip| matches(rest, &actual[skip..]))
        }
        Some((head, rest)) => match actual.split_first() {
            Some((first, more)) => line_matches(head, first) && matches(rest, more),
            None => false,
        },
    }
}

/// Is this command forbidden to run on a machine that has a real gonk? Returns why.
///
/// A real gonk listens on 1060 (HTTP, and QUIC on UDP) and its socket is `~/.ikigai/gonk.sock`.
/// The runner gives every command a scratch `HOME`, so `~` and the default socket are scratch;
/// a PORT is not, and a `curl … :1060/iki/ledger/append` in a page would file an item in the
/// reader's — or the maintainer's — real ledger every time the check ran. So a page may not
/// name that port in anything the runner executes, and the check refuses before running it.
pub fn refused(command: &str) -> Option<&'static str> {
    if command.contains("1060") {
        return Some(
            "names port 1060, a real gonk's port — a book's transcript must run against a \
             scratch gonk on a port of its own (the gonk book uses 1070)",
        );
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn an_inline_ellipsis_matches_any_text_on_the_line() {
        assert!(line_matches(
            "#1 urn:iki:ledger:default:item:…",
            "#1 urn:iki:ledger:default:item:01m495wy306p819ms8h5jyc"
        ));
        assert!(line_matches("a … b … c", "a x b y c"));
        assert!(line_matches("…", ""));
        assert!(!line_matches("#1 …", "#2 x"));
        assert!(!line_matches("a…b", "ab-"), "the end is anchored");
        assert!(
            !line_matches("ab…ba", "aba"),
            "the two ends may not overlap"
        );
    }

    #[test]
    fn a_line_of_ellipsis_matches_any_run_of_lines() {
        assert!(matches(&s(&["a", "…", "d"]), &s(&["a", "b", "c", "d"])));
        assert!(matches(&s(&["a", "…", "d"]), &s(&["a", "d"])));
        assert!(matches(&s(&["a", "…"]), &s(&["a", "b"])));
        assert!(!matches(&s(&["a", "d"]), &s(&["a", "b", "d"])));
        assert!(!matches(&s(&["a"]), &s(&["a", "b"])), "nothing may follow");
        assert!(matches(&s(&["…", "x …"]), &s(&["a", "x 1", "x 2"])));
    }

    #[test]
    fn blank_lines_and_trailing_space_are_not_differences() {
        assert_eq!(normalize(["a  ", "", "b"]), s(&["a", "b"]));
    }

    #[test]
    fn a_page_yields_its_blocks_their_declarations_and_their_commands() {
        let page = "\
Start it:

<!-- transcript: serve -->
```console
$ ikigai-gonk --port 1070
ikigai-gonk 0.1.0 — holding the store at …
```

<!-- transcript: run -->
```console
$ curl -s -X POST --data-binary 'x' \\
    http://127.0.0.1:1070/iki/ledger/append
#1 urn:iki:ledger:default:item:…

$ curl -s http://127.0.0.1:1070/iki/ledger/items
   #1  open    p-  x
```

<!-- transcript: manual — it opens a browser -->
```console
$ open http://localhost:1070/
```

```console
$ echo undeclared
undeclared
```

```text
$ not a console fence, not a transcript
```
";
        let blocks = scan_page("p.md", page);
        assert_eq!(blocks.len(), 4, "{blocks:#?}");
        assert_eq!(blocks[0].kind, Kind::Serve);
        assert_eq!(blocks[0].line, 5);
        assert_eq!(blocks[1].kind, Kind::Run);
        assert_eq!(
            blocks[1].commands[0].text,
            "curl -s -X POST --data-binary 'x' http://127.0.0.1:1070/iki/ledger/append"
        );
        assert_eq!(
            normalize(&blocks[1].commands[0].expected),
            s(&["#1 urn:iki:ledger:default:item:…"])
        );
        assert_eq!(blocks[1].commands.len(), 2);
        assert_eq!(
            blocks[2].kind,
            Kind::Manual("it opens a browser".to_string())
        );
        assert_eq!(blocks[3].kind, Kind::Undeclared);
    }

    #[test]
    fn a_file_declaration_takes_any_fence_and_keeps_its_lines_verbatim() {
        let page = "\
<!-- transcript: file .config/ikigai/config.toml -->
```toml
gonk.port = 1070
  # indented, kept
```

```toml
not = \"declared, so not a block\"
```

<!-- transcript: run -->
```console
$ cat .config/ikigai/config.toml
gonk.port = 1070
```
";
        let blocks = scan_page("p.md", page);
        assert_eq!(blocks.len(), 2, "{blocks:#?}");
        assert_eq!(
            blocks[0].kind,
            Kind::File(".config/ikigai/config.toml".to_string())
        );
        assert_eq!(
            blocks[0].content,
            s(&["gonk.port = 1070", "  # indented, kept"])
        );
        assert!(blocks[0].commands.is_empty());
        assert_eq!(blocks[1].kind, Kind::Run);
    }

    #[test]
    fn a_file_block_may_not_leave_the_scratch_directory() {
        assert!(safe_file_name("review.md"));
        assert!(safe_file_name(".config/ikigai/config.toml"));
        assert!(!safe_file_name("../escape"));
        assert!(!safe_file_name("a/../../b"));
        assert!(!safe_file_name("/etc/passwd"));
        assert!(!safe_file_name(""));
    }

    #[test]
    fn output_before_any_command_is_an_orphan() {
        let blocks = scan_page("p.md", "```console\nstray\n$ true\n```\n");
        assert_eq!(blocks[0].orphans, s(&["stray"]));
    }

    #[test]
    fn the_real_gonks_port_is_refused() {
        assert!(refused("curl http://127.0.0.1:1060/iki/ledger/items").is_some());
        assert!(refused("curl http://127.0.0.1:1070/iki/ledger/items").is_none());
    }
}
