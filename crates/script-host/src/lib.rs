//! The code taught by "Scripts are resources" and "Who a script runs as": a host of the
//! book's own that mounts `ikigai-script` beside an in-memory store.
//!
//! A script is not a feature beside the resources. It IS one, `urn:script:{name}`, with a
//! family of names around it (its versions, its compiled form, its result, its runs), and
//! every door a host has reaches it the way it reaches anything else. This crate does what
//! "Binding, and a host of your own" did for `camel-case`: it composes a kernel from spaces
//! somebody else wrote, and decides the few things only a host can.
//!
//! The data is a handful of work items in the shape gonk's work ledger keeps them (its
//! vocabulary, `https://ikigai-rs.dev/ns/ledger#`), in one named graph, [`ITEMS`], and a
//! second graph, [`PAYROLL`], that the next chapter keeps out of reach.

use std::sync::{Arc, OnceLock, Weak};

use futures::executor::block_on;
use ikigai_core::{
    ArgRef, Capability, Error, Fallback, FixedClock, Iri, Kernel, Request, Space, Verb,
    BINDINGS_THREAD,
};
use ikigai_script::authority::{same_for_all, Ceiling, CeilingPolicy};
use ikigai_script::sparql::SparqlDoor;
use ikigai_script::{ChangeHook, MemoryBackend, SpaceConfig};
use ikigai_store::DurableStore;

/// The named graph the items live in.
pub const ITEMS: &str = "urn:iki:tutorial:items";

/// A second graph, which "Who a script runs as" keeps out of a script's reach.
pub const PAYROLL: &str = "urn:iki:tutorial:payroll";

/// The instant every run in these chapters is stamped with: a fixed clock, so what the
/// programs print is the same on every machine.
pub const BOOK_TIME: u64 = 1_791_709_200_000; // 2026-10-11T09:00:00Z

// ANCHOR: seed
/// Four items and two salaries, written straight to the store: the host's own data, not a
/// script's.
pub const SEED: &str = r#"PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
PREFIX dcterms: <http://purl.org/dc/terms/>
INSERT DATA {
  GRAPH <urn:iki:tutorial:items> {
    <urn:iki:tutorial:item:1> a ledger:Item ; ledger:number 1 ; ledger:priority 1 ;
        ledger:status ledger:open ; dcterms:title "Write the README" .
    <urn:iki:tutorial:item:2> a ledger:Item ; ledger:number 2 ; ledger:priority 0 ;
        ledger:status ledger:open ; dcterms:title "Fix the flaky test" .
    <urn:iki:tutorial:item:3> a ledger:Item ; ledger:number 3 ; ledger:priority 2 ;
        ledger:status ledger:closed ; dcterms:title "Rename the crate" .
    <urn:iki:tutorial:item:4> a ledger:Item ; ledger:number 4 ; ledger:priority 3 ;
        ledger:status ledger:open ; dcterms:title "Answer the issue" .
  }
  GRAPH <urn:iki:tutorial:payroll> {
    <urn:iki:tutorial:person:alice> <urn:iki:tutorial:salary> 100 .
    <urn:iki:tutorial:person:bob> <urn:iki:tutorial:salary> 90 .
  }
}"#;
// ANCHOR_END: seed

// ANCHOR: open_items
/// A query, published as a script: the open items at or above a priority (0 is highest).
pub const OPEN_ITEMS: &str = r#"# The open items at or above a priority, by number.
# @param priority xsd:integer default 3 -- the lowest priority to list (0 is highest)
PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT ?number ?title WHERE {
  GRAPH <urn:iki:tutorial:items> {
    ?item ledger:status ledger:open ; ledger:priority ?p ;
          ledger:number ?number ; dcterms:title ?title .
    FILTER(?p <= ?priority)
  }
} ORDER BY ?number"#;
// ANCHOR_END: open_items

