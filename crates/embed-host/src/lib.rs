//! **The ledger, the store and browse in your own kernel** — the code The gonk Book's
//! embedding chapter teaches.
//!
//! gonk is a composition and its doors. This crate is the composition without the doors: ONE
//! dataset that the ledger and browse both write, the store's resources over it, browse over a
//! repository with a (stub) model mounted at `urn:llm:`, and a capability per caller instead of
//! a capability per door.
//!
//! ## The resource map, as built
//!
//! * **Sets.** Ledgers and their items; roots and their files; findings; annotations.
//! * **Atoms that hold state.** One in-memory dataset, holding the ledger's graph
//!   (`urn:iki:ledger:graph:default`) and browse's (`urn:iki:browse:graph:default`: the explain
//!   archive, review passes, findings, decisions, annotations); the repository's files; and the
//!   model, an input bound at `urn:llm:stub:*`.
//! * **Compositions.** Every ledger action; explain, review and the finding decisions; the
//!   store's query doors, which is where the two graphs JOIN.
//! * **Views.** `next`, the file listings, `explain-status`, `findings`, `annotations`.
//! * **Code.** [`space`] (one dataset handed to two families, with a promise about where the
//!   second one writes) and three capabilities. No endpoint.

use std::path::Path;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use browse_host::{stub_model, PROVIDER};
use ikigai_browse::{ExplainConfig, Mount};
use ikigai_core::{Capability, Fallback, Kernel, Space};
use ikigai_store::{DurableStore, SharerWrites};
use oxigraph::model::NamedNode;

pub use browse_host::Scratch;
pub use ledger_host::{ask, ask_as};

/// The graph browse writes in, as gonk names it.
pub const BROWSE_GRAPH: &str = "urn:iki:browse:graph:default";

// ANCHOR: space
/// One dataset, three families and a model.
pub fn space(root: &Path, calls: Arc<AtomicUsize>) -> Fallback {
    // The store HANDS OUT its dataset to one other holder, browse — and is told where that
    // holder writes. With the promise, a scoped read of any other graph (every ledger read) is
    // still cacheable; without it the store would have to assume browse might write anywhere,
    // and no read of anything could be cached.
    let writes = SharerWrites::only_the_default_graph().and_named_graph(BROWSE_GRAPH);
    let (store, dataset) =
        DurableStore::in_memory_shared_declaring(writes).expect("an in-memory store opens");

    // browse over the same dataset, confined to its graph: the promise above, kept.
    let browse = Mount::new([("demo".to_string(), root.to_path_buf())])
        .config_home(None)
        .graph(NamedNode::new(BROWSE_GRAPH).expect("an IRI"))
        .explain(
            ExplainConfig::new(dataset)
                .file_provider(PROVIDER)
                .dir_provider(PROVIDER)
                .review_provider(PROVIDER)
                .no_judge(),
        )
        .space();

    Fallback::new(vec![
        Arc::new(ikigai_store::space(store)) as Arc<dyn Space>,
        Arc::new(ikigai_ledger::space()) as Arc<dyn Space>,
        Arc::new(browse) as Arc<dyn Space>,
        Arc::new(stub_model(calls)) as Arc<dyn Space>,
    ])
}

/// The host: [`space`], a Meta renderer and the story clock.
pub fn kernel(root: &Path, calls: Arc<AtomicUsize>) -> Kernel {
    Kernel::with_meta_renderer(
        Arc::new(space(root, calls)),
        Arc::new(ikigai_vocab::TurtleRenderer),
    )
    .with_clock(Arc::new(ledger_host::StoryClock::default()))
}
// ANCHOR_END: space

// ANCHOR: callers
/// What the agent holds: one ledger's writer (the four tokens `ikigai-gonk grants default write`
/// prints), the repository, and browse's graph through the store's narrow read door.
const AGENT: [&str; 6] = [
    "urn:cap:ledger:read:default",
    "urn:cap:ledger:write:default",
    "urn:cap:store:read:graph:urn:iki:ledger:graph:default",
    "urn:cap:store:write:graph:urn:iki:ledger:graph:default",
    "urn:cap:browse:read:demo",
    "urn:cap:store:read:graph:urn:iki:browse:graph:default",
];

/// An agent working the `default` ledger: it files and reads work, reads the repository and
/// browse's graph (so it can JOIN its items to what browse knows), and spends nothing.
pub fn agent() -> Capability {
    Capability::scoped(AGENT)
}

/// A headless reviewer: it may spend the model on the repository and read what it derived,
/// and it can neither publish a finding nor touch the ledger.
pub fn reviewer() -> Capability {
    Capability::scoped(["urn:cap:browse:read:demo", "urn:cap:net:localhost"])
}

/// A person: the agent's grants, plus spending the model and deciding about findings.
pub fn person() -> Capability {
    Capability::scoped(
        AGENT
            .into_iter()
            .chain(["urn:cap:net:localhost", "urn:cap:annotate"]),
    )
}
// ANCHOR_END: callers
