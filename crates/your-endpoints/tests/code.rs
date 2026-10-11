//! The exercises for "Code and language as resources" are done here — against
//! `crates/script-host`, `crates/script-authority` and `crates/sentence-draft`, which this file
//! uses and does not edit.
//!
//! What is here is the scaffolding an exercise would otherwise spend its time on: publishing a
//! query, minting a person, reading whether an answer came from the cache, and one test proving
//! all of it works. The assertions the exercises ask for are yours to write, below the line.

#![allow(dead_code)] // each exercise uses a different part of the scaffolding

use std::sync::{Arc, Mutex};

use ikigai_core::{Capability, Error, Kernel, Verb};
use sentence_draft::stub::Stub;
use sentence_draft::Model;

/// Publish `text` as a SPARQL script at `urn:script:{name}`, under `who`; the version's name.
pub fn publish(kernel: &Kernel, who: &Capability, name: &str, text: &str) -> Result<String, Error> {
    script_host::issue(
        kernel,
        who,
        Verb::Sink,
        &format!("urn:script:{name}"),
        &[("language", "sparql"), ("content", text)],
    )
    .map(|version| version.trim().to_string())
}

/// `urn:script:{name}:result` as TSV under `who`, and whether the kernel answered it from its
/// cache (asked before the read).
pub fn result(
    kernel: &Kernel,
    who: &Capability,
    name: &str,
    args: &[(&str, &str)],
) -> (Result<String, Error>, bool) {
    let mut all = vec![("as", "text/tab-separated-values")];
    all.extend_from_slice(args);
    script_host::read(kernel, who, &format!("urn:script:{name}:result"), &all)
}

/// A person, as a host's door would mint them: these grants, naming them as the principal.
pub fn person(name: &str, scopes: &[&str]) -> Capability {
    Capability::scoped(scopes.iter().copied())
        .with_principal(&format!("urn:iki:tutorial:person:{name}"))
        .expect("a principal IRI")
}

/// A stub model answering each `(ask, drafts)` with its drafts, in order, and the handle to
/// read back what it was sent (`stub.lock().unwrap().prompts`).
pub fn stub_model(drafts: &[(&str, &[&str])]) -> (Model, Arc<Mutex<Stub>>) {
    let stub = Arc::new(Mutex::new(Stub::new(drafts)));
    (Model::Stub(Arc::clone(&stub)), stub)
}

#[test]
fn the_scaffolding_works() {
    let kernel = script_host::kernel();
    let alice = script_host::alice();
    publish(&kernel, &alice, "open-items", script_host::OPEN_ITEMS).expect("publishing");
    let (first, cached) = result(&kernel, &alice, "open-items", &[]);
    assert!(first.expect("running").contains("Fix the flaky test"));
    assert!(!cached);
    assert!(result(&kernel, &alice, "open-items", &[]).1);

    let (model, stub) = stub_model(&[("anything", &["SELECT * WHERE { ?s ?p ?o }"])]);
    let _ = sentence_draft::kernel(&model);
    assert!(stub.lock().unwrap().prompts.is_empty());
}

// ---------------------------------------------------------------------------------------------
// Your exercises, below this line.
