//! What `cargo run -p <package>` prints is what every page says it prints (ledger #165, #1072).
//!
//! The books and the README tell a reader to run the tutorial's own programs, and show what
//! they print in a ```` ```text ```` block right after. Nothing compiles one of those, so a
//! stale one sits there looking authoritative: `mdbook test` runs Rust blocks, the REPL replay
//! (`crates/book-urns/tests/book_transcripts.rs`) runs `ikigai> ` blocks, and the transcript
//! runner (this crate's `main.rs`) runs the gonk book's `console` blocks. This module is the
//! same check for a program: run its binary and compare what it prints with every copy.
//!
//! ## Who runs what
//!
//! A test can only run a binary of its OWN package (Cargo names it in `CARGO_BIN_EXE_<name>`
//! for that package's tests, and for no one else's), so each program's check lives in that
//! program's crate, as a few lines calling [`check`] with its binary. The scanner they share
//! lives here, in a crate with no dependencies, so it can be a dev-dependency of any crate in
//! the workspace without a cycle (a dev-dependency on `book-urns`, which depends on the program
//! crates, would be one). [`GATED`] lists those checks, and [`unchecked`] (run by this crate's
//! own `tests/program_runs.rs`) is the other half: **every** `cargo run` in a shell fence of
//! any page is either a package in [`GATED`] or declared manual, so a new run block nobody
//! checks fails instead of being silently skipped.
//!
//! ## What a page writes
//!
//! A shell fence (`bash`, `sh`, `shell`, `zsh`) holding one or more `cargo run -p <package>`
//! lines, then, with nothing but blank lines or HTML comments between, an output fence
//! (```` ```text ```` or a bare one) showing what the fence prints. Each run is written exactly
//! `cargo run -p <package>`, or that followed by ` -- ARGS`: the check runs the binary in its
//! place, so any other cargo flag would change what the line means. Every run in one fence must
//! be the same package. Other lines in the fence (`mkdir`, `printf … > file`) are setup, run as
//! the reader would run them, in order with the runs; the output fence is everything the fence
//! prints, all runs concatenated. An `{{#include FILE}}` line in the output fence is resolved
//! relative to the page, as mdbook does, so a chapter can share one copy with another.
//!
//! The fence runs as one `sh -e` script, in a fresh scratch directory that is also its working
//! directory, with `HOME` and `XDG_CONFIG_HOME` pointed into it: a page's setup writes into a
//! config home the reader would see, and a check must not write into the maintainer's, nor
//! read a `tutorial.toml` that happens to be there.
//!
//! "Matches" is the transcript runner's rule ([`crate::matches`]): blank lines and trailing
//! space are dropped on both sides, a line that is exactly `…` matches any run of lines, and a
//! `…` inside a line matches any text on it. That is how a page shows the head of a long
//! catalog, or elides a digest, without lying about the rest.
//!
//! ## Deliberately manual
//!
//! A run whose output is the reader's own (the exercises crate) or that needs something a test
//! cannot start (another repository's server on a socket) is declared above its fence, with its
//! reason, the way a gonk book transcript is:
//!
//! ```text
//! <!-- transcript: manual — why -->
//! ```
//!
//! A `console` block with a `$ cargo run` line is refused unless it is declared manual: its
//! output sits inside the block, and only the gonk book's runner reads those.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{declared, matches, normalize, Kind};

/// Every package whose `cargo run` output the books show, and the test that checks it. A test
/// listed here must call [`check`] with its package's name; [`unchecked`] reads the file to
/// make sure it does.
pub const GATED: &[(&str, &str)] = &[
    (
        "building-endpoints",
        "crates/building-endpoints/tests/book_runs.rs",
    ),
    ("hello-camel", "crates/hello-camel/tests/book_runs.rs"),
    (
        "loadable-module",
        "crates/loadable-module/tests/demo_transcript.rs",
    ),
];

/// The repository root: this crate is `crates/book-transcripts`.
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the repository root is two levels up")
}

