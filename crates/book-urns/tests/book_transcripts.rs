//! Every REPL transcript in the book is replayed against a real `ikigai`.
//!
//! A ```` ```text ```` block whose lines start with `ikigai> ` is a transcript: the book
//! saying "type this, see that". Nothing compiles one, so one was written to fit its
//! prose — the front door's "Cut a thread" showed `sink urn:kernel:cut urn:iki:fn:toUpper`
//! followed by `not cached` when the real CLI said `cached` (before core 0.1.73 a cut
//! reached only threads an endpoint declared, and a pure function declares none). The prose
//! was wrong and the transcript agreed with it, which is the failure mode this file exists
//! for. ⚠ And then the kernel changed: since 0.1.73 every cacheable read hangs from a thread
//! named after its own target, the CLI says `not cached`, and the corrected block went stale
//! in turn, red here and invisible, because CI does not run this file (ledger #998).
//!
//! The replay types every `ikigai> ` line of a block into ONE `ikigai --quiet` REPL, a line
//! at a time on its stdin, the way a reader at the prompt does, so the cache carries across
//! the lines as it does in their session; then it diffs what the REPL printed (stdout and
//! stderr in the one order they were written) against the block's non-prompt lines.
//! `--quiet` (`ikigai-cli` 0.1.55, ledger #1310) prints exactly the answers: no version
//! banner, no topology lines, and no prompt when stdin is not a terminal. Until it existed the
//! replay ran `--plain` and took the greeting and the prompts off the screen by hand, which
//! is what this file no longer does (ledger #1353). ⚠ It used to pass the lines as repeated `-c` arguments instead, and a
//! `-c` batch is not a REPL: from `ikigai-cli` 0.1.48 a batch whose stdin is not a terminal
//! fetches (and caches) the contract of every bare `source`, in case stdin fills its one
//! required argument (ledger #1166), so the front door's `urn:kernel:cache` listed five rows
//! where a reader sees three, and the book's pin could not move past 0.1.46 (ledger #1304).
//! Two kinds of
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
//! `not cached` because a fresh process has nothing cached, whatever the cut did, so
//! isolation would pass the block whether the kernel honored the cut or not. A block that
//! cannot be replayed at all declares itself with a reason:
//!
//! ```text
//! <!-- transcript: manual — the capability line prints this machine's home directory -->
//! ```
//!
//! A page whose REPL steps read files the reader was told to fetch first (the recorded week of
//! "Exploring without Rust", which the part's first page downloads into `~/quakes` and starts
//! the REPL over with `--files ~/quakes`) names them ONCE, anywhere on the page, as paths from
//! the repository's root:
//!
//! ```text
//! <!-- transcripts: workspace crates/quakes/quakes-2024-01-01.geojson -->
//! ```
//!
//! Every transcript on that page then starts its REPL with `--files` over a fresh directory
//! in its scratch home holding a copy of each, under its own file name: the workspace the
//! page set up, from the same bytes the reader's `curl` fetches. Nothing else on the page
//! changes; a file a block writes there is still written by the block.
//!
//! A path that names a DIRECTORY is a workspace the reader already has whole (an example's
//! `ws/`, which its own `./iki` serves with `--files`): what it holds, subdirectories and all,
//! is copied to the workspace's root, so `urn:file:feeds/alice.example.com.xml` names the
//! same file in the replay as at the reader's prompt. Only what git tracks is copied, so a
//! file a local run of the example wrote there (and `.gitignore` leaves out) is not in the
//! replay, as it is not in CI's checkout.
//!
//! Every transcript runs in a SCRATCH home: a fresh directory that is the working
//! directory, `HOME` and (under `.config`) the config home of its one `ikigai` process. The
//! front door writes `urn:file:notes.txt`, which lands in `~/.ikigai/workspace/`, and a
//! replay under the reader's real home wrote it there, and read whatever config home the
//! reader had, so a host configured differently could fail or pass for reasons that are not
//! the book's.
//!
//! Like the CLI vocabulary probe, this needs a binary and is `#[ignore]`d rather than
//! silently skipped. CI runs it (ledger #1023): `scripts/test-transcripts.sh`, in
//! `pages.yml`'s `transcripts` job, installs `ikigai-cli` at the version in
//! `books/ikigai/ikigai-cli.version` and runs this with that binary first on PATH. By hand:
//!
//!     ./scripts/test-transcripts.sh          # the pinned cli, and the gonk Book's check too
//!     cargo test -p book-urns --test book_transcripts -- --ignored --nocapture   # whatever `ikigai` is on PATH

