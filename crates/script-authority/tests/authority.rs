//! What "Who a script runs as" says, pinned against the real `ikigai-script`: the refusal's
//! TYPE for each case (the program prints its words, `book_runs.rs` checks them), and the
//! rule each answer follows from.

use ikigai_core::{Capability, Error, Kernel, Verb};
use script_authority::{alice, bob, carol, kernel, Ceilings, GRAPHS, SALARIES};
use script_host::{issue, OPEN_ITEMS, READ_ITEMS};

fn sparql(kernel: &Kernel, who: &Capability, name: &str, text: &str) -> Result<String, Error> {
    issue(
        kernel,
        who,
        Verb::Sink,
        &format!("urn:script:{name}"),
        &[("language", "sparql"), ("content", text)],
    )
}

fn result(kernel: &Kernel, who: &Capability, name: &str) -> Result<String, Error> {
    issue(
        kernel,
        who,
        Verb::Source,
        &format!("urn:script:{name}:result"),
        &[("as", "text/tab-separated-values")],
    )
}

#[test]
fn a_publish_never_gives_a_script_more_than_its_publisher_holds() {
    let kernel = kernel(&Ceilings::new());
    // Carol may write `salaries` but cannot read the graph it names: no elevation.
    match sparql(&kernel, &carol(), "salaries", SALARIES) {
        Err(Error::Denied(m)) => assert!(
            m.contains("urn:cap:store:read:graph:urn:iki:tutorial:payroll"),
            "{m}"
        ),
        other => panic!("{other:?}"),
    }
    // A SPARQL script's authority is derived; a `requires` that says otherwise is refused.
    let declared = issue(
        &kernel,
        &alice(),
        Verb::Sink,
        "urn:script:open-items",
        &[
            ("language", "sparql"),
            ("content", OPEN_ITEMS),
            (
                "requires",
                "urn:cap:store:read:graph:urn:iki:tutorial:payroll",
            ),
        ],
    );
    assert!(
        matches!(declared, Err(Error::InvalidArgument { ref name, .. }) if name == "requires"),
        "{declared:?}"
    );
}

#[test]
fn publishing_and_running_are_separate_grants() {
    let kernel = kernel(&Ceilings::new());
    sparql(&kernel, &alice(), "open-items", OPEN_ITEMS).expect("alice publishes");
    // Alice published it and holds no run grant: refused at the floor. A hyphenated name
    // declares its top-level namespace as its run gate (`authority::run_floor`).
    match result(&kernel, &alice(), "open-items") {
        Err(Error::Denied(m)) => assert!(m.contains("urn:cap:script:run:open-*"), "{m}"),
        other => panic!("{other:?}"),
    }
    // Carol may run it and reads no graph: the run is her capability, narrowed, never widened.
    assert!(matches!(
        result(&kernel, &carol(), "open-items"),
        Err(Error::Denied(_))
    ));
    assert!(result(&kernel, &bob(), "open-items")
        .unwrap()
        .contains("Fix the flaky test"));
}

#[test]
fn a_run_holds_the_least_of_runner_publisher_and_ceiling() {
    let ceilings = Ceilings::new();
    let kernel = kernel(&ceilings);
    // Published by alice, who reads the items alone: bob, who reads both, sees one graph.
    sparql(&kernel, &alice(), "graphs", GRAPHS).unwrap();
    let rows = result(&kernel, &bob(), "graphs").unwrap();
    assert!(
        rows.contains("<urn:iki:tutorial:items>") && !rows.contains("payroll"),
        "{rows}"
    );
    // Run for the record: the run's capability is exactly that, and alice's exclusion rides along.
    let run = issue(&kernel, &bob(), Verb::Sink, "urn:script:graphs:runs", &[]).unwrap();
    let record = issue(&kernel, &bob(), Verb::Source, run.trim(), &[]).unwrap();
    assert!(
        record.contains(&format!(
            "capability: urn:cap:script:write:-team-payroll {READ_ITEMS}\n"
        )),
        "{record}"
    );

    // Published by the host (root, which holds every graph), under a ceiling naming the family.
    sparql(&kernel, &Capability::root(), "every-graph", GRAPHS).unwrap();
    assert!(result(&kernel, &bob(), "every-graph")
        .unwrap()
        .contains("payroll"));
    // Narrow the ceiling: the cached answer outlives the file until the host cuts the thread.
    ceilings.set(
        "every-graph",
        "urn:cap:store:read:graph:urn:iki:tutorial:items\n",
    );
    assert!(
        result(&kernel, &bob(), "every-graph")
            .unwrap()
            .contains("payroll"),
        "stale until cut"
    );
    kernel.cut(ikigai_script::name::authority_thread("every-graph"));
    assert!(!result(&kernel, &bob(), "every-graph")
        .unwrap()
        .contains("payroll"));

    // No file at all: the host's policy answers nothing, and the run reads nothing.
    sparql(&kernel, &Capability::root(), "orphan", GRAPHS).unwrap();
    assert!(matches!(
        result(&kernel, &bob(), "orphan"),
        Err(Error::Denied(_))
    ));
}

#[test]
fn a_namespace_grant_covers_its_names_and_an_exclusion_wins() {
    let kernel = kernel(&Ceilings::new());
    for name in ["team-open", "team-a-open"] {
        sparql(&kernel, &alice(), name, OPEN_ITEMS).unwrap_or_else(|e| panic!("{name}: {e}"));
    }
    for name in ["teammate", "team-payroll", "team"] {
        assert!(
            matches!(
                sparql(&kernel, &alice(), name, OPEN_ITEMS),
                Err(Error::Denied(_))
            ),
            "{name}"
        );
    }
    // `urn:cap:script:run:team-*` runs a nested name too.
    assert!(result(&kernel, &bob(), "team-a-open").is_ok());
}
