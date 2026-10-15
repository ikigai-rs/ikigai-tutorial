//! The code taught by "A query's authority is its dataset".
//!
//! Two tenants keep their data in one store, each in a named graph of its own: [`NORTH`], a
//! workshop's orders, and [`SOUTH`], a supplier's catalog. The callers are capabilities a host's
//! door would mint: [`north`] and [`south`] each hold their own graph, and [`auditor`] holds both.
//! The same SPARQL text, [`COSTED_ORDERS`], gets three different answers from the three of them,
//! because what a query may read is not written in the query. It is the DATASET the caller names,
//! and the store opens a dataset only to a caller holding a grant for every graph in it.
//!
//! The kernel is "Scripts are resources"' host (the script space in front of the store), seeded
//! with this crate's [`SEED`] instead of that chapter's items.

use std::sync::Arc;

use ikigai_core::{Capability, Fallback, Kernel};
use ikigai_script::authority::{same_for_all, Ceiling};
use ikigai_store::{DurableStore, SharerWrites, Store};
use oxigraph::io::RdfFormat;

/// The workshop's graph: its orders.
pub const NORTH: &str = "urn:iki:tutorial:graph:north";
/// The supplier's graph: its catalog, with prices.
pub const SOUTH: &str = "urn:iki:tutorial:graph:south";

/// The grant to read [`NORTH`] through the store's graph-scoped doors.
pub const READ_NORTH: &str = "urn:cap:store:read:graph:urn:iki:tutorial:graph:north";
/// The grant to write [`NORTH`] through `urn:iki:store:graph-update`.
pub const WRITE_NORTH: &str = "urn:cap:store:write:graph:urn:iki:tutorial:graph:north";
/// The grant to read [`SOUTH`].
pub const READ_SOUTH: &str = "urn:cap:store:read:graph:urn:iki:tutorial:graph:south";
/// The grant to write [`SOUTH`].
pub const WRITE_SOUTH: &str = "urn:cap:store:write:graph:urn:iki:tutorial:graph:south";

// ANCHOR: seed
/// Three orders in the workshop's graph, three parts in the supplier's catalog. An order names a
/// part by the supplier's IRI for it, so the price of an order is a join across the two graphs.
/// The washer is not in the catalog yet: its order has no price in ANY dataset.
pub const SEED: &str = r#"PREFIX schema: <https://schema.org/>
INSERT DATA {
  GRAPH <urn:iki:tutorial:graph:north> {
    <https://north.example/order/1> a schema:Order ;
        schema:orderedItem <https://south.example/part/bolt> ; schema:orderQuantity 40 .
    <https://north.example/order/2> a schema:Order ;
        schema:orderedItem <https://south.example/part/gasket> ; schema:orderQuantity 12 .
    <https://north.example/order/3> a schema:Order ;
        schema:orderedItem <https://south.example/part/washer> ; schema:orderQuantity 500 .
  }
  GRAPH <urn:iki:tutorial:graph:south> {
    <https://south.example/part/bolt> a schema:Product ; schema:price 0.25 .
    <https://south.example/part/gasket> a schema:Product ; schema:price 3.15 .
    <https://south.example/part/hinge> a schema:Product ; schema:price 4.50 .
  }
}"#;
// ANCHOR_END: seed

// ANCHOR: costed_orders
/// Every order, with its part's price where the dataset has one. No graph is named: a bare
/// pattern reads the dataset's default graph, which under `graph=` is the merge of the graphs
/// the caller named.
pub const COSTED_ORDERS: &str = r#"PREFIX schema: <https://schema.org/>
SELECT ?order ?quantity ?price WHERE {
  ?order schema:orderedItem ?part ; schema:orderQuantity ?quantity .
  OPTIONAL { ?part schema:price ?price }
} ORDER BY ?order"#;
// ANCHOR_END: costed_orders

/// How many triples each graph in the dataset holds.
pub const COUNTS: &str = "SELECT ?g (COUNT(*) AS ?triples) \
WHERE { GRAPH ?g { ?s ?p ?o } } GROUP BY ?g ORDER BY ?g";

// ANCHOR: escapes
/// Six ways a query run over the workshop's graph alone might try to read the supplier's, each
/// with the label the program prints for it.
pub const ESCAPES: &[(&str, &str)] = &[
    (
        "GRAPH <south>",
        "PREFIX schema: <https://schema.org/>
SELECT ?part ?price WHERE {
  GRAPH <urn:iki:tutorial:graph:south> { ?part schema:price ?price }
}",
    ),
    (
        "GRAPH ?g",
        "SELECT DISTINCT ?g WHERE { GRAPH ?g { ?s ?p ?o } }",
    ),
    (
        "a sub-select naming south",
        "PREFIX schema: <https://schema.org/>
SELECT ?order ?price WHERE {
  ?order schema:orderedItem ?part .
  { SELECT ?part ?price WHERE { GRAPH <urn:iki:tutorial:graph:south> { ?part schema:price ?price } } }
}",
    ),
    (
        "a property path from an order to a price",
        "PREFIX schema: <https://schema.org/>
SELECT ?order ?price WHERE { ?order schema:orderedItem/schema:price ?price }",
    ),
    (
        "FROM <south>",
        "PREFIX schema: <https://schema.org/>
SELECT ?part ?price FROM <urn:iki:tutorial:graph:south> WHERE { ?part schema:price ?price }",
    ),
    (
        "SERVICE",
        "SELECT * WHERE { SERVICE <https://south.example/sparql> { ?s ?p ?o } }",
    ),
];
// ANCHOR_END: escapes