use std::path::{Path, PathBuf};
use std::process::Command;

use book_transcripts::{declared, page_workspace, Kind};

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
    /// The repository files the page's file workspace starts with (`transcripts: workspace`).
    workspace: Vec<String>,
}

fn repo_root() -> PathBuf {
    book_src()
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
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
        scan_page(&file, &text, &mut out);
    }
    out
}

/// One page's transcripts, appended to `out` (a `continues` block merges into the one before).
///
/// What a block declares is read by `book_transcripts::declared`, the parser every transcript
/// check shares (ledger #1075). This file used to read its own: any line in the five above the
/// fence, with no stop at an earlier fence, so a program run declared manual a few lines above a
/// REPL block exempted the REPL block too, and that block passed without being replayed. And a
/// reason wrapped over two lines lost its second.
fn scan_page(file: &str, text: &str, out: &mut Vec<Transcript>) {
    let workspace = page_workspace(text);
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
        // A declaration is an HTML comment within the five lines above the fence — close
        // enough to be read as belonging to the block, and never past an earlier fence.
        let kind = declared(&lines[i.saturating_sub(5)..i]);
        let manual = match &kind {
            Kind::Manual(reason) => Some(reason.clone()),
            _ => None,
        };
        let continues = kind == Kind::Continues;
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
                file: file.to_string(),
                line: start,
                spans: vec![start],
                commands,
                expected,
                manual,
                workspace: workspace.clone(),
            }),
        }
        i = end + 1;
    }
}

/// A transcript's own home, removed when the transcript is done, and the file beside it
/// that the REPL's screen is written to.
struct ScratchHome {
    dir: PathBuf,
    /// The file workspace the REPL starts over (`--files`), when the page declared one.
    files: Option<PathBuf>,
    /// Outside the home, so no command a page runs there can list or read it.
    screen: PathBuf,
}

impl ScratchHome {
    fn new(n: usize, workspace: &[String]) -> ScratchHome {
        let base = format!("book-transcripts-{}-{n}", std::process::id());
        let dir = std::env::temp_dir().join(&base);
        let screen = std::env::temp_dir().join(format!("{base}.screen"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".config")).expect("a scratch home");
        let files = (!workspace.is_empty()).then(|| {
            let files = dir.join("files");
            std::fs::create_dir_all(&files).expect("a scratch workspace");
            for path in workspace {
                let from = repo_root().join(path);
                if from.is_dir() {
                    copy_tracked(path, &files);
                    continue;
                }
                let name = from.file_name().expect("a workspace file has a name");
                std::fs::copy(&from, files.join(name))
                    .unwrap_or_else(|e| panic!("copying the page's workspace file {path}: {e}"));
            }
            files
        });
        ScratchHome { dir, files, screen }
    }
}

/// Copy every file git tracks under the repository directory `dir` into `to`, keeping its
/// path below `dir`.
fn copy_tracked(dir: &str, to: &Path) {
    let root = repo_root().join(dir);
    let listed = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["ls-files", "-z", "--", "."])
        .output()
        .unwrap_or_else(|e| panic!("listing the files git tracks in {dir}: {e}"));
    assert!(
        listed.status.success(),
        "git ls-files in {dir}: {}",
        String::from_utf8_lossy(&listed.stderr)
    );
    let names = String::from_utf8(listed.stdout).expect("file names are UTF-8");
    let names: Vec<&str> = names.split('\0').filter(|n| !n.is_empty()).collect();
    assert!(
        !names.is_empty(),
        "`transcripts: workspace` names {dir}, which holds no tracked file"
    );
    for name in names {
        let target = to.join(name);
        std::fs::create_dir_all(target.parent().expect("under the workspace"))
            .expect("a workspace subdirectory");
        std::fs::copy(root.join(name), &target)
            .unwrap_or_else(|e| panic!("copying the page's workspace file {dir}/{name}: {e}"));
    }
}

impl Drop for ScratchHome {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
        let _ = std::fs::remove_file(&self.screen);
    }
}

