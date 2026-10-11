//! "A sentence becomes a draft", end to end.
//!
//! ```text
//! cargo run -p sentence-draft                          # the stub model, as every gate runs it
//! cargo run -p sentence-draft -- 3                     # only section 3 (every section still runs)
//! cargo run -p sentence-draft -- 0 --ollama llama3.1   # a local Ollama, at that model
//! cargo run -p sentence-draft -- ask                   # "Your first script", step 4 (the stub)
//! ```
//!
//! Each line that starts `alice>`, `bob>`, `carol>` or `host>` is one request under that
//! capability, written the way the REPL grammar would write it; what follows is the answer.

use ikigai_core::{Capability, Kernel, Verb};
use ikigai_nl::Grounding;
use script_host::first::SHELF;
use script_host::show::{self, line, say, section};
use script_host::{issue, ITEMS};
use sentence_draft::{
    alice, bob, carol, kernel, publish_example, shelf_kernel, Model, ASK_BOOKS, ASK_P2, ASK_TITLES,
    P2_CORRECTED, TITLES_FIRST,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // The first page's step, `-- ask`: a sentence on the shelf, on a host of its own.
    if args.get(1).map(String::as_str) == Some("ask") {
        return first_ask();
    }
    show::from_args();
    // ANCHOR: model
    let model = match args.iter().position(|a| a == "--ollama") {
        Some(at) => Model::Ollama(args.get(at + 1).cloned().unwrap_or("llama3.1".into())),
        None => Model::stub(),
    };
    let kernel = kernel(&model);
    // ANCHOR_END: model
    publish_example(&kernel);
    let (alice, bob, carol, host) = (alice(), bob(), carol(), Capability::root());

    section(1, "What the model may know is what the caller may see");
    grounding(&kernel, "alice", &alice);
    grounding(&kernel, "carol", &carol);

    section(2, "The check");
    ask(&kernel, "alice", &alice, Verb::Source, "urn:nl:sparql:check",
        &[("focus", ITEMS), ("content", TITLES_FIRST)],
        "source urn:nl:sparql:check focus=urn:iki:tutorial:items   (a query naming ledger:state, piped)");
    let payroll = "SELECT ?who ?salary WHERE { GRAPH <urn:iki:tutorial:payroll> { ?who <urn:iki:tutorial:salary> ?salary } }";
    ask(
        &kernel,
        "alice",
        &alice,
        Verb::Source,
        "urn:nl:sparql:check",
        &[("content", payroll)],
        "source urn:nl:sparql:check   (a query over the payroll graph, piped)",
    );
    let nowhere = payroll.replace("payroll", "nowhere");
    ask(
        &kernel,
        "alice",
        &alice,
        Verb::Source,
        "urn:nl:sparql:check",
        &[("content", &nowhere)],
        "source urn:nl:sparql:check   (the same query over a graph that does not exist, piped)",
    );

    section(3, "A sentence, drafted, checked, repaired and saved");
    // ANCHOR: draft
    ask(
        &kernel,
        "alice",
        &alice,
        Verb::Sink,
        "urn:nl:sparql",
        &[
            ("ask", ASK_TITLES),
            ("graph", ITEMS),
            ("save", "open-titles"),
        ],
        &format!(
            "sink urn:nl:sparql ask=\"{ASK_TITLES}\" graph=urn:iki:tutorial:items save=open-titles"
        ),
    );
    // ANCHOR_END: draft
    if let Model::Stub(stub) = &model {
        let prompts = stub.lock().expect("the stub").prompts.len();
        line(&format!("\n(the model was asked {prompts} times)"));
    }

    section(4, "A draft the check passed, and should not have");
    ask(
        &kernel,
        "alice",
        &alice,
        Verb::Sink,
        "urn:nl:sparql",
        &[("ask", ASK_P2), ("graph", ITEMS), ("save", "open-p2")],
        &format!("sink urn:nl:sparql ask=\"{ASK_P2}\" graph=urn:iki:tutorial:items save=open-p2"),
    );

    section(5, "A draft is nobody's script yet");
    ask(
        &kernel,
        "bob",
        &bob,
        Verb::Source,
        "urn:script:open-p2",
        &[],
        "source urn:script:open-p2",
    );
    ask(
        &kernel,
        "host",
        &host,
        Verb::Source,
        "urn:script:open-p2:result",
        &[],
        "source urn:script:open-p2:result",
    );
    ask(
        &kernel,
        "alice",
        &alice,
        Verb::Source,
        "urn:script:open-p2:result",
        &[],
        "source urn:script:open-p2:result",
    );

    section(6, "A person corrects it, checks it, and publishes it");
    ask(
        &kernel,
        "alice",
        &alice,
        Verb::Source,
        "urn:nl:sparql:check",
        &[("content", P2_CORRECTED)],
        "source urn:nl:sparql:check   (the corrected query, piped)",
    );
    ask(
        &kernel,
        "alice",
        &alice,
        Verb::Sink,
        "urn:script:open-p2",
        &[
            ("language", "sparql"),
            ("state", "published"),
            ("content", P2_CORRECTED),
        ],
        "sink urn:script:open-p2 language=sparql state=published   (the corrected query, piped)",
    );
    ask(
        &kernel,
        "alice",
        &alice,
        Verb::Source,
        "urn:script:open-p2:result",
        &[("as", "text/tab-separated-values")],
        "source urn:script:open-p2:result as=text/tab-separated-values",
    );
    ask(
        &kernel,
        "bob",
        &bob,
        Verb::Source,
        "urn:script:open-p2",
        &[],
        "source urn:script:open-p2",
    );
}

