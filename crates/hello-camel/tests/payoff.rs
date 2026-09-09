//! The four demonstrations behind **What resolution buys you**.
//!
//! Each test here is a listing in that chapter, included by anchor, and each one proves
//! a claim the chapter makes — served from cache without the endpoint running, a golden
//! thread cut by a write, a sub-resolution showing up as a child span, a description
//! that is a graph. Run them with the output showing:
//!
//! ```text
//! cargo test -p hello-camel --test payoff -- --nocapture --test-threads 1
//! ```
//!
//! Every test builds its own kernel. The cache and the golden threads live in the
//! `Kernel`, so two tests sharing one would see each other's state — and a test that
//! wants to watch `title`'s read counter builds the kernel over a `TitleState` it keeps.

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use futures::executor::block_on;
use hello_camel::{kernel, kernel_over, TitleState, TITLE};
use ikigai_core::{
    ArgRef, Capability, Iri, Kernel, ReprType, Request, Result, TraceEvent, Tracer, Verb,
};

fn iri(name: &str) -> Iri {
    Iri::parse(name).expect("a valid IRI")
}

/// `Source` a name and hand back the text — the same four lines every test in this
/// crate starts from.
fn source(kernel: &Kernel, name: &str) -> Result<String> {
    let repr = block_on(kernel.issue(Request::new(Verb::Source, iri(name)), &Capability::root()))?;
    Ok(String::from_utf8_lossy(&repr.bytes).into_owned())
}

/// `Sink` new content into a name.
fn sink(kernel: &Kernel, name: &str, content: &str) -> Result<()> {
    let request = Request::new(Verb::Sink, iri(name))
        .with_arg("content", ArgRef::Inline(content.as_bytes().to_vec()));
    block_on(kernel.issue(request, &Capability::root()))?;
    Ok(())
}

// ANCHOR: cached
/// The second `Source` of a cacheable resource is served without the endpoint running.
#[test]
fn a_cacheable_resource_is_computed_once_and_then_served() -> Result<()> {
    let state = Arc::new(TitleState::default());
    let kernel = kernel_over(state.clone());
    let request = Request::new(Verb::Source, iri(TITLE));

    // Nothing has been resolved yet, so nothing is cached.
    assert!(!kernel.is_cached(&request, &Capability::root()));

    let first = source(&kernel, TITLE)?;
    assert!(kernel.is_cached(&request, &Capability::root()));

    let second = source(&kernel, TITLE)?;
    assert_eq!(first, second);

    // Same bytes twice is not the proof — a recompute would also give the same bytes.
    // The proof is that the endpoint ran once.
    assert_eq!(state.reads.load(Ordering::SeqCst), 1);
    println!("cached once: {first:?} served twice, endpoint ran 1 time");
    Ok(())
}
// ANCHOR_END: cached

// ANCHOR: cut
/// A write to `title` invalidates `camel-title`, which never named the thread it depends
/// on — it inherited it by resolving `title`.
#[test]
fn a_sink_upstream_cuts_the_thread_and_the_composite_recomputes() -> Result<()> {
    let state = Arc::new(TitleState::default());
    let kernel = kernel_over(state.clone());
    let composite = Request::new(Verb::Source, iri("urn:iki:tutorial:camel-title"));

    assert_eq!(
        source(&kernel, "urn:iki:tutorial:camel-title")?,
        "resourceOrientedComputing"
    );
    assert!(kernel.is_cached(&composite, &Capability::root()));
    assert_eq!(state.reads.load(Ordering::SeqCst), 1);

    // The write. The kernel cuts the thread named `urn:iki:tutorial:title` on its way
    // out, because that is the target of a successful mutating verb.
    sink(&kernel, TITLE, "golden threads cut")?;

    // ...and the composite's cache entry is gone with it, transitively.
    assert!(!kernel.is_cached(&composite, &Capability::root()));
    assert_eq!(
        source(&kernel, "urn:iki:tutorial:camel-title")?,
        "goldenThreadsCut"
    );
    // The recompute went all the way down: `title` was read again, not served stale.
    assert_eq!(state.reads.load(Ordering::SeqCst), 2);
    println!("thread cut: camel-title recomputed after a Sink to title");
    Ok(())
}
// ANCHOR_END: cut

