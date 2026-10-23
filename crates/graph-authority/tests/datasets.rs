//! What "A query's authority is its dataset" says, pinned against the real `ikigai-store`: the
//! TYPE of each refusal (the program prints its words, `book_runs.rs` checks them), and the
//! rows each permitted read returns beside it. Every refusal here is paired with a read that
//! returns rows, because a confinement that is wrong in the safe direction is otherwise
//! indistinguishable from an empty graph.

use ikigai_core::{Capability, Error, Kernel, Verb};
use script_host::{issue, read};

use graph_authority::{
    auditor, feed, feed_writes, job, kernel, north, shared_kernel, south, COSTED_ORDERS,
    COSTED_SCRIPT, ESCAPES, NORTH, SOUTH, WRITES,
};

fn select(kernel: &Kernel, who: &Capability, graphs: &str, query: &str) -> Result<String, Error> {
    issue(
        kernel,
        who,
        Verb::Source,
        "urn:iki:store:graph-select",
        &[
            ("graph", graphs),
            ("query", query),
            ("as", "text/tab-separated-values"),
        ],
    )
}

fn update(kernel: &Kernel, who: &Capability, graph: &str, text: &str) -> Result<String, Error> {
    issue(
        kernel,
        who,
        Verb::Sink,
        "urn:iki:store:graph-update",
        &[("graph", graph), ("content", text)],
    )
}

fn both() -> String {
    format!("{NORTH} {SOUTH}")
}

fn escape(label: &str) -> &'static str {
    ESCAPES.iter().find(|(l, _)| *l == label).expect(label).1
}

fn write(label: &str) -> &'static str {
    WRITES.iter().find(|(l, _)| *l == label).expect(label).1
}

#[test]
fn the_same_text_answers_each_caller_over_its_own_dataset() {
    let kernel = kernel();
    let mine = select(&kernel, &north(), NORTH, COSTED_ORDERS).unwrap();
    assert_eq!(mine.lines().count(), 4, "three orders and a header: {mine}");
    assert!(
        !mine.contains("0.25"),
        "no price is in north's graph: {mine}"
    );
    let theirs = select(&kernel, &south(), SOUTH, COSTED_ORDERS).unwrap();
    assert_eq!(
        theirs.lines().count(),
        1,
        "no order is in south's graph: {theirs}"
    );
    let joined = select(&kernel, &auditor(), &both(), COSTED_ORDERS).unwrap();
    assert!(
        joined.contains("0.25") && joined.contains("3.15"),
        "{joined}"
    );
}

#[test]
fn a_dataset_with_one_grant_missing_is_refused_naming_it_never_narrowed() {
    let kernel = kernel();
    match select(&kernel, &north(), &both(), COSTED_ORDERS) {
        Err(Error::Denied(m)) => assert!(
            m.contains("urn:cap:store:read:graph:urn:iki:tutorial:graph:south"),
            "{m}"
        ),
        other => panic!("a narrowed answer is the bug this refuses: {other:?}"),
    }
    // The pair: the same dataset, under a capability holding both grants, answers.
    assert!(select(&kernel, &auditor(), &both(), COSTED_ORDERS).is_ok());
}

#[test]
fn enumeration_lists_only_the_graphs_a_caller_may_read() {
    let kernel = kernel();
    let listed =
        |who: &Capability| issue(&kernel, who, Verb::Source, "urn:iki:store:graphs", &[]).unwrap();
    assert_eq!(listed(&north()).trim(), NORTH);
    assert_eq!(listed(&south()).trim(), SOUTH);
    assert_eq!(listed(&auditor()).trim(), both().replace(' ', "\n"));
}

#[test]
fn every_escape_is_refused_by_name_or_confined_to_the_dataset() {
    let kernel = kernel();
    let north = north();
    for label in [
        "GRAPH <south>",
        "a sub-select naming south",
        "FROM <south>",
        "SERVICE",
    ] {
        let refused = select(&kernel, &north, NORTH, escape(label));
        assert!(
            matches!(refused, Err(Error::InvalidArgument { ref name, .. }) if name == "query"),
            "{label}: {refused:?}"
        );
    }
    let graphs = select(&kernel, &north, NORTH, escape("GRAPH ?g")).unwrap();
    assert_eq!(
        graphs.lines().collect::<Vec<_>>(),
        ["?g", "<urn:iki:tutorial:graph:north>"]
    );
    let path = select(
        &kernel,
        &north,
        NORTH,
        escape("a property path from an order to a price"),
    );
    assert_eq!(
        path.unwrap().lines().count(),
        1,
        "the path cannot step into south"
    );
    // The pair: the same path over both graphs, by a caller holding both, finds the prices.
    let path = escape("a property path from an order to a price");
    let found = select(&kernel, &auditor(), &both(), path).unwrap();
    assert_eq!(found.lines().count(), 3, "{found}");
}

