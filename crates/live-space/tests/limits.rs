//! The two limits "A space that changes while it runs" states without a listing of its own,
//! held here so the chapter's words stay true: a release that lifts either fails this test,
//! and the chapter's "What it does not do yet" is updated with it.

use std::sync::Arc;

use futures::executor::{block_on, LocalPool};
use futures::task::LocalSpawnExt;
use ikigai_core::{
    ArgRef, Capability, DynamicSpace, EndpointSpace, Exact, FnEndpoint, Invocation, Iri, ReprType,
    Representation, Request, Space, Verb,
};
use live_space::{registry, source, DeclarationState, Host, DECLARATION, FOUR_DOORS};

/// A host whose loop runs under `capability`, over a declaration from `declaration`.
fn host_under(
    declaration: FnEndpoint,
    capability: Capability,
) -> (Arc<ikigai_core::Kernel>, Arc<DynamicSpace>, LocalPool) {
    let space = Arc::new(DynamicSpace::new(
        "workshop",
        Iri::parse(DECLARATION).unwrap(),
        registry(Arc::default()),
    ));
    let fixed = EndpointSpace::new().bind(Exact::new(DECLARATION), declaration);
    let root = Arc::new(ikigai_core::Fallback::new(vec![
        Arc::new(fixed) as Arc<dyn ikigai_core::Space>,
        Arc::clone(&space) as Arc<dyn ikigai_core::Space>,
    ]));
    let kernel = Arc::new(live_space::kernel(root));
    let mut pool = LocalPool::new();
    let (looping, on) = (Arc::clone(&space), Arc::clone(&kernel));
    pool.spawner()
        .spawn_local(async move {
            looping.drive(&on, &capability).await;
        })
        .unwrap();
    pool.run_until_stalled();
    (kernel, space, pool)
}

fn write(kernel: &ikigai_core::Kernel, text: &str) {
    let write = Request::new(Verb::Sink, Iri::parse(DECLARATION).unwrap())
        .with_arg("content", ArgRef::Inline(text.as_bytes().to_vec()));
    block_on(kernel.issue(write, &Capability::root())).unwrap();
}

/// A declaration that is always there, holding `FOUR_DOORS` as Turtle, and answered
/// uncacheable (or cacheable).
fn served(cacheable: bool) -> FnEndpoint {
    let turtle = ikigai_sexpr::arrangement_to_topology(FOUR_DOORS)
        .unwrap()
        .to_turtle();
    FnEndpoint::new(
        "served-declaration",
        move |inv: &Invocation<'_>| match inv.request.verb {
            Verb::Sink => Ok(Representation::new(
                ReprType::new("text/plain"),
                b"ok".to_vec(),
            )),
            _ => {
                let answer =
                    Representation::new(ReprType::new("text/turtle"), turtle.clone().into_bytes());
                Ok(if cacheable {
                    answer.cacheable()
                } else {
                    answer
                })
            }
        },
    )
}

/// Ledger #2048: `drive` under a narrowed capability hears a write to the declaration only
/// when an answer cached under that capability rests on the declaration's thread. An
/// uncacheable declaration never caches one, so the loop binds once, at start, and never
/// hears a write again. Under root it hears every write.
#[test]
fn a_narrowed_loop_misses_an_uncacheable_declaration() {
    let listener = Capability::scoped(["urn:cap:kernel:listen"]);

    let (kernel, space, mut pool) = host_under(served(false), listener.clone());
    assert_eq!(space.status().installs, 1, "the first read binds");
    write(&kernel, "anything");
    pool.run_until_stalled();
    assert_eq!(space.status().installs, 1, "the write was not heard");

    let (kernel, space, mut pool) = host_under(served(true), listener);
    write(&kernel, "anything");
    pool.run_until_stalled();
    assert_eq!(
        space.status().installs,
        2,
        "a cacheable declaration is heard"
    );

    let (kernel, space, mut pool) = host_under(served(false), Capability::root());
    write(&kernel, "anything");
    pool.run_until_stalled();
    assert_eq!(
        space.status().installs,
        2,
        "under root, every write is heard"
    );
}

/// The same rule, met by this crate's own declaration: it is cacheable, but until the first
/// write there is nothing to cache (the read is "not found"), so a loop under a narrowed
/// capability never hears that first write. `Host` runs its loop under root for this reason.
#[test]
fn a_narrowed_loop_misses_the_first_write_to_an_absent_declaration() {
    let listener = Capability::scoped(["urn:cap:kernel:listen"]);
    let declaration = live_space::declaration(Arc::new(DeclarationState::default()));
    let (kernel, space, mut pool) = host_under(declaration, listener);
    write(&kernel, FOUR_DOORS);
    pool.run_until_stalled();
    assert_eq!(space.status().installs, 0);
}

/// Ledger #2049: the status door answers, and the space's entries list it, but the topology
/// (and so the picture) does not show it: `DynamicSpace::topology` reports the inner space
/// alone.
#[test]
fn the_status_door_answers_and_is_not_in_the_topology() {
    let mut host = Host::start("workshop");
    host.write(FOUR_DOORS);
    let status = "urn:iki:space:dynamic:workshop:status";
    assert!(host.read(source(status, &[])).is_ok());
    let turtle = host.kernel.topology().to_turtle();
    assert!(turtle.contains("urn:iki:tutorial:camel-case"), "{turtle}");
    assert!(!turtle.contains(status), "{turtle}");
    let picture = host.read(source("urn:diagram:kernel", &[])).unwrap();
    assert!(!picture.contains(status));
    // The space's entries DO list it, so the kernel's catalog finds it: only the topology,
    // and every picture drawn from it, leaves it out.
    let names = host.root.entries().unwrap_or_default();
    assert!(names.iter().any(|entry| entry.pattern == status));
}