// ANCHOR: tracer
/// A tracer that keeps every event. The kernel hands it one `TraceEvent` per invocation
/// of the resolution it was passed to — and only that resolution.
#[derive(Default)]
struct Recorder(Mutex<Vec<TraceEvent>>);

impl Tracer for Recorder {
    fn record(&self, event: TraceEvent) {
        self.0.lock().expect("recorder lock").push(event);
    }
}
// ANCHOR_END: tracer

// ANCHOR: traced
/// Tracing one resolution shows the sub-resolution `camel-title` made, as a child span.
#[test]
fn a_traced_resolution_shows_the_sub_resolution_as_a_child_span() -> Result<()> {
    let kernel = kernel();
    let recorder = Arc::new(Recorder::default());
    let request = Request::new(Verb::Source, iri("urn:iki:tutorial:camel-title"));

    block_on(kernel.issue_traced(request, &Capability::root(), recorder.clone()))?;

    let events = recorder.0.lock().expect("recorder lock").clone();
    for event in &events {
        println!(
            "span {} parent {:?}  {}  cache_hit={}",
            event.span, event.parent, event.target, event.cache_hit
        );
    }

    // Two invocations: the one we asked for, and the one it made.
    let root = events
        .iter()
        .find(|e| e.parent.is_none())
        .expect("a root span");
    let child = events
        .iter()
        .find(|e| e.parent.is_some())
        .expect("a child span");
    assert_eq!(root.target, "urn:iki:tutorial:camel-title");
    assert_eq!(child.target, TITLE);
    assert_eq!(child.parent, Some(root.span));
    assert!(
        !child.cache_hit,
        "a fresh kernel: the sub-resolution was computed"
    );
    Ok(())
}
// ANCHOR_END: traced

// ANCHOR: described
/// `Meta` on `camel-case`, rendered as Turtle: the description is a graph, and the
/// endpoint's inputs are nodes in it with stable names.
#[test]
fn meta_renders_the_description_as_a_graph() -> Result<()> {
    let kernel = kernel();
    let request = Request::new(Verb::Meta, iri("urn:iki:tutorial:camel-case"))
        .with_arg("as", ArgRef::Inline(b"text/turtle".to_vec()));

    let repr = block_on(kernel.issue(request, &Capability::root()))?;
    let turtle = String::from_utf8_lossy(&repr.bytes);
    println!("{turtle}");

    assert_eq!(repr.repr_type.media_type, "text/turtle");
    assert!(turtle.contains("<urn:ikigai:endpoint:camel-case> a ik:Endpoint"));
    assert!(turtle.contains("ik:input <urn:ikigai:endpoint:camel-case:input:in>"));
    assert!(turtle.contains("ik:class <http://www.w3.org/2001/XMLSchema#string>"));
    Ok(())
}
// ANCHOR_END: described

// ANCHOR: catalog
/// `urn:kernel:catalog` is every bound endpoint's description, as one graph.
#[test]
fn the_catalog_is_one_graph_over_every_binding() -> Result<()> {
    let kernel = kernel();
    let catalog = source(&kernel, "urn:kernel:catalog")?;

    // Ours, and ikigai-fn's — the host is mostly other people's endpoints.
    assert!(catalog.contains("<urn:ikigai:endpoint:camel-case> a ik:Endpoint"));
    assert!(catalog.contains("<urn:ikigai:endpoint:title> a ik:Endpoint"));
    assert!(catalog.contains("<urn:ikigai:endpoint:toUpper> a ik:Endpoint"));
    // The two-verb endpoint shows up as two actions, one per verb.
    assert!(catalog.contains("<urn:ikigai:endpoint:title:action:source> a ik:Action"));
    assert!(catalog.contains("<urn:ikigai:endpoint:title:action:sink> a ik:Action"));
    Ok(())
}
// ANCHOR_END: catalog

#[test]
fn a_representation_carries_its_type() -> Result<()> {
    let kernel = kernel();
    let repr = block_on(kernel.issue(Request::new(Verb::Source, iri(TITLE)), &Capability::root()))?;
    assert_eq!(
        repr.repr_type,
        ReprType::new("text/plain").with_param("charset", "utf-8")
    );
    Ok(())
}