/// Type `commands` into one `ikigai --quiet` REPL in `home`, a line each, and return what its
/// screen showed: stdout and stderr as ONE stream, in the order the REPL wrote them, because
/// that is what a reader's terminal shows (an error under the command that caused it, not
/// after the whole session). With `--quiet` and a stdin that is not a terminal, that screen is
/// the answers and nothing else, so it is compared as it is.
///
/// The two streams share one file rather than two pipes: two pipes read separately lose the
/// order between them, and one descriptor behind both keeps it (the REPL flushes each answer
/// as it writes it, and stderr is unbuffered).
///
/// The lines are typed at a READER's pace, not pasted: each is written once the answer to the
/// one before has landed and the screen has been still for [`SETTLED`]. ⚠ A paste raced the
/// CLI's file watcher. From `ikigai-cli` 0.2 a `sink urn:file:…` stages its bytes in a
/// `.ikigai-fs-<hex>.tmp` and renames it into place, and the watcher cuts the file's thread
/// when that event arrives, on its own thread; pasted, the `source` on the next line could be
/// served before the cut and then made stale by it, so "The same four things from a shell"
/// printed `not cached` for a read it had just made, about one replay in twelve (ledger
/// #2554). A reader at the prompt takes far longer than the watcher between two lines, so the
/// pause is what they see. Nothing is waited for longer than [`ANSWER_WAIT`]: a command that
/// prints nothing is typed past once that has passed.
///
/// The pace is also why a `cache` readout lists `urn:time:now`: the CLI's host reads its clock
/// every second (a job it registers for the full-screen REPL's tab bar) and that answer is
/// cached until the next minute, so a session older than a second, which every reader's is,
/// holds it. In the second after a minute turns, before the job reads it again, that row says
/// `expired` (2 readouts in 330, measured on 0.2.5), so no session is started in the last
/// [`MINUTE_MARGIN`] of a minute: no session here lasts that long, so none spans the turn.
fn type_into_repl(home: &ScratchHome, commands: &[String]) -> std::io::Result<String> {
    use std::io::Write;
    use std::process::Stdio;

    clear_of_the_minute();
    let screen = std::fs::File::create(&home.screen)?;
    let mut ikigai = Command::new("ikigai");
    ikigai
        .current_dir(&home.dir)
        .env("HOME", &home.dir)
        .env("XDG_CONFIG_HOME", home.dir.join(".config"))
        .arg("--quiet");
    if let Some(files) = &home.files {
        ikigai.arg("--files").arg(files);
    }
    let mut child = ikigai
        .stdin(Stdio::piped())
        .stdout(Stdio::from(screen.try_clone()?))
        .stderr(Stdio::from(screen))
        .spawn()?;
    {
        let mut stdin = child.stdin.take().expect("stdin was piped");
        for c in commands {
            let before = screen_len(home);
            writeln!(stdin, "{c}")?;
            stdin.flush()?;
            wait_for_answer(home, before);
        }
        // Dropping stdin is the reader's Ctrl-D: the REPL ends at EOF.
    }
    child.wait()?;
    std::fs::read_to_string(&home.screen)
}

/// How long the screen must be still after an answer before the next line is typed.
const SETTLED: std::time::Duration = std::time::Duration::from_millis(200);
/// How close to the turn of a minute a session may not start.
const MINUTE_MARGIN: u64 = 10;

/// Wait, when the minute is about to turn, until it has (and a second more).
fn clear_of_the_minute() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("after 1970");
    let into = now.as_secs() % 60;
    if into >= 60 - MINUTE_MARGIN {
        std::thread::sleep(std::time::Duration::from_secs(61 - into));
    }
}
/// How long a line's answer is waited for before the next line is typed anyway.
const ANSWER_WAIT: std::time::Duration = std::time::Duration::from_secs(10);

fn screen_len(home: &ScratchHome) -> u64 {
    std::fs::metadata(&home.screen)
        .map(|m| m.len())
        .unwrap_or(0)
}

/// Wait until the screen has grown past `before` and then been still for [`SETTLED`], or
/// [`ANSWER_WAIT`] has passed.
fn wait_for_answer(home: &ScratchHome, before: u64) {
    let start = std::time::Instant::now();
    let mut last = before;
    let mut still_since = std::time::Instant::now();
    while start.elapsed() < ANSWER_WAIT {
        std::thread::sleep(std::time::Duration::from_millis(10));
        let now = screen_len(home);
        if now != last {
            last = now;
            still_since = std::time::Instant::now();
        } else if now > before && still_since.elapsed() >= SETTLED {
            return;
        }
    }
}

/// Drop the lines that legitimately differ between the book and a replay: cache verdicts
/// and the batch summary; blank lines; and rewrite durations and a file thread's generation.
fn normalize(lines: impl IntoIterator<Item = String>) -> Vec<String> {
    let lines = lines
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
        .map(file_generation)
        .collect();
    unorder_cache_ties(lines)
}