// ANCHOR: close_item
/// An update, published as a script: close one item, by number.
pub const CLOSE_ITEM: &str = r#"# Close one item.
# @param number xsd:integer -- the item to close
PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
DELETE { GRAPH <urn:iki:tutorial:items> { ?item ledger:status ledger:open } }
INSERT { GRAPH <urn:iki:tutorial:items> { ?item ledger:status ledger:closed } }
WHERE  { GRAPH <urn:iki:tutorial:items> { ?item ledger:number ?number ; ledger:status ledger:open } }"#;
// ANCHOR_END: close_item

/// The grant to read [`ITEMS`] through the store's graph-scoped doors.
pub const READ_ITEMS: &str = "urn:cap:store:read:graph:urn:iki:tutorial:items";
/// The grant to write [`ITEMS`].
pub const WRITE_ITEMS: &str = "urn:cap:store:write:graph:urn:iki:tutorial:items";
/// The grant to read [`PAYROLL`].
pub const READ_PAYROLL: &str = "urn:cap:store:read:graph:urn:iki:tutorial:payroll";

// ANCHOR: space
/// The host's spaces, in resolution order: the script host, configured, in front of the store
/// it runs SPARQL against. `ceiling` answers "the most this script may touch" for each script by
/// name, and `on_change` is told about every publish and retire.
pub fn root(ceiling: CeilingPolicy, on_change: Option<ChangeHook>) -> Vec<Arc<dyn Space>> {
    let mut config = SpaceConfig::new(Arc::new(MemoryBackend::new()), ceiling)
        // SPARQL scripts, run through the store's graph-scoped doors at `urn:iki:store:`.
        .sparql(SparqlDoor::store());
    if let Some(hook) = on_change {
        config = config.on_change(hook);
    }
    let store = DurableStore::in_memory().expect("an in-memory store");
    vec![
        Arc::new(ikigai_script::space(config)) as Arc<dyn Space>,
        Arc::new(ikigai_store::space(store)) as Arc<dyn Space>,
    ]
}
// ANCHOR_END: space

/// The host's space with no ceiling and no kernel to tell, for the book's name gate.
pub fn space() -> Fallback {
    Fallback::new(root(same_for_all(Ceiling::unbounded()), None))
}

// ANCHOR: kernel
/// A kernel over [`root`] and then `beside` (other spaces the host mounts), with the store
/// seeded. A published query is its own entry in the catalog, and the kernel caches
/// descriptions under `urn:kernel:bindings`, which only the host can cut: so the change hook
/// reaches back to the kernel it is part of, through a cell filled once the kernel exists.
pub fn kernel_beside(ceiling: CeilingPolicy, beside: Vec<Arc<dyn Space>>) -> Arc<Kernel> {
    let cell: Arc<OnceLock<Weak<Kernel>>> = Arc::new(OnceLock::new());
    let reach = Arc::clone(&cell);
    let hook: ChangeHook = Arc::new(move |_name: &str| {
        if let Some(kernel) = reach.get().and_then(Weak::upgrade) {
            kernel.cut(BINDINGS_THREAD);
        }
    });
    let mut spaces = root(ceiling, Some(hook));
    spaces.extend(beside);
    let kernel = Arc::new(
        Kernel::with_meta_renderer(
            Arc::new(Fallback::new(spaces)),
            Arc::new(ikigai_vocab::TurtleRenderer),
        )
        .with_clock(Arc::new(FixedClock::at(BOOK_TIME))),
    );
    let _ = cell.set(Arc::downgrade(&kernel));
    issue(
        &kernel,
        &Capability::root(),
        Verb::Sink,
        "urn:iki:store:update",
        &[("content", SEED)],
    )
    .expect("seeding the store");
    kernel
}
// ANCHOR_END: kernel

/// [`kernel_beside`] with nothing beside the script host and the store.
pub fn kernel_with(ceiling: CeilingPolicy) -> Arc<Kernel> {
    kernel_beside(ceiling, Vec::new())
}

/// [`kernel_with`] under no ceiling: the host lets a script touch whatever its publisher
/// held and its runner holds.
pub fn kernel() -> Arc<Kernel> {
    kernel_with(same_for_all(Ceiling::unbounded()))
}

