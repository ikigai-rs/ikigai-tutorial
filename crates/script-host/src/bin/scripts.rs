//! "Scripts are resources", run end to end: publish a query at a name, fetch it without
//! running it, read its result twice, run an update for its effects, watch the read
//! recompute, and publish a second version.
//!
//! ```text
//! cargo run -p script-host          # every section
//! cargo run -p script-host -- 2     # only section 2 (every section still runs)
//! ```
//!
//! A read says whether the kernel answered it from its cache (`[cached]`) or had to compute
//! it (`[computed]`), asked of the kernel before the read.

use ikigai_core::{Capability, Kernel, Verb};
use script_host::show::{self, ask, line, say, section};
use script_host::{alice, kernel, read, version_of, CLOSE_ITEM, OPEN_ITEMS};

fn main() {
    show::from_args();
    let kernel = kernel();
    let alice = alice();

    section(
        1,
        "Publish a query at a name, and fetch it without running it",
    );
    // ANCHOR: publish
    let first = ask(
        &kernel,
        "alice",
        &alice,
        Verb::Sink,
        "urn:script:open-items",
        &[("language", "sparql"), ("content", OPEN_ITEMS)],
        "sink urn:script:open-items language=sparql   (the query, piped)",
    )
    .expect("alice may publish open-items");
    // ANCHOR_END: publish
    say("alice", "source urn:script:open-items");
    let (source, _) = read(&kernel, &alice, "urn:script:open-items", &[]);
    let source = source.expect("alice may read open-items");
    // The text comes back exactly as it went in: fetching a script never runs it.
    assert_eq!(source, OPEN_ITEMS);
    for text in source.lines().take(2) {
        line(text);
    }
    line(&format!(
        "…   ({} lines, as published)",
        source.lines().count()
    ));

    section(2, "Run it as a read");
    result(&kernel, &alice, &[]);
    result(&kernel, &alice, &[]);
    result(&kernel, &alice, &[("priority", "1")]);
    // What a write to the items graph would make stale: the host asks, under its own
    // authority, which cached answers hang from the store's graph-write thread.
    ask(
        &kernel,
        "host",
        &Capability::root(),
        Verb::Source,
        "urn:kernel:dependents",
        &[("thread", "urn:iki:store:graph-update")],
        "dependents urn:iki:store:graph-update",
    )
    .expect("the host may ask");

    section(3, "Run a script for its effects");
    ask(
        &kernel,
        "alice",
        &alice,
        Verb::Sink,
        "urn:script:close-item",
        &[("language", "sparql"), ("content", CLOSE_ITEM)],
        "sink urn:script:close-item language=sparql   (the update, piped)",
    )
    .expect("alice may publish close-item");
    // ANCHOR: run
    let run = ask(
        &kernel,
        "alice",
        &alice,
        Verb::Sink,
        "urn:script:close-item:runs",
        &[("number", "2")],
        "sink urn:script:close-item:runs number=2",
    )
    .expect("alice may run close-item");
    // ANCHOR_END: run
    let run = run.trim().to_string();
    ask(
        &kernel,
        "alice",
        &alice,
        Verb::Source,
        &run,
        &[],
        &format!("source {run}"),
    )
    .expect("a runner may read the record of a run");
    result(&kernel, &alice, &[]);

    section(4, "Publish a second version");
    let reordered = OPEN_ITEMS.replace("ORDER BY ?number", "ORDER BY DESC(?number)");
    let second = ask(
        &kernel,
        "alice",
        &alice,
        Verb::Sink,
        "urn:script:open-items",
        &[("language", "sparql"), ("content", &reordered)],
        "sink urn:script:open-items language=sparql   (the same query, ORDER BY DESC(?number))",
    )
    .expect("alice may republish open-items");
    result(&kernel, &alice, &[]);
    // The first version is still there, by its digest, and still says what it said.
    let old = version_of(&first);
    assert_ne!(old, version_of(&second));
    say("alice", &format!("source {old}"));
    let (old_source, _) = read(&kernel, &alice, old, &[]);
    let old_source = old_source.expect("an old version stays readable");
    assert_eq!(old_source, OPEN_ITEMS);
    line("…");
    line(old_source.lines().last().unwrap_or_default());
}

/// Read the result of `open-items` as TSV, and say how it was answered.
fn result(kernel: &Kernel, who: &Capability, args: &[(&str, &str)]) {
    let mut all = vec![("as", "text/tab-separated-values")];
    all.extend_from_slice(args);
    let shown: String = args.iter().map(|(k, v)| format!(" {k}={v}")).collect();
    say(
        "alice",
        &format!("source urn:script:open-items:result as=text/tab-separated-values{shown}"),
    );
    let (answer, cached) = read(kernel, who, "urn:script:open-items:result", &all);
    show::text(&answer.unwrap_or_else(|e| panic!("open-items: {e}")));
    line(if cached { "[cached]" } else { "[computed]" });
}