/// The grounding `who` gets, summarized from its JSON face: each part's shown-of-offered
/// count, each graph it describes, and each worked example.
fn grounding(kernel: &Kernel, who: &str, capability: &Capability) {
    say(
        who,
        "source urn:nl:grounding as=application/json   (summarized)",
    );
    let text = issue(
        kernel,
        capability,
        Verb::Source,
        "urn:nl:grounding",
        &[("as", "application/json")],
    )
    .unwrap_or_else(|e| panic!("the grounding: {e}"));
    let g: Grounding = serde_json::from_str(&text).expect("the JSON face parses");
    line(&format!(
        "actions     {} of {}",
        g.actions.shown, g.actions.of
    ));
    line(&format!(
        "vocabulary  {} of {}",
        g.vocabulary.shown, g.vocabulary.of
    ));
    line(&format!(
        "graphs      {} of {}",
        g.graphs.shown, g.graphs.of
    ));
    for shape in &g.graphs.items {
        line(&format!(
            "  {}  {} triples, {} predicates",
            shape.graph, shape.triples, shape.properties
        ));
    }
    line(&format!(
        "examples    {} of {}",
        g.examples.shown, g.examples.of
    ));
    for example in &g.examples.items {
        line(&format!(
            "  {} ({}, {})",
            example.name, example.language, example.version
        ));
    }
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

/// "Your first script", step 4: ask in a sentence, look at the draft, publish it, run it. On a
/// host of its own, holding the shelf of three books, with the stub as the model.
fn first_ask() {
    let kernel = shelf_kernel(&Model::stub());
    let you = Capability::root();
    ask(
        &kernel,
        "you",
        &you,
        Verb::Sink,
        "urn:nl:sparql",
        &[("ask", ASK_BOOKS), ("graph", SHELF), ("save", "book-count")],
        &format!(
            "sink urn:nl:sparql ask=\"{ASK_BOOKS}\" graph=urn:iki:tutorial:shelf save=book-count"
        ),
    );
    // Publishing is the draft's own text, sunk again with `state=published`.
    say(
        "you",
        "source urn:script:book-count | sink urn:script:book-count language=sparql state=published",
    );
    let draft = issue(&kernel, &you, Verb::Source, "urn:script:book-count", &[])
        .unwrap_or_else(|e| panic!("the draft: {e}"));
    let published = issue(
        &kernel,
        &you,
        Verb::Sink,
        "urn:script:book-count",
        &[
            ("language", "sparql"),
            ("state", "published"),
            ("content", &draft),
        ],
    )
    .unwrap_or_else(|e| panic!("publishing the draft: {e}"));
    show::text(&published);
    ask(
        &kernel,
        "you",
        &you,
        Verb::Source,
        "urn:script:book-count:result",
        &[("as", "text/csv")],
        "source urn:script:book-count:result as=text/csv",
    );
}