// ANCHOR: alice
/// Alice, as the host's door would mint her: a capability naming her as its principal,
/// holding what she may do with these two scripts and the one graph they touch.
pub fn alice() -> Capability {
    Capability::scoped([
        "urn:cap:script:write:open-items",
        "urn:cap:script:read:open-items",
        "urn:cap:script:run:open-items",
        "urn:cap:script:write:close-item",
        "urn:cap:script:run:close-item",
        READ_ITEMS,
        WRITE_ITEMS,
    ])
    .with_principal("urn:iki:tutorial:person:alice")
    .expect("a principal IRI")
}
// ANCHOR_END: alice

/// A request for `verb` on `iri` with inline arguments.
pub fn request(verb: Verb, iri: &str, args: &[(&str, &str)]) -> Request {
    args.iter().fold(
        Request::new(verb, Iri::parse(iri).expect("an IRI")),
        |request, (name, value)| request.with_arg(*name, ArgRef::Inline(value.as_bytes().to_vec())),
    )
}

/// Resolve under `capability`, and the answer as text.
pub fn issue(
    kernel: &Kernel,
    capability: &Capability,
    verb: Verb,
    iri: &str,
    args: &[(&str, &str)],
) -> Result<String, Error> {
    block_on(kernel.issue(request(verb, iri, args), capability))
        .map(|repr| String::from_utf8_lossy(&repr.bytes).into_owned())
}

/// Resolve a read under `capability`, saying whether the kernel answered it from its cache.
/// The question is asked BEFORE the read, of the same request and capability: "would this be
/// served without running anything?"
pub fn read(
    kernel: &Kernel,
    capability: &Capability,
    iri: &str,
    args: &[(&str, &str)],
) -> (Result<String, Error>, bool) {
    let cached = kernel.is_cached(&request(Verb::Source, iri, args), capability);
    (issue(kernel, capability, Verb::Source, iri, args), cached)
}

/// The version IRI a publish answers, trimmed.
pub fn version_of(answer: &str) -> &str {
    answer.trim()
}

/// How the three programs in this section print: a request as the REPL grammar would write
/// it (`who> command`), then its answer or `error:` and the refusal word for word, under
/// numbered sections. Each program runs every section, because each builds on the last, and
/// prints only the one its argument names (`cargo run -p script-host -- 2`), or all of them.
pub mod show {
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    use ikigai_core::{Capability, Error, Kernel, Verb};

    /// The section the program was asked for; 0 for every section.
    static ONLY: AtomicUsize = AtomicUsize::new(0);
    static SHOWING: AtomicBool = AtomicBool::new(true);

    /// Read the program's one optional argument: the section to print.
    pub fn from_args() {
        let only = std::env::args()
            .nth(1)
            .and_then(|a| a.parse::<usize>().ok())
            .unwrap_or(0);
        ONLY.store(only, Ordering::SeqCst);
    }

    /// Start section `n`; print its title when it is shown.
    pub fn section(n: usize, title: &str) {
        let only = ONLY.load(Ordering::SeqCst);
        SHOWING.store(only == 0 || only == n, Ordering::SeqCst);
        if only == 0 {
            line(&format!("\n── {n}. {title}"));
        }
    }

    /// Print one line, when the current section is shown.
    pub fn line(text: &str) {
        if SHOWING.load(Ordering::SeqCst) {
            println!("{text}");
        }
    }

    /// Print text that may or may not end in a newline, as one block.
    pub fn text(text: &str) {
        line(text.strip_suffix('\n').unwrap_or(text));
    }

    /// Print one request, as the REPL grammar would write it.
    pub fn say(who: &str, command: &str) {
        line(&format!("\n{who}> {command}"));
    }

    /// One request under `capability`: printed, issued, and its answer or refusal printed.
    /// The answer comes back for a caller that needs it.
    pub fn ask(
        kernel: &Kernel,
        who: &str,
        capability: &Capability,
        verb: Verb,
        iri: &str,
        args: &[(&str, &str)],
        command: &str,
    ) -> Result<String, Error> {
        say(who, command);
        let answer = super::issue(kernel, capability, verb, iri, args);
        match &answer {
            Ok(body) => text(body),
            Err(e) => line(&format!("error: {e}")),
        }
        answer
    }
}
