//! What the chapter claims about its drafts, checked against the real `ikigai-nl` and
//! `ikigai-script`, with the stub as the model.
//!
//! The live demo (gonk, 2026-10-11) found a draft that compared an IRI-valued status to the
//! string `"open"` AND paired `FROM <g>` with `GRAPH <g> { … }`, previewed nothing, and passed
//! the check. Its note blamed both for the empty preview. On a host that runs a query through
//! the store's graph-scoped doors (ikigai-script's dataset: every graph the text names is both
//! the default graph and a named graph), only the first is a defect: these tests say which,
//! so the chapter cannot say otherwise.

use ikigai_core::Verb;
use ikigai_nl::{Check, Draft, Status};
use script_host::{issue, ITEMS};
use sentence_draft::{
    alice, bob, carol, kernel, publish_example, Model, ASK_P2, ASK_TITLES, P2_CORRECTED, P2_DRAFT,
    TITLES_FIRST,
};

fn check(text: &str) -> Check {
    let kernel = kernel(&Model::stub());
    let json = issue(
        &kernel,
        &alice(),
        Verb::Source,
        "urn:nl:sparql:check",
        &[("content", text), ("as", "application/json")],
    )
    .unwrap_or_else(|e| panic!("the check: {e}"));
    serde_json::from_str(&json).expect("the check's JSON face")
}

fn rows(check: &Check) -> usize {
    check.preview.as_ref().map_or(0, |p| p.rows.len())
}

#[test]
fn the_demo_draft_passes_the_check_with_an_empty_preview() {
    let c = check(P2_DRAFT);
    assert!(c.valid, "{c:#?}");
    assert!(c.errors.is_empty(), "{:?}", c.errors);
    assert_eq!(
        rows(&c),
        0,
        "the preview is empty, and the check does not call that a failure (ledger #1242)"
    );
}

#[test]
fn the_string_status_alone_empties_it() {
    // The status as the IRI it is, FROM and GRAPH both kept: the rows come back.
    let iri_status = P2_DRAFT.replace("\"open\"", "<https://ikigai-rs.dev/ns/ledger#open>");
    assert_eq!(
        rows(&check(&iri_status)),
        2,
        "FROM paired with GRAPH is not what emptied it here"
    );
    // The string kept, FROM dropped: still empty.
    let no_from = P2_DRAFT.replace("FROM <urn:iki:tutorial:items>\n", "");
    assert_eq!(rows(&check(&no_from)), 0);
}

#[test]
fn the_corrected_query_previews_the_two_items() {
    let c = check(P2_CORRECTED);
    assert!(c.valid, "{c:#?}");
    assert_eq!(rows(&c), 2);
}

#[test]
fn the_check_catches_an_unknown_predicate_and_the_drafter_repairs_it() {
    let c = check(TITLES_FIRST);
    assert!(!c.valid);
    assert!(
        c.errors.iter().any(|e| e.contains("ledger#state")),
        "{:?}",
        c.errors
    );

    let model = Model::stub();
    let kernel = kernel(&model);
    let json = issue(
        &kernel,
        &alice(),
        Verb::Sink,
        "urn:nl:sparql",
        &[
            ("ask", ASK_TITLES),
            ("graph", ITEMS),
            ("save", "open-titles"),
            ("as", "application/json"),
        ],
    )
    .expect("drafting");
    let draft: Draft = serde_json::from_str(&json).expect("the draft's JSON face");
    assert_eq!(draft.status, Status::Saved, "{draft:#?}");
    assert_eq!(draft.attempts.len(), 2);
    assert!(!draft.attempts[0].valid && draft.attempts[1].valid);
    let Model::Stub(stub) = &model else {
        unreachable!()
    };
    let prompts = &stub.lock().unwrap().prompts;
    assert_eq!(prompts.len(), 2);
    // The repair prompt carries the refused draft and what was refused, nothing more.
    assert!(prompts[1].contains("ledger#state"), "{}", prompts[1]);
}

#[test]
fn a_saved_draft_is_its_authors_alone_until_a_person_publishes_it() {
    let kernel = kernel(&Model::stub());
    issue(
        &kernel,
        &alice(),
        Verb::Sink,
        "urn:nl:sparql",
        &[("ask", ASK_P2), ("graph", ITEMS), ("save", "open-p2")],
    )
    .expect("drafting");
    let head = issue(
        &kernel,
        &alice(),
        Verb::Source,
        "urn:script:open-p2",
        &[("as", "application/json")],
    )
    .expect("alice sees her draft");
    assert!(head.contains("\"state\": \"draft\""), "{head}");
    match issue(&kernel, &bob(), Verb::Source, "urn:script:open-p2", &[]) {
        Err(ikigai_core::Error::NotFound(_)) => {}
        other => panic!("bob holds the read grant and must not learn the draft exists: {other:?}"),
    }
    issue(
        &kernel,
        &alice(),
        Verb::Sink,
        "urn:script:open-p2",
        &[
            ("language", "sparql"),
            ("state", "published"),
            ("content", P2_CORRECTED),
        ],
    )
    .expect("publishing");
    assert!(issue(&kernel, &bob(), Verb::Source, "urn:script:open-p2", &[]).is_ok());
}

#[test]
fn the_grounding_is_the_callers_own_view() {
    let kernel = kernel(&Model::stub());
    publish_example(&kernel);
    let view = |who| {
        let json = issue(
            &kernel,
            &who,
            Verb::Source,
            "urn:nl:grounding",
            &[("as", "application/json")],
        )
        .expect("the grounding");
        serde_json::from_str::<ikigai_nl::Grounding>(&json).expect("its JSON face")
    };
    let (a, c) = (view(alice()), view(carol()));
    assert_eq!(
        a.graphs
            .items
            .iter()
            .map(|g| g.graph.as_str())
            .collect::<Vec<_>>(),
        vec![ITEMS]
    );
    assert!(c.graphs.items.is_empty());
    assert_eq!(a.examples.items.len(), 1);
    assert!(c.examples.items.is_empty());
    assert!(a.actions.of > c.actions.of);
}
