//! Run every shell transcript in a book, and say which ones the page gets wrong.
//!
//!     cargo run -p book-transcripts -- books/gonk --strict --path <dir with ikigai-gonk>
//!     cargo run -p book-transcripts -- books/gonk --list      # what would run, run nothing
//!
//! `scripts/test-transcripts.sh` is the usual way in: it installs gonk at the rev the book
//! pins (`books/gonk/gonk.rev`) and passes its directory as `--path`. See the crate docs
//! (`src/lib.rs`) for the block declarations and what "matches" means.
//!
//! ## The scratch home, and why it is not optional
//!
//! Every page runs in a fresh directory, its commands' working directory. `HOME` and
//! `XDG_CONFIG_HOME` are NEVER the reader's: by default they point into a second, empty TRAP
//! directory, and a page that leaves a file there fails, naming it — a command that forgot its
//! `--config-home` or `--data-home` writes exactly there, and on a reader's machine it would
//! have written into their real homes. A page that declares
//! `<!-- transcripts: home — why -->` gets its own directory as `HOME` instead (the crate docs,
//! `src/lib.rs`, say more). Either way a gonk a page starts holds its store, socket,
//! certificates and grants in scratch, and an `ikigai` a page runs reads a config home with
//! nothing in it unless the page put something there. That is not a convenience: the machine
//! this runs on may have a real gonk, whose store is one writer's and whose ledger is
//! somebody's work. The one thing a scratch home cannot redirect is a TCP port, which is why a
//! command naming 1060 is refused before it runs (`book_transcripts::refused`).
//!
//! `--path DIR` (repeatable) is put in front of `PATH`, so a page says `ikigai-gonk`, as a
//! reader types it, and the runner decides which build that means.

use std::io::{BufRead, BufReader, Read};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, ExitCode, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use book_transcripts::{
    home_declared, matches, normalize, pages, refused, safe_file_name, scan_page, stray_writes,
    Block, Kind, ANY,
};

/// How long one `run` command may take before it is a failure.
const RUN_PATIENCE: Duration = Duration::from_secs(60);
/// How long a `serve` command has to print what the page says it prints at startup.
const SERVE_PATIENCE: Duration = Duration::from_secs(60);

struct Options {
    book: PathBuf,
    paths: Vec<PathBuf>,
    strict: bool,
    list: bool,
}

fn options() -> Result<Options, String> {
    let mut args = std::env::args().skip(1);
    let mut book = None;
    let mut paths = Vec::new();
    let mut strict = false;
    let mut list = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--strict" => strict = true,
            "--list" => list = true,
            "--path" => {
                let dir = args.next().ok_or("--path needs a directory")?;
                let dir = PathBuf::from(dir)
                    .canonicalize()
                    .map_err(|e| format!("--path: {e}"))?;
                paths.push(dir);
            }
            flag if flag.starts_with("--") => return Err(format!("unknown flag {flag}")),
            _ if book.is_none() => book = Some(PathBuf::from(arg)),
            _ => return Err("one book at a time".to_string()),
        }
    }
    Ok(Options {
        book: book.ok_or("usage: book-transcripts <book dir> [--strict] [--list] [--path DIR]…")?,
        paths,
        strict,
        list,
    })
}

fn main() -> ExitCode {
    let options = match options() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("book-transcripts: {e}");
            return ExitCode::from(2);
        }
    };
    let src = options.book.join("src");
    let files = match pages(&src) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("book-transcripts: {}: {e}", src.display());
            return ExitCode::from(2);
        }
    };

    let mut failures: Vec<String> = Vec::new();
    let mut ran = 0usize;
    for path in files {
        let file = path
            .strip_prefix(&src)
            .expect("under src")
            .to_string_lossy()
            .into_owned();
        let text = std::fs::read_to_string(&path).expect("a readable page");
        let blocks = scan_page(&file, &text);
        if blocks.is_empty() {
            continue;
        }
        if options.list {
            for b in &blocks {
                println!("{}:{} {:?}", b.file, b.line, b.kind);
                for c in &b.commands {
                    println!("    $ {}", c.text);
                }
            }
            continue;
        }
        let (n, mut failed) = run_page(&options, &blocks, home_declared(&text));
        ran += n;
        failures.append(&mut failed);
    }

    if options.list {
        return ExitCode::SUCCESS;
    }
    if ran == 0 && failures.is_empty() && options.strict {
        eprintln!(
            "book-transcripts: nothing ran in {} — under --strict a book with no transcript \
             is a scanner that is not reading it",
            options.book.display()
        );
        return ExitCode::FAILURE;
    }
    if failures.is_empty() {
        println!("{ran} command(s) replayed; every transcript says what its command prints");
        return ExitCode::SUCCESS;
    }
    eprintln!(
        "\n{} transcript failure(s):\n\n{}\n\nFix the page to say what the command prints — or, \
         if it genuinely cannot run here, declare it above the fence: \
         <!-- transcript: manual — why -->",
        failures.len(),
        failures.join("\n\n")
    );
    ExitCode::FAILURE
}