// ANCHOR: writes
/// What the workshop sends to `urn:iki:store:graph-update graph=<north>`, each with the label
/// the program prints for it.
pub const WRITES: &[(&str, &str)] = &[
    (
        "an order of its own",
        "PREFIX schema: <https://schema.org/>
INSERT DATA { GRAPH <urn:iki:tutorial:graph:north> {
  <https://north.example/order/4> a schema:Order ;
      schema:orderedItem <https://south.example/part/hinge> ; schema:orderQuantity 2 } }",
    ),
    (
        "a better price, in the supplier's graph",
        "PREFIX schema: <https://schema.org/>
INSERT DATA { GRAPH <urn:iki:tutorial:graph:south> {
  <https://south.example/part/bolt> schema:price 0.01 } }",
    ),
    (
        "the same price, in the default graph",
        "PREFIX schema: <https://schema.org/>
INSERT DATA { <https://south.example/part/bolt> schema:price 0.01 }",
    ),
    (
        "WITH <south>, priced from its own orders",
        "PREFIX schema: <https://schema.org/>
WITH <urn:iki:tutorial:graph:south>
INSERT { ?part schema:price 0.01 }
WHERE { GRAPH <urn:iki:tutorial:graph:north> { ?order schema:orderedItem ?part } }",
    ),
    (
        "WITH <south>, reading south",
        "PREFIX schema: <https://schema.org/>
WITH <urn:iki:tutorial:graph:south>
DELETE { ?part schema:price ?old } INSERT { ?part schema:price 0.01 }
WHERE { ?part schema:price ?old }",
    ),
    (
        "CREATE GRAPH",
        "CREATE GRAPH <urn:iki:tutorial:graph:elsewhere>",
    ),
    ("DROP ALL", "DROP ALL"),
];
// ANCHOR_END: writes

// ANCHOR: costed_script
/// [`COSTED_ORDERS`] as a published script. A script's dataset is not chosen by whoever runs it,
/// so this text names its graphs, and its authority is derived from the names.
pub const COSTED_SCRIPT: &str = r#"# Every order, with its part's price.
PREFIX schema: <https://schema.org/>
SELECT ?order ?quantity ?price WHERE {
  GRAPH <urn:iki:tutorial:graph:north> {
    ?order schema:orderedItem ?part ; schema:orderQuantity ?quantity
  }
  OPTIONAL { GRAPH <urn:iki:tutorial:graph:south> { ?part schema:price ?price } }
} ORDER BY ?order"#;
// ANCHOR_END: costed_script

/// The grant to run the published [`COSTED_SCRIPT`], which every caller here holds, so that the
/// graphs are the only difference between them.
pub const RUN_COSTED: &str = "urn:cap:script:run:costed-orders";

// ANCHOR: callers
/// The workshop: reads and writes its own graph.
pub fn north() -> Capability {
    caller(
        "urn:iki:tutorial:tenant:north",
        &[READ_NORTH, WRITE_NORTH, RUN_COSTED],
    )
}

/// The supplier: reads and writes its own graph.
pub fn south() -> Capability {
    caller(
        "urn:iki:tutorial:tenant:south",
        &[READ_SOUTH, WRITE_SOUTH, RUN_COSTED],
    )
}

/// An auditor: reads both graphs and writes neither.
pub fn auditor() -> Capability {
    caller(
        "urn:iki:tutorial:auditor",
        &[READ_NORTH, READ_SOUTH, RUN_COSTED],
    )
}

/// The supplier's price feed: writes the supplier's graph and reads nothing.
pub fn feed() -> Capability {
    caller("urn:iki:tutorial:feed", &[WRITE_SOUTH])
}

/// An old job that holds the store's BROAD write key, the one that means `DROP ALL` of
/// everything.
pub fn job() -> Capability {
    caller("urn:iki:tutorial:job", &["urn:cap:store:write"])
}
// ANCHOR_END: callers

fn caller(principal: &str, scopes: &[&str]) -> Capability {
    Capability::scoped(scopes.iter().copied())
        .with_principal(principal)
        .expect("a principal IRI")
}

// ANCHOR: kernel
/// "Scripts are resources"' kernel (the script space in front of an in-memory store), with the
/// store seeded by [`SEED`]. The host sets no ceiling on scripts: a run may touch what its
/// publisher held and its runner holds.
pub fn kernel() -> Arc<Kernel> {
    script_host::kernel_over(same_for_all(Ceiling::unbounded()), Vec::new(), Some(SEED))
}
// ANCHOR_END: kernel

// ANCHOR: shared
/// A store with a second writer the kernel cannot see: the supplier's price feed, which loads
/// straight into the store's handle and writes only the supplier's graph. The host SAYS so,
/// which is what lets a read of any other graph be cached. The handle comes back for the
/// program to play the feed with.
pub fn shared_kernel() -> (Arc<Kernel>, Arc<Store>) {
    let writes = SharerWrites::only_the_default_graph().and_named_graph(SOUTH);
    let (store, handle) = DurableStore::in_memory_shared_declaring(writes).expect("a store");
    let kernel = Kernel::new(Arc::new(Fallback::new(vec![Arc::new(
        ikigai_store::space(store),
    )])));
    script_host::issue(
        &kernel,
        &Capability::root(),
        ikigai_core::Verb::Sink,
        "urn:iki:store:update",
        &[("content", SEED)],
    )
    .expect("seeding the store");
    (Arc::new(kernel), handle)
}

/// The feed's write, through the handle: no request, so no thread is cut.
pub fn feed_writes(handle: &Store, nquads: &str) {
    handle
        .load_from_slice(RdfFormat::NQuads, nquads.as_bytes())
        .expect("the feed's write");
}
// ANCHOR_END: shared
