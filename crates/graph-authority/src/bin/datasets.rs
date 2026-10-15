//! "A query's authority is its dataset", as answers and refusals a reader can run.
//!
//! ```text
//! cargo run -p graph-authority          # every section
//! cargo run -p graph-authority -- 4     # only section 4 (every section still runs)
//! ```
//!
//! Each line that starts `north>`, `south>`, `auditor>`, `feed>`, `job>` or `host>` is one
//! request under that capability, written the way the REPL grammar would write it; what follows
//! is the answer, or `error:` and the refusal, word for word.

use ikigai_core::{Capability, Kernel, Verb};
use script_host::show::{self, line, say, section, text};

use graph_authority::{
    auditor, feed, feed_writes, job, kernel, north, shared_kernel, south, COSTED_ORDERS,
    COSTED_SCRIPT, COUNTS, ESCAPES, NORTH, SOUTH, WRITES,
};

const TSV: (&str, &str) = ("as", "text/tab-separated-values");

fn main() {
    show::from_args();
    let kernel = kernel();
    let (north, south, auditor, host) = (north(), south(), auditor(), Capability::root());
    let both = format!("{NORTH} {SOUTH}");

    section(1, "Which graphs each caller may read");
    let mut datasets = Vec::new();
    for (who, cap) in [("north", &north), ("south", &south), ("auditor", &auditor)] {
        let listed = ask(&kernel, who, cap, Verb::Source, "urn:iki:store:graphs", &[]);
        let graphs: Vec<String> = listed.lines().map(str::to_string).collect();
        datasets.push((who, cap, graphs.join(" ")));
    }

    section(2, "One text, three datasets");
    for (who, cap, dataset) in &datasets {
        select(&kernel, who, cap, dataset, COSTED_ORDERS, "the query above");
    }

    section(3, "Refuse, never narrow");
    select(
        &kernel,
        "north",
        &north,
        &both,
        COSTED_ORDERS,
        "the query above",
    );
    select(
        &kernel,
        "south",
        &south,
        &both,
        COSTED_ORDERS,
        "the query above",
    );

    section(4, "Confinement by construction");
    for (label, query) in ESCAPES {
        select(&kernel, "north", &north, NORTH, query, label);
    }

    section(5, "The write side");
    // A store of its own, so that what this section writes (and drops) is not what the
    // sections after it read.
    let writes = graph_authority::kernel();
    for (label, update) in WRITES {
        sink(&writes, "north", &north, NORTH, update, label);
    }
    let feed = feed();
    sink(
        &writes,
        "feed",
        &feed,
        SOUTH,
        "PREFIX schema: <https://schema.org/>\n\
         INSERT DATA { GRAPH <urn:iki:tutorial:graph:south> {\n  \
         <https://south.example/part/hinge> schema:price 3.95 } }",
        "a new price",
    );
    sink(
        &writes,
        "feed",
        &feed,
        SOUTH,
        "PREFIX schema: <https://schema.org/>\n\
         DELETE WHERE { GRAPH <urn:iki:tutorial:graph:south> { ?part schema:price ?price } }",
        "DELETE WHERE: every price",
    );
    sink(
        &writes,
        "job",
        &job(),
        SOUTH,
        "CLEAR GRAPH <urn:iki:tutorial:graph:south>",
        "CLEAR GRAPH, under the broad write key",
    );
    select(
        &writes,
        "auditor",
        &auditor,
        &both,
        COUNTS,
        "triples per graph",
    );

    section(6, "Authority from the text");
    ask(
        &kernel,
        "host",
        &host,
        Verb::Sink,
        "urn:script:costed-orders",
        &[("language", "sparql"), ("content", COSTED_SCRIPT)],
    );
    for (who, cap) in [("north", &north), ("south", &south), ("auditor", &auditor)] {
        offered(&kernel, who, cap, "urn:script:costed-orders");
    }
    for (who, cap) in [("north", &north), ("auditor", &auditor)] {
        ask(
            &kernel,
            who,
            cap,
            Verb::Source,
            "urn:script:costed-orders:result",
            &[TSV],
        );
    }

    section(7, "The cache keeps tenants apart");
    let fresh = graph_authority::kernel();
    cached(&fresh, "north", &north, NORTH);
    cached(&fresh, "north", &north, NORTH);
    cached(&fresh, "auditor", &auditor, NORTH);
    let (shared, handle) = shared_kernel();
    line("\n(a store whose price feed writes the supplier's graph through its own handle)");
    cached(&shared, "north", &north, NORTH);
    cached(&shared, "north", &north, NORTH);
    cached(&shared, "auditor", &auditor, &both);
    cached(&shared, "auditor", &auditor, &both);
    say("feed", "(loads the washer's price straight into the store)");
    feed_writes(
        &handle,
        "<https://south.example/part/washer> <https://schema.org/price> \"0.05\"^^<http://www.w3.org/2001/XMLSchema#decimal> <urn:iki:tutorial:graph:south> .\n",
    );
    cached(&shared, "auditor", &auditor, &both);

    section(8, "Fair shares of the machine");
    let one_row = [
        ("graph", NORTH),
        ("query", COSTED_ORDERS),
        ("max_rows", "1"),
        TSV,
    ];
    ask_as(
        &kernel,
        "north",
        &north,
        Verb::Source,
        "urn:iki:store:graph-select",
        &one_row,
        &format!(
            "source urn:iki:store:graph-select graph={NORTH} max_rows=1 as=text/tab-separated-values   (the query above)"
        ),
    );
    let wide = wide_join(33);
    ask_as(
        &kernel,
        "north",
        &north,
        Verb::Source,
        "urn:iki:store:graph-select",
        &[("graph", NORTH), ("query", &wide), TSV],
        &format!(
            "source urn:iki:store:graph-select graph={NORTH} as=text/tab-separated-values   (a join of 33 triple patterns)"
        ),
    );
}