#[test]
fn a_scoped_write_lands_in_its_own_graph_or_nowhere() {
    let kernel = kernel();
    let north = north();
    assert!(update(&kernel, &north, NORTH, write("an order of its own")).is_ok());
    for label in [
        "a better price, in the supplier's graph",
        "the same price, in the default graph",
        "CREATE GRAPH",
    ] {
        let refused = update(&kernel, &north, NORTH, write(label));
        assert!(
            matches!(refused, Err(Error::Denied(_))),
            "{label}: {refused:?}"
        );
    }
    // A WITH that names another graph is a bad argument for a write scoped to north, refused
    // before it is evaluated (ikigai-store 0.2.24), whether its template would have written south
    // or its WHERE only reads it: the second used to succeed quietly with `+0 -0`.
    for label in [
        "WITH <south>, priced from its own orders",
        "WITH <south>, reading south",
    ] {
        let refused = update(&kernel, &north, NORTH, write(label));
        assert!(
            matches!(&refused, Err(Error::InvalidArgument { name, detail })
                if name == "content" && detail.contains("<urn:iki:tutorial:graph:south>")),
            "{label}: {refused:?}"
        );
    }
    // DROP ALL empties north and nothing else.
    update(&kernel, &north, NORTH, write("DROP ALL")).unwrap();
    let left = select(&kernel, &auditor(), &both(), COSTED_ORDERS).unwrap();
    assert_eq!(left.lines().count(), 1, "north is empty: {left}");
    let prices = select(&kernel, &south(), SOUTH, "SELECT ?p WHERE { ?s ?p ?o }").unwrap();
    assert_eq!(prices.lines().count(), 7, "south is untouched: {prices}");
}

#[test]
fn a_write_grant_alone_neither_reads_nor_is_the_broad_key() {
    let kernel = kernel();
    let counted = update(
        &kernel,
        &feed(),
        SOUTH,
        "INSERT DATA { GRAPH <urn:iki:tutorial:graph:south> { <urn:x> <urn:y> 1 } }",
    )
    .unwrap();
    assert!(!counted.contains('+'), "counts are a read: {counted}");
    let reads = update(
        &kernel,
        &feed(),
        SOUTH,
        "DELETE WHERE { GRAPH <urn:iki:tutorial:graph:south> { ?s ?p ?o } }",
    );
    assert!(
        matches!(reads, Err(Error::Denied(ref m)) if m.contains("WHERE")),
        "{reads:?}"
    );
    let broad = update(
        &kernel,
        &job(),
        SOUTH,
        "CLEAR GRAPH <urn:iki:tutorial:graph:south>",
    );
    assert!(matches!(broad, Err(Error::Denied(_))), "{broad:?}");
}

#[test]
fn a_published_query_is_offered_and_run_only_with_every_graph_it_names() {
    let kernel = kernel();
    issue(
        &kernel,
        &Capability::root(),
        Verb::Sink,
        "urn:script:costed-orders",
        &[("language", "sparql"), ("content", COSTED_SCRIPT)],
    )
    .unwrap();
    let offered = |who: &Capability| {
        issue(&kernel, who, Verb::Source, "urn:kernel:actions", &[])
            .unwrap()
            .contains("urn:script:costed-orders:result")
    };
    assert!(!offered(&north()));
    assert!(!offered(&south()));
    assert!(offered(&auditor()));
    let run = |who: &Capability| {
        issue(
            &kernel,
            who,
            Verb::Source,
            "urn:script:costed-orders:result",
            &[("as", "text/tab-separated-values")],
        )
    };
    assert!(matches!(run(&north()), Err(Error::Denied(ref m)) if m.contains("graph:south")));
    assert!(run(&auditor()).unwrap().contains("3.15"));
}

#[test]
fn the_cache_is_keyed_on_authority_and_never_holds_a_join_over_a_shared_graph() {
    let args = [
        ("graph", NORTH),
        ("query", COSTED_ORDERS),
        ("as", "text/tab-separated-values"),
    ];
    let kernel = kernel();
    assert!(!read(&kernel, &north(), "urn:iki:store:graph-select", &args).1);
    assert!(read(&kernel, &north(), "urn:iki:store:graph-select", &args).1);
    assert!(
        !read(&kernel, &auditor(), "urn:iki:store:graph-select", &args).1,
        "another capability never shares north's entry"
    );

    let (shared, handle) = shared_kernel();
    let both = both();
    let join = [
        ("graph", both.as_str()),
        ("query", COSTED_ORDERS),
        ("as", "text/tab-separated-values"),
    ];
    read(&shared, &north(), "urn:iki:store:graph-select", &args)
        .0
        .unwrap();
    assert!(read(&shared, &north(), "urn:iki:store:graph-select", &args).1);
    read(&shared, &auditor(), "urn:iki:store:graph-select", &join)
        .0
        .unwrap();
    assert!(!read(&shared, &auditor(), "urn:iki:store:graph-select", &join).1);
    feed_writes(
        &handle,
        "<https://south.example/part/washer> <https://schema.org/price> \"0.05\"^^<http://www.w3.org/2001/XMLSchema#decimal> <urn:iki:tutorial:graph:south> .\n",
    );
    let (fresh, _) = read(&shared, &auditor(), "urn:iki:store:graph-select", &join);
    assert!(
        fresh.unwrap().contains("0.05"),
        "the feed's write is seen at once"
    );
}

#[test]
fn an_answer_past_its_bound_is_refused_never_truncated() {
    let kernel = kernel();
    let refused = issue(
        &kernel,
        &north(),
        Verb::Source,
        "urn:iki:store:graph-select",
        &[
            ("graph", NORTH),
            ("query", COSTED_ORDERS),
            ("max_rows", "1"),
        ],
    );
    assert!(
        matches!(refused, Err(Error::InvalidArgument { ref name, .. }) if name == "query"),
        "{refused:?}"
    );
}
