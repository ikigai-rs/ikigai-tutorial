//! "Who a script runs as", as refusals and answers a reader can run.
//!
//! ```text
//! cargo run -p script-authority          # every section
//! cargo run -p script-authority -- 4     # only section 4 (every section still runs)
//! ```
//!
//! Each line that starts `alice>`, `bob>`, `carol>` or `host>` is one request under that
//! capability, written the way the REPL grammar would write it; what follows is the answer,
//! or `error:` and the refusal, word for word.

use ikigai_core::{Capability, Kernel, Verb};
use script_authority::{alice, bob, carol, kernel, Ceilings, GRAPHS, SALARIES};
use script_host::show::{self, say, section};
use script_host::OPEN_ITEMS;

fn main() {
    show::from_args();
    let ceilings = Ceilings::new();
    let kernel = kernel(&ceilings);
    let (alice, bob, carol, host) = (alice(), bob(), carol(), Capability::root());
    let tsv = ("as", "text/tab-separated-values");

    section(1, "A publish is checked against its publisher");
    ask(
        &kernel,
        "carol",
        &carol,
        Verb::Sink,
        "urn:script:salaries",
        &[("language", "sparql"), ("content", SALARIES)],
        "sink urn:script:salaries language=sparql   (a query over the payroll graph)",
    );
    ask(&kernel, "alice", &alice, Verb::Sink, "urn:script:open-items",
        &[("language", "sparql"), ("content", OPEN_ITEMS), ("requires", "urn:cap:store:read:graph:urn:iki:tutorial:payroll")],
        "sink urn:script:open-items language=sparql requires=urn:cap:store:read:graph:urn:iki:tutorial:payroll");
    ask(
        &kernel,
        "alice",
        &alice,
        Verb::Sink,
        "urn:script:open-items",
        &[("language", "sparql"), ("content", OPEN_ITEMS)],
        "sink urn:script:open-items language=sparql",
    );

    section(2, "A run is checked against its runner");
    ask(
        &kernel,
        "alice",
        &alice,
        Verb::Source,
        "urn:script:open-items:result",
        &[tsv],
        "source urn:script:open-items:result as=text/tab-separated-values",
    );
    ask(
        &kernel,
        "carol",
        &carol,
        Verb::Source,
        "urn:script:open-items:result",
        &[tsv],
        "source urn:script:open-items:result as=text/tab-separated-values",
    );
    ask(
        &kernel,
        "bob",
        &bob,
        Verb::Source,
        "urn:script:open-items:result",
        &[tsv],
        "source urn:script:open-items:result as=text/tab-separated-values",
    );

    section(3, "A run never holds more than its publisher held");
    ask(
        &kernel,
        "alice",
        &alice,
        Verb::Sink,
        "urn:script:graphs",
        &[("language", "sparql"), ("content", GRAPHS)],
        "sink urn:script:graphs language=sparql   (GRAPH ?g: every graph the run may read)",
    );
    ask(
        &kernel,
        "bob",
        &bob,
        Verb::Source,
        "urn:script:graphs:result",
        &[tsv],
        "source urn:script:graphs:result as=text/tab-separated-values",
    );
    ask(
        &kernel,
        "bob",
        &bob,
        Verb::Sink,
        "urn:script:graphs:runs",
        &[],
        "sink urn:script:graphs:runs   (the same query, run for the record)",
    );
    ask(
        &kernel,
        "bob",
        &bob,
        Verb::Source,
        "urn:script:graphs:run:1",
        &[],
        "source urn:script:graphs:run:1",
    );

    section(4, "Nor more than the host's ceiling allows");
    ask(
        &kernel,
        "host",
        &host,
        Verb::Sink,
        "urn:script:every-graph",
        &[("language", "sparql"), ("content", GRAPHS)],
        "sink urn:script:every-graph language=sparql   (the same query, published by the host)",
    );
    ask(
        &kernel,
        "bob",
        &bob,
        Verb::Source,
        "urn:script:every-graph:result",
        &[tsv],
        "source urn:script:every-graph:result as=text/tab-separated-values",
    );
    // ANCHOR: narrow
    say(
        "host",
        "(writes script-authority/every-graph: the items graph alone)",
    );
    ceilings.set(
        "every-graph",
        "urn:cap:store:read:graph:urn:iki:tutorial:items\n",
    );
    // ANCHOR_END: narrow
    ask(
        &kernel,
        "bob",
        &bob,
        Verb::Source,
        "urn:script:every-graph:result",
        &[tsv],
        "source urn:script:every-graph:result as=text/tab-separated-values",
    );
    // ANCHOR: cut
    ask(
        &kernel,
        "host",
        &host,
        Verb::Sink,
        "urn:kernel:cut",
        &[("thread", "urn:script:every-graph:authority")],
        "sink urn:kernel:cut thread=urn:script:every-graph:authority",
    );
    // ANCHOR_END: cut
    ask(
        &kernel,
        "bob",
        &bob,
        Verb::Source,
        "urn:script:every-graph:result",
        &[tsv],
        "source urn:script:every-graph:result as=text/tab-separated-values",
    );
    ask(
        &kernel,
        "host",
        &host,
        Verb::Sink,
        "urn:script:orphan",
        &[("language", "sparql"), ("content", GRAPHS)],
        "sink urn:script:orphan language=sparql   (no ceiling file names it)",
    );
    ask(
        &kernel,
        "bob",
        &bob,
        Verb::Source,
        "urn:script:orphan:result",
        &[tsv],
        "source urn:script:orphan:result as=text/tab-separated-values",
    );

    section(5, "A namespace grant, and an exclusion that wins over it");
    for name in ["team-open", "team-a-open", "teammate", "team-payroll"] {
        ask(
            &kernel,
            "alice",
            &alice,
            Verb::Sink,
            &format!("urn:script:{name}"),
            &[("language", "sparql"), ("content", OPEN_ITEMS)],
            &format!("sink urn:script:{name} language=sparql"),
        );
    }
    ask(
        &kernel,
        "bob",
        &bob,
        Verb::Source,
        "urn:script:team-a-open:result",
        &[tsv, ("priority", "1")],
        "source urn:script:team-a-open:result as=text/tab-separated-values priority=1",
    );
}

/// One request under `capability`, printed as `who> command`, then its answer or refusal.
fn ask(
    kernel: &Kernel,
    who: &str,
    capability: &Capability,
    verb: Verb,
    iri: &str,
    args: &[(&str, &str)],
    command: &str,
) {
    // A refusal is part of what the program shows, so it is printed, not raised.
    let _ = show::ask(kernel, who, capability, verb, iri, args, command);
}