/// A query joining `n` triple patterns, one more than the store plans.
fn wide_join(n: usize) -> String {
    let patterns: Vec<String> = (0..n)
        .map(|i| format!("?order <https://schema.org/orderQuantity> ?q{i} ."))
        .collect();
    format!("SELECT ?order WHERE {{ {} }}", patterns.join(" "))
}

/// One request, printed as `who> source|sink <iri> args…`, then its answer or refusal. The
/// answer comes back, or the empty string after a refusal.
fn ask(
    kernel: &Kernel,
    who: &str,
    cap: &Capability,
    verb: Verb,
    iri: &str,
    args: &[(&str, &str)],
) -> String {
    let shown: Vec<String> = args
        .iter()
        .filter(|(name, _)| *name != "content")
        .map(|(name, value)| format!("{name}={value}"))
        .collect();
    let mut command = format!(
        "{} {iri}",
        if verb == Verb::Sink { "sink" } else { "source" }
    );
    if !shown.is_empty() {
        command = format!("{command} {}", shown.join(" "));
    }
    if args.iter().any(|(name, _)| *name == "content") {
        command = format!("{command}   (the text above)");
    }
    ask_as(kernel, who, cap, verb, iri, args, &command)
}

fn ask_as(
    kernel: &Kernel,
    who: &str,
    cap: &Capability,
    verb: Verb,
    iri: &str,
    args: &[(&str, &str)],
    command: &str,
) -> String {
    show::ask(kernel, who, cap, verb, iri, args, command).unwrap_or_default()
}

/// A scoped SELECT over `graphs`, as tab-separated values.
fn select(kernel: &Kernel, who: &str, cap: &Capability, graphs: &str, query: &str, label: &str) {
    let graph = if graphs.contains(' ') {
        format!("\"{graphs}\"")
    } else {
        graphs.to_string()
    };
    ask_as(
        kernel,
        who,
        cap,
        Verb::Source,
        "urn:iki:store:graph-select",
        &[("graph", graphs), ("query", query), TSV],
        &format!(
            "source urn:iki:store:graph-select graph={graph} as=text/tab-separated-values   ({label})"
        ),
    );
}

/// A scoped update of `graph`.
fn sink(kernel: &Kernel, who: &str, cap: &Capability, graph: &str, update: &str, label: &str) {
    ask_as(
        kernel,
        who,
        cap,
        Verb::Sink,
        "urn:iki:store:graph-update",
        &[("graph", graph), ("content", update)],
        &format!("sink urn:iki:store:graph-update graph={graph}   ({label})"),
    );
}

/// The caller's action manifold, cut down to the lines that name `prefix`.
fn offered(kernel: &Kernel, who: &str, cap: &Capability, prefix: &str) {
    say(
        who,
        &format!("source urn:kernel:actions   (the lines naming {prefix})"),
    );
    match script_host::issue(kernel, cap, Verb::Source, "urn:kernel:actions", &[]) {
        Ok(body) => {
            let lines: Vec<&str> = body.lines().filter(|l| l.contains(prefix)).collect();
            if lines.is_empty() {
                line("(none)");
            } else {
                text(&lines.join("\n"));
            }
        }
        Err(e) => line(&format!("error: {e}")),
    }
}

/// [`COSTED_ORDERS`] over `graphs`, saying whether the kernel served it from its cache. The
/// answer is summarized (how many orders, how many of them priced), because section 2 printed
/// it in full and what this section is about is the line after it.
fn cached(kernel: &Kernel, who: &str, cap: &Capability, graphs: &str) {
    let args = [("graph", graphs), ("query", COSTED_ORDERS), TSV];
    let graph = if graphs.contains(' ') {
        format!("\"{graphs}\"")
    } else {
        graphs.to_string()
    };
    say(
        who,
        &format!(
            "source urn:iki:store:graph-select graph={graph} as=text/tab-separated-values   (the query above)"
        ),
    );
    let (answer, hit) = script_host::read(kernel, cap, "urn:iki:store:graph-select", &args);
    let verdict = if hit {
        "served from the cache"
    } else {
        "computed"
    };
    match answer {
        Ok(body) => {
            let rows: Vec<&str> = body.lines().skip(1).collect();
            let priced = rows
                .iter()
                .filter(|row| row.rsplit('\t').next().is_some_and(|p| !p.is_empty()))
                .count();
            line(&format!(
                "{} orders, {priced} priced   ({verdict})",
                rows.len()
            ));
        }
        Err(e) => line(&format!("error: {e}")),
    }
}
