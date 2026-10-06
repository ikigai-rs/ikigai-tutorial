# The ledger, the store and browse in your own kernel

Every part of this book has shown a piece twice: as modules in a kernel the reader builds, and as
gonk. This last chapter puts the pieces together the way gonk does, in a kernel of your own, and
leaves out only what makes gonk a server — the doors. What is left is short, and the part of it
worth reading closely is not the composition but the capabilities: with no doors, *who is asking*
is no longer decided by which socket a request came in on, so the host has to say it, per caller.

## One dataset, three families

The ledger and browse both keep state in RDF, and gonk keeps both in **one** dataset, because the
question it exists to answer — *what work is there about this file, and what do we know about the
file* — is a join across them. Here is the whole composition, from
[`crates/embed-host`](https://github.com/ikigai-rs/ikigai-tutorial/tree/main/crates/embed-host):

```rust,ignore
{{#include ../../../../crates/embed-host/src/lib.rs:space}}
```

Three things in it are decisions rather than plumbing.

- **The dataset is handed out once.** `in_memory_shared_declaring` gives the store's space its
  dataset and returns a second handle for browse, which takes an `oxigraph` store directly. A
  durable host does the same with `open_shared_declaring(path, …)`; gonk does exactly this, with
  RocksDB under it.
- **The store is told where the other holder writes.** The store caches a read confined to one
  graph only if nothing it does not see can change that graph. Browse writes behind the store's
  back, so without `SharerWrites` the store would have to treat every read as uncacheable — and
  since a resource is only as cacheable as what it reads, every ledger read with it. With the
  promise, ledger reads stay cached.
- **browse is confined to the graph that was promised.** `Mount::graph` is browse's half of the
  same statement. A promise the sharer does not keep is the worst case — cached answers that never
  expire — so the two are written from one constant.

The model is the stub from [the explain chapter](../browse/explain.md). In a host of your own it
would be a mount of a peer serving `urn:llm:`, as in gonk.

## Who is asking

gonk decides a caller's capability by its door: the socket's owner is root, a loopback browser is
the anonymous grant, a QUIC client is its certificate's grant. A kernel with no doors is asked by
code, and each piece of that code should hold only what its job needs. Three callers:

```rust,ignore
{{#include ../../../../crates/embed-host/src/lib.rs:callers}}
```

The agent is the four-token ledger writer from [the first chapter](../ledger/your-own.md), plus
reading the repository and browse's graph. The reviewer may spend the model and nothing else. The
person may do both and decide. Watch each one stay inside its lines:

```rust
# extern crate embed_host;
# extern crate ikigai_core;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use embed_host::{agent, ask_as, kernel, person, reviewer, Scratch};
use ikigai_core::{Error, Verb};

let repo = Scratch::new();
let host = kernel(repo.path(), Arc::new(AtomicUsize::new(0)));
let (agent, reviewer, person) = (agent(), reviewer(), person());
let file = "urn:repo:demo:file:src/lib.rs";
let denied = |r: Result<String, Error>| matches!(r, Err(Error::Denied(_)));

// The agent files work about a file, and reads the file.
let filed = ask_as(&host, &agent, Verb::Sink, "urn:iki:ledger:append",
    &[("content", "Make initials total"), ("about", file)]).unwrap();
assert!(filed.starts_with("#1 "), "{filed}");
assert!(ask_as(&host, &agent, Verb::Source, file, &[]).unwrap().contains("pub fn initials"));
// It may not spend the model, or write another ledger.
assert!(denied(ask_as(&host, &agent, Verb::Source, "urn:repo:demo:explain:src/lib.rs", &[])));
assert!(denied(ask_as(&host, &agent, Verb::Sink, "urn:iki:ledger:acme:append",
    &[("content", "Not mine")])));

// The reviewer runs a pass; it may not publish what it found, or touch the ledger.
let pass = ask_as(&host, &reviewer, Verb::Source, "urn:repo:demo:review:src/lib.rs", &[]).unwrap();
assert!(pass.contains("2 findings"), "{pass}");
let finding = "urn:iki:finding:bd771489c5a22c9a4d45f4dd";
assert!(denied(ask_as(&host, &reviewer, Verb::Sink, finding, &[("decision", "publish")])));
assert!(denied(ask_as(&host, &reviewer, Verb::Source, "urn:iki:ledger:items", &[])));

// The person decides.
ask_as(&host, &person, Verb::Sink, finding,
    &[("decision", "publish"), ("reproduced", "yes")]).unwrap();

// And the store's promise holds: with browse writing into the same dataset, a ledger read is
// still answered from the cache the second time.
let items = ikigai_core::Request::new(Verb::Source,
    ikigai_core::Iri::parse("urn:iki:ledger:items").unwrap());
ask_as(&host, &agent, Verb::Source, "urn:iki:ledger:items", &[]).unwrap();
assert!(host.is_cached(&items, &agent));
```

The last check is the `SharerWrites` promise at work. Build the same space with
`DurableStore::in_memory_shared()`, which hands out the dataset and promises nothing, and that
assertion fails: every read is computed again, the ledger's included.

None of those refusals is code in this crate. Each is a resource's declared capability, enforced
by the kernel against the caller's scopes, and each refusal names the token it wanted.

## The join

Now the reason for one dataset. Which work items are about a file that has a published
annotation, and what does the annotation say? The ledger's `ledger:about` and browse's
`ik:annotates` both point at the same IRI, `urn:repo:demo:file:src/lib.rs`, so it is one query
over the two graphs — asked by the agent, through the store's narrow door, which checks a read
token for *each* graph it is given:

```rust
# extern crate embed_host;
# extern crate ikigai_core;
# use std::sync::atomic::AtomicUsize;
# use std::sync::Arc;
use embed_host::{agent, ask_as, kernel, person, reviewer, Scratch};
use ikigai_core::{Error, Verb};

# let repo = Scratch::new();
# let host = kernel(repo.path(), Arc::new(AtomicUsize::new(0)));
# let file = "urn:repo:demo:file:src/lib.rs";
# ask_as(&host, &agent(), Verb::Sink, "urn:iki:ledger:append",
#     &[("content", "Make initials total"), ("about", file)]).unwrap();
# ask_as(&host, &reviewer(), Verb::Source, "urn:repo:demo:review:src/lib.rs", &[]).unwrap();
# ask_as(&host, &person(), Verb::Sink, "urn:iki:finding:bd771489c5a22c9a4d45f4dd",
#     &[("decision", "publish"), ("reproduced", "yes")]).unwrap();
let query = r#"
    PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
    PREFIX ik: <https://ikigai-rs.dev/ns#>
    PREFIX oa: <http://www.w3.org/ns/oa#>
    SELECT ?item ?line WHERE {
      ?i ledger:about ?file ; ledger:number ?item .
      ?a a oa:Annotation ; ik:annotates ?file ; oa:hasSelector ?s .
      ?s oa:exact ?line .
    }"#;
let both = "urn:iki:ledger:graph:default urn:iki:browse:graph:default";
let rows = ask_as(&host, &agent(), Verb::Source, "urn:iki:store:graph-select",
    &[("query", query), ("graph", both), ("as", "text/tab-separated-values")]).unwrap();
assert_eq!(rows, "?item\t?line\n\
    1\t\"name.split(' ').map(|word| &word[..1]).collect()\"\n");

// The reviewer holds no read token for the ledger's graph, so it cannot ask.
let refused = ask_as(&host, &reviewer(), Verb::Source, "urn:iki:store:graph-select",
    &[("query", query), ("graph", both)]);
assert!(matches!(refused, Err(Error::Denied(_))));
```

Item 1 is about the file, and the file carries one published annotation, on the line that panics.
The declined finding is not an annotation and does not appear; the pending ones never did.

## What gonk adds

| | your kernel | gonk |
| --- | --- | --- |
| dataset | in memory, `in_memory_shared_declaring` | RocksDB at `~/.ikigai/store`, `open_shared_declaring` |
| model | the stub | a peer mounted at `urn:llm:` (`--mount`) |
| who is asking | a `Capability` the code passes | the door: socket owner, loopback or passkey, a QUIC certificate's grant |
| cache freshness of a file read | browse declares reads live | a watcher on each root cuts the threads, so reads are cached |
| pages | none | the browser face, the review queue |
| backup, provisioning, bridges | none | `backup`, `client add`, `grants`, `checkout`, `kata import`, `roborev file` |

Nothing in the left column is a reduced version of the right. It is the same three modules with the
same resources and the same capabilities; gonk is what you add when other processes, other machines
and people with browsers need to reach them.

## The resource map, as built

- **Sets**: ledgers and their items; roots and their files; findings; annotations.
- **Atoms that hold state**: one dataset (the ledger's graph and browse's graph), the repository's
  files, and the model as an input.
- **Compositions**: every ledger action, explain and review, every finding decision, and the
  store's query doors where the graphs meet.
- **Views**: `next`, listings, `explain-status`, `findings`, `annotations`, and any `SELECT` over
  the union a caller holds tokens for.
- **Code written**: one function that hands one dataset to two families with a promise, and three
  lists of capability tokens. No endpoint.