/// `urn:file:notes.txt  gen 2` → `urn:file:notes.txt  gen <n>`.
///
/// The CLI watches its file root and cuts `urn:file:<path>` on every change event, its own
/// writes included, on the watcher's schedule rather than the command's. macOS delivers
/// those events after a short replay has exited, so the book's `gen 2` (the `sink`, then the
/// cut by hand) is what a Mac prints; Linux delivers them at once, and the same block printed
/// `gen 5` on CI's runner while the front door's printed `gen 2` in the same job (ledger
/// #1023). How many times a FILE thread was cut is a race, so it is not compared. Every
/// other thread's generation is: nothing but a command cuts those.
fn file_generation(line: String) -> String {
    let mut words = line.split_whitespace().rev();
    let (Some(n), Some("gen"), Some(name)) = (words.next(), words.next(), words.next()) else {
        return line;
    };
    if !name.starts_with("urn:file:") || !n.bytes().all(|b| b.is_ascii_digit()) {
        return line;
    }
    let cut = line.len() - line.trim_end().len() + n.len();
    format!("{}<n>", &line[..line.len() - cut])
}

/// Sort, on both sides, the rows of a `cache` readout that name the same resource.
///
/// `urn:kernel:cache` orders its rows by name (and resolution chain), and that is all it
/// orders by: two entries for one name — `toUpper`'s answer and its description, in the
/// front door — come out in the cache's hash order, which changes from one process to the
/// next (ledger #1023: the front door's block replayed green or red at random against one
/// binary). The order of a tie is not something the book can claim, so it is not compared;
/// the rows themselves still are, every column of each.
fn unorder_cache_ties(lines: Vec<String>) -> Vec<String> {
    fn name(line: &str) -> Option<&str> {
        let first = line.split_whitespace().next()?;
        (line.starts_with(' ') && first.starts_with("urn:")).then_some(first)
    }
    let mut out = Vec::with_capacity(lines.len());
    let mut in_cache = false;
    let mut run: Vec<String> = Vec::new();
    let flush = |run: &mut Vec<String>, out: &mut Vec<String>| {
        run.sort();
        out.append(run);
    };
    for line in lines {
        if in_cache && !line.starts_with(' ') {
            in_cache = false;
        }
        if !in_cache {
            flush(&mut run, &mut out);
            in_cache = line == "cache";
            out.push(line);
            continue;
        }
        if run
            .last()
            .and_then(|l| name(l))
            .is_some_and(|n| Some(n) != name(&line))
        {
            flush(&mut run, &mut out);
        }
        if name(&line).is_some() {
            run.push(line);
        } else {
            flush(&mut run, &mut out);
            out.push(line);
        }
    }
    flush(&mut run, &mut out);
    out
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
    for (n, t) in found.iter().enumerate() {
        if let Some(reason) = &t.manual {
            println!("skip {}:{} — manual: {reason}", t.file, t.line);
            continue;
        }
        let home = ScratchHome::new(n, &t.workspace);
        let screen = match type_into_repl(&home, &t.commands) {
            Ok(screen) => screen,
            Err(e) => panic!(
                "could not run `ikigai` ({e}); install it with `cargo install ikigai-cli --locked`"
            ),
        };
        let actual = normalize(screen.lines().map(str::to_string));
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
    fn a_file_threads_generation_is_not_compared_and_any_other_is() {
        assert_eq!(
            normalize(s(&["  urn:file:notes.txt  gen 2"])),
            normalize(s(&["  urn:file:notes.txt  gen 5"]))
        );
        assert_ne!(
            normalize(s(&["  urn:iki:fn:toUpper  gen 1"])),
            normalize(s(&["  urn:iki:fn:toUpper  gen 2"]))
        );
        assert_eq!(
            normalize(s(&["  urn:file:a  gen 12"])),
            s(&["  urn:file:a  gen <n>"])
        );
    }

    #[test]
    fn rows_of_one_name_in_a_cache_readout_compare_in_any_order() {
        let book = s(&[
            "cache",
            "  entries  3 / 4096 (2 stale)",
            "  urn:a  cut   text/plain  1 thread",
            "  urn:b  live  application/json  1 thread",
            "  urn:b  cut   text/plain  1 thread",
            "threads (cut generations)",
            "  urn:b  gen 1",
        ]);
        let mut cli = book.clone();
        cli.swap(3, 4);
        assert_eq!(normalize(book.clone()), normalize(cli));
        // Only a tie is unordered: rows of different names keep the CLI's order...
        let mut renamed = book.clone();
        renamed.swap(2, 3);
        assert_ne!(normalize(book.clone()), normalize(renamed));
        // ...a row's columns are still compared...
        let mut changed = book.clone();
        changed[4] = "  urn:b  live  text/plain  1 thread".into();
        assert_ne!(normalize(book.clone()), normalize(changed));
        // ...and nothing outside a `cache` readout is reordered.
        let lines = s(&["  urn:b  x", "  urn:b  a"]);
        assert_eq!(normalize(lines.clone()), lines);
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

    fn scan(text: &str) -> Vec<Transcript> {
        let mut out = Vec::new();
        scan_page("p.md", text, &mut out);
        out
    }

    #[test]
    fn a_program_runs_manual_marker_does_not_exempt_a_repl_block_after_it() {
        // Ledger #1075: the marker is within five lines of the REPL fence, but an earlier
        // fence sits between them, so it is the run's and the REPL block is replayed.
        let found = scan(
            "<!-- transcript: manual — the reader's own crate -->\n\
             ```bash\n\
             cargo run -p your-endpoints\n\
             ```\n\
             \n\
             ```text\n\
             ikigai> source urn:iki:fn:toUpper hello\n\
             HELLO\n\
             ```\n",
        );
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].manual, None, "the REPL block must be replayed");
        assert_eq!(found[0].commands, s(&["source urn:iki:fn:toUpper hello"]));
    }

    #[test]
    fn a_manual_reason_over_several_lines_is_kept_whole() {
        let found = scan(
            "<!-- transcript: manual — the capability line prints this machine's\n\
             \x20    home directory -->\n\
             ```text\n\
             ikigai> source urn:kernel:capability\n\
             ```\n",
        );
        assert_eq!(
            found[0].manual.as_deref(),
            Some("the capability line prints this machine's home directory")
        );
    }

    #[test]
    fn continues_merges_only_when_it_is_the_blocks_own_declaration() {
        let found = scan(
            "```text\n\
             ikigai> a\n\
             ```\n\
             <!-- transcript: continues -->\n\
             ```text\n\
             ikigai> b\n\
             ```\n\
             \n\
             ```text\n\
             ikigai> c\n\
             ```\n",
        );
        assert_eq!(found.len(), 2, "{found:?}");
        assert_eq!(found[0].commands, s(&["a", "b"]));
        assert_eq!(
            found[1].commands,
            s(&["c"]),
            "the marker above `b` is not `c`'s"
        );
    }

    #[test]
    fn a_pages_workspace_declaration_reaches_every_transcript_on_it() {
        let found = scan(
            "<!-- transcripts: workspace crates/quakes/a.geojson, b.jq -->\n\
             ```text\n\
             ikigai> a\n\
             ```\n\
             \n\
             ```text\n\
             ikigai> b\n\
             ```\n",
        );
        assert_eq!(found.len(), 2, "{found:?}");
        for t in &found {
            assert_eq!(t.workspace, s(&["crates/quakes/a.geojson", "b.jq"]));
        }
        assert!(scan("```text\nikigai> a\n```\n")[0].workspace.is_empty());
    }

    #[test]
    fn every_declared_workspace_file_is_in_the_repository() {
        for t in transcripts() {
            for path in &t.workspace {
                let at = repo_root().join(path);
                assert!(
                    at.is_file() || at.is_dir(),
                    "{}: `transcripts: workspace` names {path}, which is not a file or a \
                     directory in the repository",
                    t.file
                );
            }
        }
    }

    #[test]
    fn a_workspace_directory_is_copied_whole_and_only_what_git_tracks() {
        let to = std::env::temp_dir().join(format!("book-transcripts-copy-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&to);
        std::fs::create_dir_all(&to).unwrap();
        copy_tracked("examples/rss/ws", &to);
        assert!(to.join("directory.opml").is_file());
        assert!(
            to.join("feeds/alice.example.com.xml").is_file(),
            "subdirectories keep their path"
        );
        assert!(
            !to.join("searches").exists(),
            "ws/searches/ is what save-searches.iki writes, ignored by git, never copied"
        );
        let _ = std::fs::remove_dir_all(&to);
    }

    #[test]
    fn the_scanner_finds_the_books_transcripts_and_their_commands() {
        let found = transcripts();
        assert!(found.len() >= 3, "{}", found.len());
        assert!(found.iter().all(|t| !t.commands.is_empty()));
    }
}
