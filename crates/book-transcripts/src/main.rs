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
//! Every page runs in a fresh directory, and every command is given that directory as its
//! working directory AND as `HOME`, with `XDG_CONFIG_HOME` under it — whatever the page says.
//! So a gonk a page starts holds its store, socket, certificates and grants in scratch, and an
//! `ikigai` a page runs reads a config home with nothing in it. That is not a convenience: the
//! machine this runs on may have a real gonk, whose store is one writer's and whose ledger is
//! somebody's work. The one thing a scratch home cannot redirect is a TCP port, which is why a
//! command naming 1060 is refused before it runs (`book_transcripts::refused`).
//!
//! `--path DIR` (repeatable) is put in front of `PATH`, so a page says `ikigai-gonk`, as a
//! reader types it, and the runner decides which build that means.

use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Child, Command, ExitCode, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use book_transcripts::{matches, normalize, pages, refused, scan_page, Block, Kind, ANY};

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
        let (n, mut failed) = run_page(&options, &blocks);
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

/// One page, one session: its blocks in order, in one scratch home. Returns the number of
/// commands run and the failures.
fn run_page(options: &Options, blocks: &[Block]) -> (usize, Vec<String>) {
    let mut failures = Vec::new();
    let mut ran = 0;
    let scratch = Scratch::new();
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
    (ran, failures)
}

fn report(at: &str, command: &str, expected: &[String], actual: &[String]) -> String {
    format!(
        "{at}\n    $ {command}\n    the page says:\n      {}\n    the command printed:\n      {}",
        expected.join("\n      "),
        actual.join("\n      ")
    )
}

/// A page's scratch directory: its working directory, its `HOME`, and its config home.
struct Scratch {
    dir: PathBuf,
}

impl Scratch {
    /// A fresh directory under the system temp directory, with a SHORT name: a page's server
    /// may put a Unix socket in it, and a socket path is capped at 104 bytes on macOS.
    fn new() -> Scratch {
        let base = std::env::temp_dir();
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        let dir = base.join(format!("bt{}-{}", std::process::id(), stamp % 1_000_000));
        std::fs::create_dir_all(dir.join(".config")).expect("a scratch directory");
        Scratch {
            dir: dir.canonicalize().expect("the scratch directory exists"),
        }
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
            .env("HOME", &self.dir)
            .env("XDG_CONFIG_HOME", self.dir.join(".config"))
            .env("PATH", path)
            .stdin(Stdio::null());
        command
    }

    /// Run one command to completion; its stdout then its stderr, as lines, normalized.
    fn run(&self, paths: &[PathBuf], text: &str) -> Result<Vec<String>, String> {
        let mut child = self
            .command(paths, text)
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
        // would leave the server running, holding its port, after the page is done.
        let mut child = scratch
            .command(paths, &format!("exec {}", command.text))
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
                    let _ = child.kill();
                    let _ = child.wait();
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

    fn stop(mut self) {
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
