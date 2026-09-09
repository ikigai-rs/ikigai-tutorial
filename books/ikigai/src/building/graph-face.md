# The graph face

[What resolution buys you](../getting-started/payoff.md) ended with the catalog as one
Turtle graph, and [Why an endpoint describes itself](../getting-started/self-description.md)
claimed that "what can I do with these things?" is a query over it. This chapter runs the
query. It is the shortest chapter in the part, because the RDF audience already knows
what happens next and everybody else only needs to see that it does.

## Two endpoints, no new code

Nothing is written here. `urn:kernel:catalog` already answers as Turtle. The published
`ikigai-sparql` crate binds `urn:sparql:select` (and `ask`, `construct`, `describe`), and
this part's host mounts its space beside Part I's — the `Fallback` in
`building_endpoints::space()`. The SPARQL endpoint holds no store of its own: its `graph`
argument names a resource, and it **resolves that resource through the kernel** and
queries whatever came back.

So the graph you query is the catalog, fetched the ordinary way, by name.

## One SELECT over your own endpoints

"Which endpoints take a string?" — asked of the graph:

```rust,ignore
{{#include ../../../../crates/building-endpoints/src/graph.rs:query}}
```

And the test that asks it of this host:

```rust,ignore
{{#include ../../../../crates/building-endpoints/src/graph.rs:select}}
```

```bash
cargo run -p building-endpoints -- --strings
```

```text
camel-case
title
```

Three things that test pins down are the chapter.

**Declaring `class` is what made the answer possible.** `camel-case` and `title` declared
`.class(xsd:string)` on an input, so they are in the result. `counter` declared
`xsd:nonNegativeInteger`, so it is correctly absent. `markdown` declared no class at all —
and it is invisible to the question, not because it takes something else but because it
never said what it takes. That is [Why an endpoint describes
itself](../getting-started/self-description.md), as a query result.

**The property path is not decoration.** `(ik:input | ik:action/ik:input)` walks two
shapes. A single-verb endpoint's inputs hang off the endpoint node; an endpoint with
per-verb `ActionSpec`s — `title`, `counter` — declares each verb's inputs on that verb's
*action* node. Ask only `ik:input` and every multi-verb endpoint drops out of the answer
silently. (The first draft of this chapter did exactly that, and the test caught it.)

**The answer is a representation.** `as=text/csv` chose it; `application/sparql-results+json`
is the default, and CONSTRUCT would have come back as Turtle. A query result is a resolution
like any other, with a type you asked for.

## From a shell

The CLI links the same crate, so the same question works against its much larger catalog:

```bash
ikigai --plain -c 'source urn:sparql:select graph=urn:kernel:catalog query="PREFIX ik: <https://ikigai-rs.dev/ns#> SELECT DISTINCT ?id WHERE { ?e a ik:Endpoint ; ik:id ?id ; (ik:input | ik:action/ik:input) ?i . ?i ik:class <http://www.w3.org/2001/XMLSchema#string> } ORDER BY ?id" as=text/csv'
```

> ⚠ The ikigai vocabulary is always loaded beside whatever `graph` names — the query
> above could have joined `ik:Endpoint` to its `rdfs:subClassOf` relatives — and a
> transreptor shows up as both `ik:Endpoint` and `ik:Transreptor`, typed explicitly so a
> consumer that does not reason over the subclass axiom still finds it. If a query over
> the catalog returns nothing where you expected something, the first things to check are
> the property path above and whether the endpoint declared a `class` at all.

## Try it

```rust
# extern crate building_endpoints;
# extern crate ikigai_core;
# extern crate futures;
use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Iri, Request, Verb};

let kernel = building_endpoints::kernel_in(None);

// ASK: does anything in this host convert to Markdown?
let request = Request::new(Verb::Source, Iri::parse("urn:sparql:ask").unwrap())
    .with_arg("graph", ArgRef::Inline(b"urn:kernel:catalog".to_vec()))
    .with_arg("query", ArgRef::Inline(
        b"PREFIX ik: <https://ikigai-rs.dev/ns#> ASK { ?t a ik:Transreptor ; ik:transreptsTo \"text/markdown\" }".to_vec(),
    ))
    .with_arg("as", ArgRef::Inline(b"text/csv".to_vec()));

let repr = block_on(kernel.issue(request, &Capability::root())).unwrap();
assert_eq!(String::from_utf8_lossy(&repr.bytes).trim(), "true");
```
