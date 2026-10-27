//! The runner itself, run the way CI runs it, over books written here for the purpose.
//!
//! The scanner and the matcher are unit-tested in `src/`; what only a run can show is that a
//! stale transcript FAILS the run (and says what the page claimed beside what the command
//! printed), that a normalizer lets through exactly its shape and nothing else, that what did not
//! run is counted, and that a server started on port 0 is reached by the next command at the port
//! it chose. Each book is a scratch directory with one page; the commands are `printf`, `date`,
//! `python3` and `curl`, so this needs no ikigai binary and runs in every `cargo test`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A one-page book in a fresh directory, removed when dropped.
struct Book {
    dir: PathBuf,
}

impl Book {
    fn new(name: &str, page: &str) -> Book {
        let dir = std::env::temp_dir().join(format!("bt-runner-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).expect("a book directory");
        std::fs::write(dir.join("src/page.md"), page).expect("the page");
        Book { dir }
    }

    fn run(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_book-transcripts"))
            .arg(&self.dir)
            .arg("--strict")
            .output()
            .expect("the runner starts")
    }
}

impl Drop for Book {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn page(blocks: &[&str]) -> String {
    blocks.join("\n\n")
}

fn console(declaration: &str, lines: &[&str]) -> String {
    format!(
        "<!-- transcript: {declaration} -->\n```console\n{}\n```\n",
        lines.join("\n")
    )
}

/// The reproduction this crate exists for: the page recorded output the command no longer
/// prints, and the run fails, naming the page, the command, the claim and the truth.
#[test]
fn a_stale_transcript_fails_the_run_and_says_what_changed() {
    let book = Book::new(
        "stale",
        &page(&[
            "# A page",
            &console(
                "run",
                &["$ printf 'hello, operator\\n'", "goodbye, operator"],
            ),
        ]),
    );
    let out = book.run();
    let stderr = text(&out.stderr);
    assert!(
        !out.status.success(),
        "a stale transcript must fail the run"
    );
    assert!(stderr.contains("1 transcript failure(s)"), "{stderr}");
    assert!(stderr.contains("page.md:"), "{stderr}");
    assert!(
        stderr.contains("the page says:\n      goodbye, operator"),
        "{stderr}"
    );
    assert!(
        stderr.contains("the command printed:\n      hello, operator"),
        "{stderr}"
    );

    // And the same page, brought up to date, passes.
    let book = Book::new(
        "current",
        &page(&[
            "# A page",
            &console("run", &["$ printf 'hello, operator\\n'", "hello, operator"]),
        ]),
    );
    let out = book.run();
    assert!(out.status.success(), "{}", text(&out.stderr));
}

#[test]
fn a_normalizer_lets_its_shape_through_and_nothing_else() {
    let now = &[
        "$ date -u +%Y-%m-%dT%H:%M:%SZ",
        // When this page was written: never the instant the check runs.
        "2026-10-27T18:04:05Z",
    ];
    // Without the normalizer the instant is a claim, and a false one.
    let out = Book::new("ts-off", &console("run", now)).run();
    assert!(!out.status.success(), "{}", text(&out.stdout));

    // With it, any instant is the page's instant...
    let out = Book::new("ts-on", &console("run; normalize timestamp", now)).run();
    assert!(out.status.success(), "{}", text(&out.stderr));

    // ...and the words around it are still compared exactly.
    let out = Book::new(
        "ts-words",
        &console(
            "run; normalize timestamp",
            &[
                "$ date -u +'started %Y-%m-%dT%H:%M:%SZ'",
                "stopped 2026-10-27T18:04:05Z",
            ],
        ),
    )
    .run();
    let stderr = text(&out.stderr);
    assert!(!out.status.success(), "{stderr}");
    assert!(
        stderr.contains("the page says (compared through: timestamp):"),
        "{stderr}"
    );

    // A whole number of seconds is a setting, not a measurement: `duration` leaves it alone.
    let out = Book::new(
        "dur",
        &console(
            "run; normalize duration",
            &[
                "$ printf 'timeout 10s, took 12ms\\n'",
                "timeout 30s, took 3ms",
            ],
        ),
    )
    .run();
    assert!(!out.status.success(), "{}", text(&out.stdout));
}

#[test]
fn an_unknown_normalizer_fails_the_block_naming_the_known_ones() {
    let out = Book::new(
        "unknown",
        &console("run; normalize timestamps", &["$ printf 'x\\n'", "x"]),
    )
    .run();
    let stderr = text(&out.stderr);
    assert!(!out.status.success());
    assert!(
        stderr.contains("no normalizer is called `timestamps`"),
        "{stderr}"
    );
}

#[test]
fn what_did_not_run_is_counted_with_its_reason() {
    let out = Book::new(
        "tally",
        &page(&[
            &console(
                "run",
                &["$ printf 'one\\n'", "one", "$ printf 'two\\n'", "two"],
            ),
            &console(
                "manual — it needs a person at a terminal",
                &["$ ikigai", "ikigai> "],
            ),
            "<!-- transcript: file notes.txt -->\n```text\nremember the milk\n```\n",
            &console("run", &["$ cat notes.txt", "remember the milk"]),
        ]),
    )
    .run();
    let stdout = text(&out.stdout);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(
        stdout.contains(
            "2 block(s) run (3 command(s), 0 server(s) started), 1 file block(s) written, \
             1 block(s) not run (manual): 33% of 3 transcript block(s) unrun"
        ),
        "{stdout}"
    );
    assert!(
        stdout.contains("not run: page.md:") && stdout.contains("it needs a person at a terminal"),
        "{stdout}"
    );
}

/// `python3` and `curl`, which the serving test drives, or a plain failure saying so (never a
/// silent skip: a test that passes because it did not run is the failure this crate exists for).
fn have(tool: &str) {
    let found = Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {tool}"))
        .output()
        .is_ok_and(|o| o.status.success());
    assert!(found, "this test drives `{tool}`, which is not on PATH");
}

/// The serving fixture: a server binds port 0 and prints the port it chose; the page shows the
/// port it chose when the page was written; the next command, written with that port, reaches
/// this run's server; and the page may not claim a second port for the same server.
#[test]
fn a_server_on_port_zero_is_reached_at_the_port_it_chose() {
    have("python3");
    have("curl");
    let serve = console(
        "serve; normalize port",
        &[
            "$ python3 -u -m http.server 0 --bind 127.0.0.1",
            "Serving HTTP on 127.0.0.1 port … (http://127.0.0.1:51234/) ...",
        ],
    );
    let reach = console(
        "run; normalize port",
        &[
            "$ curl -s -o /dev/null -w '%{http_code} %{url_effective}\\n' http://127.0.0.1:51234/",
            "200 http://127.0.0.1:51234/",
        ],
    );
    let out = Book::new("serve", &page(&[&serve, &reach])).run();
    assert!(
        out.status.success(),
        "{}\n{}",
        text(&out.stdout),
        text(&out.stderr)
    );
    assert!(text(&out.stdout).contains("1 server(s) started"));

    // The same server, claimed at a second port: refused, not normalized away.
    let elsewhere = console(
        "run; normalize port",
        &[
            "$ curl -s -o /dev/null -w '%{url_effective}\\n' http://127.0.0.1:51234/",
            "http://127.0.0.1:59999/",
        ],
    );
    let out = Book::new("serve-twice", &page(&[&serve, &elsewhere])).run();
    let stderr = text(&out.stderr);
    assert!(!out.status.success(), "{stderr}");
    assert!(
        stderr.contains("the page shows two ports, 51234 and 59999"),
        "{stderr}"
    );
}

#[test]
fn the_runner_refuses_a_book_with_nothing_to_run_under_strict() {
    let book = Book::new("empty", "# Nothing here\n");
    let out = book.run();
    assert!(!out.status.success());
    assert!(Path::new(&book.dir).exists());
}