/// One page, one session: its blocks in order, in one scratch directory. `home` is the
/// page's `transcripts: home` reason, if it declared one. Returns the number of commands run
/// and the failures.
fn run_page(options: &Options, blocks: &[Block], home: Option<String>) -> (usize, Vec<String>) {
    let mut failures = Vec::new();
    let mut ran = 0;
    let page = blocks.first().map(|b| b.file.as_str()).unwrap_or("?");
    if home.as_deref() == Some("") {
        failures.push(format!(
            "{page}: declares `transcripts: home` without a reason. Say what reads `~` and why \
             no flag can name it: <!-- transcripts: home — why -->"
        ));
        return (ran, failures);
    }
    let scratch = Scratch::new(home.is_some());
    let mut servers: Vec<Server> = Vec::new();

    for block in blocks {
        let at = format!("{}:{}", block.file, block.line);
        match &block.kind {
            Kind::Manual(reason) => {
                println!("skip {at} — manual: {reason}");
                continue;
            }
            Kind::Undeclared if options.strict => {
                failures.push(format!(
                    "{at}: a transcript with no declaration. Say how it is checked above the \
                     fence: <!-- transcript: run -->, <!-- transcript: serve -->, or \
                     <!-- transcript: manual — why -->"
                ));
                continue;
            }
            Kind::Undeclared => {
                println!("skip {at} — undeclared (not --strict)");
                continue;
            }
            Kind::File(name) => {
                match scratch.write(name, &block.content) {
                    Ok(()) => println!("file {at} → {name}"),
                    Err(e) => failures.push(format!("{at}: file {name}: {e}")),
                }
                continue;
            }
            Kind::Run | Kind::Serve => {}
        }
        if !block.orphans.is_empty() {
            failures.push(format!(
                "{at}: output before any command, so nothing printed it: {:?}",
                block.orphans
            ));
            continue;
        }
        if let Some(why) = block.commands.iter().find_map(|c| refused(&c.text)) {
            failures.push(format!("{at}: refused, not run — {why}"));
            continue;
        }
        if block.kind == Kind::Serve {
            if block.commands.len() != 1 {
                failures.push(format!(
                    "{at}: a serve block starts ONE command; this one has {}",
                    block.commands.len()
                ));
                continue;
            }
            let command = &block.commands[0];
            ran += 1;
            match Server::start(&scratch, &options.paths, command) {
                Ok(server) => {
                    println!("ok   {at} (serving: {})", command.text);
                    servers.push(server);
                }
                Err(e) => failures.push(format!("{at}\n    $ {}\n{e}", command.text)),
            }
            continue;
        }
        let mut block_ok = true;
        for command in &block.commands {
            ran += 1;
            let actual = scratch.run(&options.paths, &command.text);
            let expected = normalize(&command.expected);
            match actual {
                Ok(lines) if matches(&expected, &lines) => {}
                Ok(lines) => {
                    block_ok = false;
                    failures.push(report(&at, &command.text, &expected, &lines));
                }
                Err(e) => {
                    block_ok = false;
                    failures.push(format!("{at}\n    $ {}\n    {e}", command.text));
                }
            }
        }
        println!(
            "{} {at} ({} command{})",
            if block_ok { "ok  " } else { "MISS" },
            block.commands.len(),
            if block.commands.len() == 1 { "" } else { "s" }
        );
    }
    for server in servers {
        server.stop();
    }
    if let Some(trap) = &scratch.trap {
        let stray = stray_writes(trap);
        if !stray.is_empty() {
            failures.push(format!(
                "{page}: wrote into a HOME it did not declare: {}. A command here is missing \
                 its homes (`--config-home`, `--data-home`); run by a reader, it would write \
                 into theirs. If the page truly needs `HOME`, declare it once: \
                 <!-- transcripts: home — why -->",
                stray
                    .iter()
                    .map(|p| format!("~/{}", p.display()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
    (ran, failures)
}

fn report(at: &str, command: &str, expected: &[String], actual: &[String]) -> String {
    format!(
        "{at}\n    $ {command}\n    the page says:\n      {}\n    the command printed:\n      {}",
        expected.join("\n      "),
        actual.join("\n      ")
    )
}

/// A page's scratch directory (its working directory), and the `HOME` its commands get: the
/// directory itself when the page declared `transcripts: home`, else an empty trap beside it.
struct Scratch {
    dir: PathBuf,
    home: PathBuf,
    /// The trap, when `home` is one: anything left in it is a write the page did not declare.
    trap: Option<PathBuf>,
}

impl Scratch {
    /// A fresh directory under the system temp directory, with a SHORT name: a page's server
    /// may put a Unix socket in it, and a socket path is capped at 104 bytes on macOS.
    fn new(home_is_the_page: bool) -> Scratch {
        let base = std::env::temp_dir();
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        let name = format!("bt{}-{}", std::process::id(), stamp % 1_000_000);
        let dir = base.join(&name);
        std::fs::create_dir_all(dir.join(".config")).expect("a scratch directory");
        let dir = dir.canonicalize().expect("the scratch directory exists");
        if home_is_the_page {
            return Scratch {
                home: dir.clone(),
                dir,
                trap: None,
            };
        }
        // Empty, and NOT given a `.config`: the config home inside it is named, never made, so
        // whatever appears in the trap was written by a command.
        let trap = base.join(format!("{name}-home"));
        std::fs::create_dir_all(&trap).expect("a trap home");
        let trap = trap.canonicalize().expect("the trap exists");
        Scratch {
            dir,
            home: trap.clone(),
            trap: Some(trap),
        }
    }

    /// Write a `file` block: its lines, each ending in a newline, at `name` under the scratch
    /// directory, creating the directories on the way.
    fn write(&self, name: &str, lines: &[String]) -> Result<(), String> {
        if !safe_file_name(name) {
            return Err(
                "a file block's name must be relative and stay in the scratch \
                        directory (no `..`, no leading `/`)"
                    .to_string(),
            );
        }
        let path = self.dir.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut text = lines.join("\n");
        text.push('\n');
        std::fs::write(&path, text).map_err(|e| e.to_string())
    }

    fn command(&self, paths: &[PathBuf], text: &str) -> Command {
        let mut path = std::env::join_paths(paths).unwrap_or_default();
        if let Some(rest) = std::env::var_os("PATH") {
            let mut joined = paths.to_vec();
            joined.extend(std::env::split_paths(&rest));
            path = std::env::join_paths(joined).unwrap_or(path);
        }
        let mut command = Command::new("sh");
        command
            .arg("-c")
            .arg(text)
            .current_dir(&self.dir)
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", self.home.join(".config"))
            .env("PATH", path)
            .stdin(Stdio::null());
        command
    }

    /// Run one command to completion; what it printed, as lines, normalized.
    ///
    /// Standard error is sent to the SAME pipe as standard output (`exec 2>&1` before the
    /// command, so the command's own redirections still apply after it), so the lines come
    /// back in the order a terminal shows them. Read from two pipes, a command that writes
    /// results to stdout and status lines to stderr (`ikigai -c a -c b` does) came back with
    /// every status line at the end, which is not what a reader sees.
    fn run(&self, paths: &[PathBuf], text: &str) -> Result<Vec<String>, String> {
        let mut child = self
            .command(paths, &format!("exec 2>&1\n{text}"))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("could not start sh: {e}"))?;
        let stdout = drain(child.stdout.take().expect("piped"));
        let stderr = drain(child.stderr.take().expect("piped"));
        let deadline = Instant::now() + RUN_PATIENCE;
        // `try_wait` polls for the timeout only; the verdict is `wait`'s, after the pipes
        // close — stderr at EOF says the child is exiting, not that it has been reaped.
        while child.try_wait().map_err(|e| e.to_string())?.is_none() {
            if Instant::now() > deadline {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("still running after {}s", RUN_PATIENCE.as_secs()));
            }
            thread::sleep(Duration::from_millis(20));
        }
        child.wait().map_err(|e| e.to_string())?;
        let mut lines = stdout.join().expect("the stdout reader");
        lines.extend(stderr.join().expect("the stderr reader"));
        Ok(normalize(lines))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
        if let Some(trap) = &self.trap {
            let _ = std::fs::remove_dir_all(trap);
        }
    }
}

/// Read a pipe to its end on a thread, as lines.
fn drain(pipe: impl Read + Send + 'static) -> thread::JoinHandle<Vec<String>> {
    thread::spawn(move || BufReader::new(pipe).lines().map_while(Result::ok).collect())
}

/// A command a `serve` block started, running until the page is done.
struct Server {
    child: Child,
}

impl Server {
    /// Start it, and wait until what it has printed matches the page's lines (anything after
    /// them is allowed: a server goes on printing).
    fn start(
        scratch: &Scratch,
        paths: &[PathBuf],
        command: &book_transcripts::Command,
    ) -> Result<Server, String> {
        // `exec`, so the process this holds IS the server: killing a shell that forked it
        // would leave the server running, holding its port, after the page is done. And its
        // own process GROUP, which is what `stop` signals: a serve line that is a pipeline
        // (`ikigai-gonk … 2>&1 | tee gonk.log`, so a page can read the log back) is several
        // processes, and `exec` replaces only the first.
        let mut child = scratch
            .command(paths, &format!("exec {}", command.text))
            .process_group(0)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("    could not start sh: {e}"))?;
        let lines = lines_of(
            child.stdout.take().expect("piped"),
            child.stderr.take().expect("piped"),
        );
        let mut expected = normalize(&command.expected);
        expected.push(ANY.to_string());
        let mut seen: Vec<String> = Vec::new();
        let deadline = Instant::now() + SERVE_PATIENCE;
        loop {
            if matches(&expected, &normalize(&seen)) {
                return Ok(Server { child });
            }
            let left = deadline.saturating_duration_since(Instant::now());
            match lines.recv_timeout(left) {
                Ok(line) => seen.push(line),
                Err(_) => {
                    let exited = child.try_wait().ok().flatten();
                    // The whole GROUP, as `stop` does: with only `child.kill()`, a serve line
                    // that is a pipeline (`… | tee gonk.log`) left the server running after
                    // its banner failed to match, holding the port, and every later page's
                    // server failed to bind — one changed banner line read as a dozen pages
                    // broken (seen moving the pin to edb9a3d).
                    Server { child }.stop();
                    expected.pop();
                    return Err(format!(
                        "    {}\n    the page says it starts with:\n      {}\n    it printed:\n      {}",
                        match exited {
                            Some(status) => format!("it exited ({status})"),
                            None => format!("no match within {}s", SERVE_PATIENCE.as_secs()),
                        },
                        expected.join("\n      "),
                        normalize(&seen).join("\n      ")
                    ));
                }
            }
        }
    }

    /// Stop the server and everything in its process group, then reap it.
    fn stop(mut self) {
        let group = format!("-{}", self.child.id());
        let _ = Command::new("kill")
            .args(["-TERM", "--", &group])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if let Ok(Some(_)) = self.child.try_wait() {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
        let _ = Command::new("kill")
            .args(["-KILL", "--", &group])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Every line a server prints, from both pipes, as it prints them. The readers keep draining
/// after the banner has matched, so a server that logs every request never blocks on a full
/// pipe.
fn lines_of(
    stdout: impl Read + Send + 'static,
    stderr: impl Read + Send + 'static,
) -> Receiver<String> {
    let (tx, rx) = mpsc::channel();
    for pipe in [Box::new(stdout) as Box<dyn Read + Send>, Box::new(stderr)] {
        let tx = tx.clone();
        thread::spawn(move || {
            for line in BufReader::new(pipe).lines().map_while(Result::ok) {
                let _ = tx.send(line);
            }
        });
    }
    rx
}