/// One fenced block: its info string, the line its opening fence is on, its body lines, and
/// what the lines above it declare.
#[derive(Debug)]
pub struct Fence {
    /// The info string (`bash`, `text`, `rust,ignore`, …), trimmed.
    pub info: String,
    /// The opening fence's line, 1-based.
    pub line: usize,
    /// The lines between the fences, verbatim.
    pub body: Vec<String>,
    /// Whether anything but blank lines and HTML comments sits between the previous fence's
    /// close and this fence's open.
    pub prose_before: bool,
    /// The `<!-- transcript: … -->` declaration within the five lines above, if any.
    pub kind: Kind,
}

/// Every fenced block in a page, in order. A fence opens with three or more backticks and
/// closes with at least as many and nothing else, so a four-backtick block can quote a
/// three-backtick one without ending early.
pub fn fences(text: &str) -> Vec<Fence> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let mut open: Option<(usize, Fence)> = None;
    let mut prose_since_close = false;
    let mut in_comment = false;
    for (i, raw) in lines.iter().enumerate() {
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
                        kind: declared(&lines[i.saturating_sub(5)..i]),
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

fn is_console(info: &str) -> bool {
    info.split_whitespace().next() == Some("console")
}

fn is_output(info: &str) -> bool {
    info.is_empty() || info == "text"
}

/// Is this command a `cargo run`?
pub fn is_cargo_run(command: &str) -> bool {
    command.split_whitespace().take(2).eq(["cargo", "run"])
}

/// The package a `cargo run` names with `-p` or `--package`, if it names one.
pub fn package_of(command: &str) -> Option<&str> {
    let words: Vec<&str> = command.split_whitespace().collect();
    let flag = words
        .iter()
        .take_while(|w| **w != "--")
        .position(|w| *w == "-p" || *w == "--package")?;
    words.get(flag + 1).copied()
}

/// A fence's commands: its non-blank lines, trimmed, with `\` continuations joined, and a
/// `console` block's `$ ` prompt removed (its other lines are output, not commands).
fn commands(console: bool, body: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut continuing = false;
    for raw in body {
        let line = raw.trim();
        if continuing {
            let last = out.last_mut().expect("a continuation follows a command");
            last.push(' ');
            last.push_str(line);
        } else {
            let command = if console {
                match line.strip_prefix("$ ") {
                    Some(c) => c.trim(),
                    None => continue,
                }
            } else {
                line
            };
            if command.is_empty() {
                continue;
            }
            out.push(command.to_string());
        }
        let last = out.last_mut().expect("a line was just pushed or extended");
        continuing = last.ends_with('\\');
        if continuing {
            last.pop();
            last.truncate(last.trim_end().len());
        }
    }
    out
}

/// A fence that runs at least one `cargo run`, as a page shows it.
#[derive(Debug)]
pub struct RunBlock {
    /// The page, relative to the repository root.
    pub page: String,
    /// The fence's opening line, 1-based.
    pub line: usize,
    /// Whether it is a `console` block (refused unless manual).
    pub console: bool,
    /// What the lines above it declare.
    pub kind: Kind,
    /// Every command in the fence, in order.
    pub commands: Vec<String>,
    /// The output fence right after it, with its includes resolved; `None` when the next fence
    /// is not an output fence or prose sits between the two.
    pub output: Option<Vec<String>>,
}

impl RunBlock {
    fn at(&self) -> String {
        format!("{}:{}", self.page, self.line)
    }

    /// Does any run in this block run `package`?
    pub fn runs(&self, package: &str) -> bool {
        self.commands
            .iter()
            .any(|c| is_cargo_run(c) && package_of(c) == Some(package))
    }
}

/// The lines a reader sees in an output fence: each `{{#include PATH}}` line replaced by the
/// file it names, relative to the page, the way mdbook does it.
fn rendered(page: &Path, body: &[String]) -> Result<Vec<String>, String> {
    let mut lines = Vec::new();
    for line in body {
        let trimmed = line.trim();
        if let Some(path) = trimmed
            .strip_prefix("{{#include ")
            .and_then(|rest| rest.strip_suffix("}}"))
        {
            if path.contains(':') {
                return Err(format!(
                    "`{trimmed}` includes an anchor or line range; this check resolves a whole \
                     file only"
                ));
            }
            let file = page
                .parent()
                .expect("a page has a directory")
                .join(path.trim());
            let text = std::fs::read_to_string(&file).map_err(|e| format!("{trimmed}: {e}"))?;
            lines.extend(text.lines().map(str::to_string));
        } else {
            lines.push(line.clone());
        }
    }
    Ok(lines)
}

/// The README and every page under `books/*/src/`. Dot-entries are skipped: they are local
/// state, not pages (`serve-with-drafts.sh` leaves `books/.drafts.*` behind, an editor its
/// undo files), and git ignores them, so CI would never see what a local run found there.
pub fn markdown_pages(root: &Path) -> Vec<PathBuf> {
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

/// Every shell or `console` fence, on every page, that runs a `cargo run`. An include the
/// output fence cannot resolve is an error, named by page.
pub fn run_blocks(root: &Path) -> Result<Vec<RunBlock>, String> {
    let mut out = Vec::new();
    for page in markdown_pages(root) {
        let rel = page
            .strip_prefix(root)
            .expect("under the root")
            .to_string_lossy()
            .into_owned();
        let text = std::fs::read_to_string(&page).map_err(|e| format!("{rel}: {e}"))?;
        let fences = fences(&text);
        for (i, fence) in fences.iter().enumerate() {
            let console = is_console(&fence.info);
            if !console && !is_shell(&fence.info) {
                continue;
            }
            let commands = commands(console, &fence.body);
            if !commands.iter().any(|c| is_cargo_run(c)) {
                continue;
            }
            let output = match fences.get(i + 1) {
                Some(next) if !console && is_output(&next.info) && !next.prose_before => Some(
                    rendered(&page, &next.body).map_err(|e| format!("{rel}:{}: {e}", next.line))?,
                ),
                _ => None,
            };
            out.push(RunBlock {
                page: rel.clone(),
                line: fence.line,
                console,
                kind: fence.kind.clone(),
                commands,
                output,
            });
        }
    }
    Ok(out)
}

/// The fence as a script for `sh`: setup lines as written, and each run of `package` with the
/// binary, named by `$BOOK_RUN_BIN`, in place of `cargo run -p <package> --`.
pub fn script(package: &str, commands: &[String]) -> Result<String, String> {
    let prefix = format!("cargo run -p {package}");
    let mut lines = Vec::new();
    for command in commands {
        if !is_cargo_run(command) {
            lines.push(command.clone());
            continue;
        }
        if package_of(command) != Some(package) {
            return Err(format!(
                "`{command}` runs another package in the same fence as `{prefix}`; give each \
                 program a fence of its own, so its output fence is its output alone"
            ));
        }
        let args = match command.strip_prefix(&prefix) {
            Some("") | Some(" --") => "",
            Some(rest) if rest.starts_with(" -- ") => &rest[" --".len()..],
            _ => {
                return Err(format!(
                    "write the run as `{prefix}` or `{prefix} -- ARGS` (found `{command}`): the \
                     check runs the binary in its place, so any other cargo flag would change \
                     what the line means"
                ))
            }
        };
        lines.push(format!("\"$BOOK_RUN_BIN\"{args}"));
    }
    Ok(lines.join("\n"))
}

/// Run a fence's script in a fresh scratch directory (its working directory, `HOME` and
/// `XDG_CONFIG_HOME`), with `bin` as `$BOOK_RUN_BIN`; what it printed on stdout.
pub fn run(script: &str, bin: &str) -> Result<String, String> {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "book-runs-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    let home = dir.join("home");
    let config = dir.join("config");
    for d in [&home, &config] {
        std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
    }
    let out = Command::new("sh")
        .arg("-ec")
        .arg(script)
        .current_dir(&dir)
        .env("HOME", &home)
        .env("XDG_CONFIG_HOME", &config)
        .env("BOOK_RUN_BIN", bin)
        .output();
    let _ = std::fs::remove_dir_all(&dir);
    let out = out.map_err(|e| format!("sh did not start: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "the fence failed ({}):\n{}",
            out.status,
            indent(String::from_utf8_lossy(&out.stderr).lines())
        ));
    }
    String::from_utf8(out.stdout).map_err(|e| format!("the fence printed non-UTF-8: {e}"))
}

/// Check every copy of `package`'s output against `bin`, the binary Cargo built for the
/// calling test (`env!("CARGO_BIN_EXE_<name>")`), and panic with every difference.
///
/// `expected_pages` is the guard on the scan itself: each must hold at least one checked run,
/// so a page that stops being found (a renamed fence language, a moved file) is reported
/// instead of passing on fewer copies. A copy the check cannot check fails rather than being
/// skipped: a run with no output fence after it, a run spelled with other cargo flags, or one
/// sharing its fence with another package.
pub fn check(package: &str, bin: &str, expected_pages: &[&str]) {
    let root = repo_root();
    let blocks = run_blocks(&root).unwrap_or_else(|e| panic!("{e}"));
    let mut found: Vec<String> = Vec::new();
    let mut problems = Vec::new();
    for block in blocks.iter().filter(|b| b.runs(package)) {
        if matches!(block.kind, Kind::Manual(_)) {
            continue;
        }
        let at = block.at();
        if block.console {
            problems.push(format!(
                "{at}: a `console` block runs `cargo run -p {package}`; use a shell fence \
                 followed by its output fence, or declare it manual"
            ));
            continue;
        }
        let script = match script(package, &block.commands) {
            Ok(s) => s,
            Err(e) => {
                problems.push(format!("{at}: {e}"));
                continue;
            }
        };
        let Some(expected) = &block.output else {
            problems.push(format!(
                "{at}: `cargo run -p {package}` is not followed by its output (a ```text or \
                 bare fence, with nothing but blank lines or comments between them), and is \
                 not declared `<!-- transcript: manual — why -->`"
            ));
            continue;
        };
        match run(&script, bin) {
            Err(e) => problems.push(format!("{at}: {e}")),
            Ok(actual) => {
                let expected = normalize(expected);
                let actual = normalize(actual.lines());
                if !matches(&expected, &actual) {
                    problems.push(format!(
                        "{at}: the page shows\n{}\nbut the fence prints\n{}",
                        indent(&expected),
                        indent(&actual)
                    ));
                }
                found.push(block.page.clone());
            }
        }
    }
    for page in expected_pages {
        if !found.iter().any(|f| f == page) {
            problems.push(format!(
                "{page}: expected a checked run of `{package}` here and the scan found none; if \
                 it moved, update the test's expected pages"
            ));
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n\n"));
    found.dedup();
    eprintln!(
        "`cargo run -p {package}`: {} cop{} checked: {}",
        found.len(),
        if found.len() == 1 { "y" } else { "ies" },
        found.join(", ")
    );
}

/// Every `cargo run` on every page that nothing checks and nothing declares manual, and every
/// [`GATED`] entry whose test does not call [`check`] for its package. Empty is the pass.
pub fn unchecked(root: &Path) -> Vec<String> {
    let mut problems = Vec::new();
    let blocks = match run_blocks(root) {
        Ok(b) => b,
        Err(e) => return vec![e],
    };
    for block in &blocks {
        let at = block.at();
        if let Kind::Manual(reason) = &block.kind {
            if reason.is_empty() {
                problems.push(format!(
                    "{at}: declared manual with no reason; say why, where an editor reads it"
                ));
            }
            continue;
        }
        if block.console {
            problems.push(format!(
                "{at}: a `console` block with a `cargo run`; use a shell fence followed by its \
                 output fence, or declare it `<!-- transcript: manual — why -->`"
            ));
            continue;
        }
        for command in block.commands.iter().filter(|c| is_cargo_run(c)) {
            match package_of(command) {
                Some(p) if GATED.iter().any(|(g, _)| *g == p) => {}
                Some(p) => problems.push(format!(
                    "{at}: `{command}` is neither checked nor declared manual: add `{p}` to \
                     book_transcripts::runs::GATED with a test that calls `runs::check`, or \
                     declare `<!-- transcript: manual — why -->` above the fence"
                )),
                None => problems.push(format!(
                    "{at}: `{command}` names no package (`-p`), so nothing can say which \
                     program's output to check"
                )),
            }
        }
    }
    for (package, test) in GATED {
        match std::fs::read_to_string(root.join(test)) {
            Ok(text)
                if text.contains("runs::check(") && text.contains(&format!("\"{package}\"")) => {}
            Ok(_) => problems.push(format!(
                "{test}: listed in GATED for `{package}` but does not call \
                 `runs::check(\"{package}\", …)`"
            )),
            Err(e) => problems.push(format!("{test}: listed in GATED for `{package}`: {e}")),
        }
    }
    problems
}

fn indent<I, S>(lines: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    lines
        .into_iter()
        .map(|l| format!("    | {}", l.as_ref()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The scanner's own rules, on text small enough to read.
#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

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
        let f = fences(
            "```bash\ncargo run -p loadable-module\n```\n\nSome prose.\n\n```text\nx\n```\n",
        );
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

    #[test]
    fn a_manual_declaration_above_a_fence_is_read_and_does_not_reach_the_next() {
        let f = fences(
            "<!-- transcript: manual — the reader's own crate -->\n```bash\ncargo run -p x\n```\n\n```bash\ncargo run -p y\n```\n",
        );
        assert_eq!(f[0].kind, Kind::Manual("the reader's own crate".into()));
        assert_eq!(f[1].kind, Kind::Undeclared);
    }

    #[test]
    fn the_package_is_the_word_after_dash_p_and_never_an_argument() {
        assert_eq!(
            package_of("cargo run -p hello-camel -- --catalog"),
            Some("hello-camel")
        );
        assert_eq!(package_of("cargo run --package a"), Some("a"));
        assert_eq!(package_of("cargo run -- -p a"), None);
        assert_eq!(package_of("cargo run"), None);
        assert!(is_cargo_run("cargo run -p a"));
        assert!(!is_cargo_run("cargo install ikigai-cli"));
    }

    #[test]
    fn commands_join_continuations_and_a_console_block_keeps_only_prompts() {
        assert_eq!(
            commands(
                false,
                &s(&["  mkdir -p x", "", "cargo run -p a \\", "  -- b"])
            ),
            s(&["mkdir -p x", "cargo run -p a -- b"])
        );
        assert_eq!(
            commands(true, &s(&["$ cargo run -p a", "it printed this"])),
            s(&["cargo run -p a"])
        );
    }

    #[test]
    fn a_script_runs_setup_as_written_and_the_binary_in_place_of_each_run() {
        let script = script(
            "b",
            &s(&[
                "printf 'x' > f",
                "cargo run -p b -- --banner",
                "cargo run -p b",
                "cargo run -p b -- \"two words\"",
            ]),
        )
        .unwrap();
        assert_eq!(
            script,
            "printf 'x' > f\n\"$BOOK_RUN_BIN\" --banner\n\"$BOOK_RUN_BIN\"\n\"$BOOK_RUN_BIN\" \"two words\""
        );
    }

    #[test]
    fn a_script_refuses_what_it_cannot_run_faithfully() {
        assert!(script("b", &s(&["cargo run -p b", "cargo run -p c"])).is_err());
        assert!(script("b", &s(&["cargo run -p b --release"])).is_err());
        assert!(script("b", &s(&["cargo run -p b --bin x"])).is_err());
        assert!(script("b", &s(&["cargo run -p b --x"])).is_err());
    }

    #[test]
    fn a_fence_runs_in_a_scratch_home_and_its_stdout_is_the_answer() {
        // `echo` stands in for a binary: what matters is the substitution, the setup and the
        // home the fence sees.
        let out = run(
            "mkdir -p \"$XDG_CONFIG_HOME/ikigai\"\nprintf 'hi\\n' > \"$XDG_CONFIG_HOME/ikigai/t\"\ncat \"$XDG_CONFIG_HOME/ikigai/t\"\n\"$BOOK_RUN_BIN\" one",
            "echo",
        )
        .unwrap();
        assert_eq!(out, "hi\none\n");
        assert!(run("false", "echo").is_err(), "a failing fence fails");
    }
}
