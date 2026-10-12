//! The code behind "Your first script", the page before "Scripts are resources": the same
//! host, with a shelf of books in it instead of work items, and the fewest requests that show
//! each idea once.
//!
//! ```text
//! cargo run -p script-host -- hello    # a script that greets, then one with a parameter
//! cargo run -p script-host -- shelf    # a script over data: add a book, and the answer follows
//! cargo run -p script-host -- grant    # who may run it: refused, one grant line, answered
//! ```
//!
//! Each step builds a host of its own, so each runs alone. The fourth step, a sentence that
//! becomes a script, is `cargo run -p sentence-draft -- ask`, because it needs a model door.
//!
//! The scripts and the data are files beside this one (`src/first/`), so the page shows them
//! exactly as they are published.

use std::sync::Arc;

use ikigai_core::{Capability, Error, Kernel, Verb};
use ikigai_script::authority::{same_for_all, Ceiling};

use crate::show::{ask, line, say, text};
use crate::{kernel_over, read};

/// The named graph the shelf lives in.
pub const SHELF: &str = "urn:iki:tutorial:shelf";

/// The smallest script there is: a query that answers one row. A script names the data it
/// reads, even a hello world: the host derives who may publish and run it from the graphs it
/// names and runs it over exactly those, and refuses at publish a query that names none
/// (ledger #1278, decided: keep the `FROM`). So this one names an empty graph, and uses
/// nothing in it.
pub const HELLO: &str = include_str!("first/hello.rq");

/// The same greeting, with a parameter: `who`, a string, `world` unless the caller says.
pub const HELLO_WHO: &str = include_str!("first/hello-who.rq");

/// Three books on a shelf, written straight to the store: the host's own data.
pub const BOOKS: &str = include_str!("first/shelf.ru");

/// Every title on the shelf, A to Z.
pub const TITLES: &str = include_str!("first/titles.rq");

/// A fourth book.
pub const FOURTH: &str = include_str!("first/fourth.ru");

/// The grant to read the shelf.
pub const READ_SHELF: &str = "urn:cap:store:read:graph:urn:iki:tutorial:shelf";

/// The one grant line step three adds: run `titles`.
pub const RUN_TITLES: &str = "urn:cap:script:run:titles";

/// A host with nothing in its store: the script host in front of an empty store.
pub fn empty() -> Arc<Kernel> {
    kernel_over(same_for_all(Ceiling::unbounded()), Vec::new(), None)
}

/// A host whose store holds the shelf of three books.
pub fn shelf() -> Arc<Kernel> {
    kernel_over(same_for_all(Ceiling::unbounded()), Vec::new(), Some(BOOKS))
}

/// Alice may publish `titles`, and read the shelf it reads.
pub fn alice() -> Capability {
    person("alice", &["urn:cap:script:write:titles", READ_SHELF])
}

/// Bob may read the shelf, and, once he is given the one line [`RUN_TITLES`], run `titles`.
pub fn bob(may_run: bool) -> Capability {
    let mut scopes = vec![READ_SHELF];
    if may_run {
        scopes.push(RUN_TITLES);
    }
    person("bob", &scopes)
}

fn person(name: &str, scopes: &[&str]) -> Capability {
    Capability::scoped(scopes.iter().copied())
        .with_principal(&format!("urn:iki:tutorial:person:{name}"))
        .expect("a principal IRI")
}

/// Run one of the page's steps, by the word a reader types after `--`; `false` for any other
/// word.
pub fn run(step: &str) -> bool {
    match step {
        "hello" => hello(),
        "shelf" => over_data(),
        "grant" => grant(),
        _ => return false,
    }
    true
}

const CSV: (&str, &str) = ("as", "text/csv");

/// Publish a greeting and run it; publish it again with a parameter, and run it twice.
fn hello() {
    let kernel = empty();
    let you = Capability::root();
    publish(&kernel, "you", &you, "hello", HELLO, "hello.rq");
    result(&kernel, "you", &you, "hello", &[]).expect("hello runs");
    publish(&kernel, "you", &you, "hello", HELLO_WHO, "hello-who.rq");
    result(&kernel, "you", &you, "hello", &[]).expect("hello runs");
    result(&kernel, "you", &you, "hello", &[("who", "Brian")]).expect("hello runs");
}

/// A script over the shelf: read it twice, add a book, read it again.
fn over_data() {
    let kernel = shelf();
    let you = Capability::root();
    publish(&kernel, "you", &you, "titles", TITLES, "titles.rq");
    counted(&kernel, &you);
    counted(&kernel, &you);
    ask(
        &kernel,
        "you",
        &you,
        Verb::Sink,
        "urn:iki:store:graph-update",
        &[("graph", SHELF), ("content", FOURTH)],
        "sink urn:iki:store:graph-update graph=urn:iki:tutorial:shelf   (fourth.ru, piped)",
    )
    .expect("the host may write its own store");
    counted(&kernel, &you);
}

/// Alice publishes `titles`; Bob is refused; Bob's grant gains one line; Bob gets the answer.
fn grant() {
    let kernel = shelf();
    publish(&kernel, "alice", &alice(), "titles", TITLES, "titles.rq");
    // A refusal is part of what the page shows, so it is printed, not raised.
    let _ = result(&kernel, "bob", &bob(false), "titles", &[]);
    say(
        "host",
        &format!("(bob's grant gains one line: {RUN_TITLES})"),
    );
    result(&kernel, "bob", &bob(true), "titles", &[]).expect("bob may run titles now");
}

/// `who` publishes `body`, the file `file`, at `urn:script:{name}`.
fn publish(kernel: &Kernel, who: &str, cap: &Capability, name: &str, body: &str, file: &str) {
    ask(
        kernel,
        who,
        cap,
        Verb::Sink,
        &format!("urn:script:{name}"),
        &[("language", "sparql"), ("content", body)],
        &format!("sink urn:script:{name} language=sparql   ({file}, piped)"),
    )
    .unwrap_or_else(|e| panic!("{who} may publish {name}: {e}"));
}

/// `who` reads `urn:script:{name}:result` as CSV, with `args`.
fn result(
    kernel: &Kernel,
    who: &str,
    cap: &Capability,
    name: &str,
    args: &[(&str, &str)],
) -> Result<String, Error> {
    let mut all = vec![CSV];
    all.extend_from_slice(args);
    let shown: String = args.iter().map(|(k, v)| format!(" {k}={v}")).collect();
    ask(
        kernel,
        who,
        cap,
        Verb::Source,
        &format!("urn:script:{name}:result"),
        &all,
        &format!("source urn:script:{name}:result as=text/csv{shown}"),
    )
}

/// Read `titles`, and say whether anything ran to answer it (asked of the kernel before the
/// read, as "Scripts are resources" does).
fn counted(kernel: &Kernel, you: &Capability) {
    say("you", "source urn:script:titles:result as=text/csv");
    let (answer, cached) = read(kernel, you, "urn:script:titles:result", &[CSV]);
    text(&answer.unwrap_or_else(|e| panic!("titles: {e}")));
    line(if cached {
        "(from the cache: nothing ran)"
    } else {
        "(computed)"
    });
}
