//! What "Scripts are resources" says about a script, checked against the real `ikigai-script`
//! and `ikigai-store`: the program's output is checked by `book_runs.rs`; these pin the claims
//! the prose makes about it, so a page cannot keep saying them after they stop being true.

use ikigai_core::{Capability, Error, Verb};
use script_host::{alice, issue, kernel, read, CLOSE_ITEM, OPEN_ITEMS, READ_ITEMS, WRITE_ITEMS};

fn publish(kernel: &ikigai_core::Kernel, name: &str, text: &str) -> String {
    issue(
        kernel,
        &alice(),
        Verb::Sink,
        &format!("urn:script:{name}"),
        &[("language", "sparql"), ("content", text)],
    )
    .unwrap_or_else(|e| panic!("publishing {name}: {e}"))
    .trim()
    .to_string()
}

#[test]
fn a_version_is_named_by_its_content() {
    let kernel = kernel();
    let first = publish(&kernel, "open-items", OPEN_ITEMS);
    assert!(
        first.starts_with("urn:script:open-items:version:sha256:"),
        "{first}"
    );
    // The same text again is the same version; a different text is another.
    assert_eq!(publish(&kernel, "open-items", OPEN_ITEMS), first);
    let other = publish(&kernel, "open-items", &format!("{OPEN_ITEMS}\n"));
    assert_ne!(other, first, "one more byte is another version");
    // Every version stays readable by name after the head moves past it.
    let (old, _) = read(&kernel, &alice(), &first, &[]);
    assert_eq!(old.unwrap(), OPEN_ITEMS);
}

#[test]
fn the_result_is_cached_and_cut_by_a_write_to_what_it_read_and_by_a_republish() {
    let kernel = kernel();
    publish(&kernel, "open-items", OPEN_ITEMS);
    publish(&kernel, "close-item", CLOSE_ITEM);
    let result = |k: &ikigai_core::Kernel| read(k, &alice(), "urn:script:open-items:result", &[]);
    let (first, cached) = result(&kernel);
    assert!(!cached);
    assert!(first.unwrap().contains("Fix the flaky test"));
    assert!(
        result(&kernel).1,
        "the second read is answered from the cache"
    );

    // A run of the update writes the items graph, which cuts the thread the answer hangs from.
    issue(
        &kernel,
        &alice(),
        Verb::Sink,
        "urn:script:close-item:runs",
        &[("number", "2")],
    )
    .expect("running close-item");
    let (after, cached) = result(&kernel);
    assert!(!cached, "recomputed after the write");
    assert!(!after.unwrap().contains("Fix the flaky test"));
    assert!(result(&kernel).1);

    // Republishing cuts the script's own thread.
    publish(
        &kernel,
        "open-items",
        &OPEN_ITEMS.replace("ORDER BY ?number", "ORDER BY DESC(?number)"),
    );
    assert!(!result(&kernel).1, "recomputed after the republish");
}

#[test]
fn a_run_is_an_atom_saying_who_which_version_and_how_it_ended() {
    let kernel = kernel();
    let version = publish(&kernel, "close-item", CLOSE_ITEM);
    let run = issue(
        &kernel,
        &alice(),
        Verb::Sink,
        "urn:script:close-item:runs",
        &[("number", "2")],
    )
    .expect("running close-item");
    assert_eq!(run.trim(), "urn:script:close-item:run:1");
    let record = issue(
        &kernel,
        &alice(),
        Verb::Source,
        "urn:script:close-item:run:1",
        &[("as", "application/json")],
    )
    .expect("the run record");
    let record: serde_json::Value = serde_json::from_str(&record).expect("the JSON face");
    assert_eq!(record["principal"], "urn:iki:tutorial:person:alice");
    assert_eq!(
        format!(
            "urn:script:close-item:version:{}",
            record["version"].as_str().unwrap()
        ),
        version
    );
    assert_eq!(record["outcome"]["status"], "ok", "{record}");
    // Exactly the derived authority: write the graph, and read it (the update has a WHERE).
    assert_eq!(
        record["capability"],
        serde_json::json!([READ_ITEMS, WRITE_ITEMS])
    );
    // A second run is a second atom.
    let again = issue(
        &kernel,
        &alice(),
        Verb::Sink,
        "urn:script:close-item:runs",
        &[("number", "4")],
    )
    .expect("running close-item again");
    assert_eq!(again.trim(), "urn:script:close-item:run:2");
}

#[test]
fn fetching_a_script_needs_its_read_grant_and_never_runs_it() {
    let kernel = kernel();
    publish(&kernel, "open-items", OPEN_ITEMS);
    let stranger = Capability::scoped(["urn:cap:script:run:open-items", READ_ITEMS]);
    match issue(
        &kernel,
        &stranger,
        Verb::Source,
        "urn:script:open-items",
        &[],
    ) {
        Err(Error::Denied(_)) => {}
        other => panic!("a runner without the read grant may not read the source: {other:?}"),
    }
    // Fetching the source left nothing cached for the result: nothing ran.
    assert!(!kernel.is_cached(
        &script_host::request(Verb::Source, "urn:script:open-items:result", &[]),
        &alice()
    ));
}
