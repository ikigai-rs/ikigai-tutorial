# A query's authority is its dataset

[Who is asking](identity.md) mentioned a store's per-graph check in passing, as one example of an
endpoint adding a finer ceiling of its own. This chapter is that example, in full, because it is where the
capability model meets the question people actually ask a shared store: **may this caller run
this query?**

The usual answer looks at the query. Which graphs does the text mention, and may the caller read
each of them? That answer is a parser's opinion of a language with sub-selects, property paths,
`GRAPH ?g`, `FROM`, and `SERVICE`, and every one of those is a way to reach data the text does not
spell out. `ikigai-store` answers a different question. The caller names a **dataset**, a set of
graphs, and the store opens exactly that dataset, through the SPARQL engine's own API, to a caller
holding a grant for every graph in it. The text then runs inside the dataset and cannot get out,
because nothing outside it is there.

So the same SPARQL text is three different queries in the hands of three different callers. This
chapter runs that, and then tries to break it:

<!-- transcript: manual — every section at once; each section's block below is checked. -->
```bash
cargo run -p graph-authority
```

The program runs all eight sections every time; each block below prints one of them.

## Two tenants, one store

A workshop keeps its orders in one named graph, and its supplier keeps a price list in another.
An order names a part by the supplier's own IRI for it, so pricing an order is a join across the
two graphs:

<!-- urn-gate: illustration urn:iki:tutorial:graph:north — the name of a named GRAPH in the
     store, taken by the store's doors as an argument; no space binds it. -->
<!-- urn-gate: illustration urn:iki:tutorial:graph:south — likewise, the supplier's graph. -->
<!-- urn-gate: illustration urn:iki:tutorial:graph:elsewhere — a graph an update tries to
     create, and is refused. -->

```rust,ignore
{{#include ../../../../crates/graph-authority/src/lib.rs:seed}}
```

The callers are capabilities, the kind a host's door mints for whoever authenticated:

```rust,ignore
{{#include ../../../../crates/graph-authority/src/lib.rs:callers}}
```

A grant names exactly one graph: `urn:cap:store:read:graph:<iri>` to read it through the store's
graph-scoped doors, `urn:cap:store:write:graph:<iri>` to write it. There is no prefix form, and
reading and writing one graph are two grants. The broad keys, `urn:cap:store:read` and
`urn:cap:store:write`, are the authority for the whole-dataset doors (`urn:iki:store:select`,
`urn:iki:store:update`), and a graph-scoped door does not take one in place of the graph grant it
declares.

The kernel is [Scripts are resources](../code/scripts.md)' host, the script space in front of an
in-memory store, seeded with the two graphs instead of that chapter's work items. Scripts are a
later part of this book; the one fact about them this chapter needs comes in its own section.

```rust,ignore
{{#include ../../../../crates/graph-authority/src/lib.rs:kernel}}
```

## Which graphs may I read?

Before a caller can name a dataset it needs to know what it may name. `urn:iki:store:graphs`
answers that, and only that:

```bash
cargo run -p graph-authority -- 1
```

```text
north> source urn:iki:store:graphs
urn:iki:tutorial:graph:north

south> source urn:iki:store:graphs
urn:iki:tutorial:graph:south

auditor> source urn:iki:store:graphs
urn:iki:tutorial:graph:north
urn:iki:tutorial:graph:south
```

The answer is **exists and may read**, never one without the other. A tenant learns nothing about
a graph it holds no grant for: not that it exists, not that it does not, not how many there are.
Its lines are always a subset of the graphs its own grants name, so the listing is not an oracle.
A broad reader (`urn:cap:store:read`, or root) gets every named graph in the store, and both kinds
of caller get the same bytes for the part of the store they can both see. The store's default
graph is never listed: it has no IRI, so no grant can name it, and no scoped read can reach it.

## One text, three answers

Here is the query. It names no graph at all, so its bare patterns read the dataset's default
graph, which under `graph=` is the merge of the graphs the caller named:

```rust,ignore
{{#include ../../../../crates/graph-authority/src/lib.rs:costed_orders}}
```

Each caller runs it over the dataset its listing gave back:

```bash
cargo run -p graph-authority -- 2
```

```text
north> source urn:iki:store:graph-select graph=urn:iki:tutorial:graph:north as=text/tab-separated-values   (the query above)
?order	?quantity	?price
<https://north.example/order/1>	40
<https://north.example/order/2>	12
<https://north.example/order/3>	500

south> source urn:iki:store:graph-select graph=urn:iki:tutorial:graph:south as=text/tab-separated-values   (the query above)
?order	?quantity	?price

auditor> source urn:iki:store:graph-select graph="urn:iki:tutorial:graph:north urn:iki:tutorial:graph:south" as=text/tab-separated-values   (the query above)
?order	?quantity	?price
<https://north.example/order/1>	40	0.25
<https://north.example/order/2>	12	3.15
<https://north.example/order/3>	500
```

Three answers, all correct. The workshop sees its orders and no prices, because no price is in
its graph. The supplier sees no orders, because no order is in its graph. The auditor, holding
both grants, names both graphs (whitespace between them, quoted as one argument) and gets the
join: two orders priced, and the washer unpriced because the catalog does not list it yet.

The text did not change and nobody checked it against a list of permitted graphs. What changed
was `graph=`, and the store set that dataset through the engine's API before evaluation, exactly
as if the query had said `FROM <north> FROM NAMED <north>`, without the query saying it.

## Refuse, never narrow

Now the workshop asks for the join it cannot have:

```bash
cargo run -p graph-authority -- 3
```

```text
north> source urn:iki:store:graph-select graph="urn:iki:tutorial:graph:north urn:iki:tutorial:graph:south" as=text/tab-separated-values   (the query above)
error: denied: reading <urn:iki:tutorial:graph:south> through `urn:iki:store:graph-select` needs the grant `urn:cap:store:read:graph:urn:iki:tutorial:graph:south`, which this capability does not hold. Every graph a scoped read names must be granted: the read is refused rather than answered over the graphs that are, because a narrower dataset than the one asked for returns rows that look right and are not. A grant names exactly one graph; holding `urn:cap:store:read` does not imply it, and is instead the authority for `urn:iki:store:select` over the whole dataset.

south> source urn:iki:store:graph-select graph="urn:iki:tutorial:graph:north urn:iki:tutorial:graph:south" as=text/tab-separated-values   (the query above)
error: denied: reading <urn:iki:tutorial:graph:north> through `urn:iki:store:graph-select` needs the grant `urn:cap:store:read:graph:urn:iki:tutorial:graph:north`, which this capability does not hold. Every graph a scoped read names must be granted: the read is refused rather than answered over the graphs that are, because a narrower dataset than the one asked for returns rows that look right and are not. A grant names exactly one graph; holding `urn:cap:store:read` does not imply it, and is instead the authority for `urn:iki:store:select` over the whole dataset.
```

This is the point of the chapter. A dataset with one grant missing is a typed `Denied` that
**names the grant**, and it is never answered over the graphs the caller does hold.

Look at what the narrowed answer would have been. Drop the supplier's graph from the workshop's
join and the query runs over the workshop's graph alone: three orders, no prices. That is
exactly the first answer in the previous section, which was correct there. Here it would be a
lie, and a plausible one. "Our supplier has no prices for any of these" is a sentence somebody
could act on, and nothing in the rows says that it is a statement about the caller's grants and
not about the supplier.

> **Why narrowing is the usual failure.** Graph-level security is most often built as a filter:
> take the graphs the query would read, drop the ones the caller may not, and run what is left.
> It is easy to build, it never errs on the side of disclosure, and it feels safe because the
> failure it risks (reading too much) is the one everybody tests for. Its own failure is the
> quiet one: an `OPTIONAL` that stops matching, a `FILTER NOT EXISTS` that starts passing, a
> `COUNT` that comes back smaller, each of them a well-formed answer to a question the caller
> did not ask. An absent row and a forbidden row look the same in a result set, so the caller
> cannot tell from the answer that anything was taken out. A refusal names the missing grant,
> and the caller can see what happened and what would fix it.

Before `ikigai-store` 0.2.5 a scoped read took one graph, and the join a caller wrote came back
empty rather than refused. The several-graph dataset and the refusal arrived together, because
one without the other would have been the filter described above.

## Confinement by construction

The workshop now tries to reach the supplier's graph from a read whose dataset is its own. Six
ways in:

```rust,ignore
{{#include ../../../../crates/graph-authority/src/lib.rs:escapes}}
```

```bash
cargo run -p graph-authority -- 4
```

```text
north> source urn:iki:store:graph-select graph=urn:iki:tutorial:graph:north as=text/tab-separated-values   (GRAPH <south>)
error: invalid argument `query`: this query names <urn:iki:tutorial:graph:south> in a `GRAPH` block, and it is not in this read's dataset, `graph=urn:iki:tutorial:graph:north`. `urn:iki:store:graph-select` reads exactly the graphs `graph=` names, so that block would match nothing; the query is refused rather than answered empty, because an empty answer to a mistyped or forgotten graph looks exactly like an empty graph. Nothing was evaluated. Correct the IRI, or add the graph to `graph=` (separated by whitespace) under a grant for it

north> source urn:iki:store:graph-select graph=urn:iki:tutorial:graph:north as=text/tab-separated-values   (GRAPH ?g)
?g
<urn:iki:tutorial:graph:north>

north> source urn:iki:store:graph-select graph=urn:iki:tutorial:graph:north as=text/tab-separated-values   (a sub-select naming south)
error: invalid argument `query`: this query names <urn:iki:tutorial:graph:south> in a `GRAPH` block, and it is not in this read's dataset, `graph=urn:iki:tutorial:graph:north`. `urn:iki:store:graph-select` reads exactly the graphs `graph=` names, so that block would match nothing; the query is refused rather than answered empty, because an empty answer to a mistyped or forgotten graph looks exactly like an empty graph. Nothing was evaluated. Correct the IRI, or add the graph to `graph=` (separated by whitespace) under a grant for it

north> source urn:iki:store:graph-select graph=urn:iki:tutorial:graph:north as=text/tab-separated-values   (a property path from an order to a price)
?order	?price

north> source urn:iki:store:graph-select graph=urn:iki:tutorial:graph:north as=text/tab-separated-values   (FROM <south>)
error: invalid argument `query`: this query carries its own `FROM` / `FROM NAMED` clauses, and `urn:iki:store:graph-select` already fixes the dataset: `graph=` IS the dataset, exactly `FROM <urn:iki:tutorial:graph:north> FROM NAMED <urn:iki:tutorial:graph:north>`. The clauses are refused rather than overridden, because answering a `FROM` naming another graph with this graph's rows would be a wrong answer that looked right. Drop them, and name every graph you mean in `graph=`

north> source urn:iki:store:graph-select graph=urn:iki:tutorial:graph:north as=text/tab-separated-values   (SERVICE)
error: invalid argument `query`: `SERVICE` is not available through this store: a federated call would be an outbound request from inside a SPARQL string, gated by no network capability, and no grant opens it here. Nothing was evaluated. Fetch remote data through the kernel, where the net capability applies, and sink it into `urn:iki:store:load`
```

| the attempt | what the workshop gets | why |
|---|---|---|
| `GRAPH <south> { … }` | refused, naming the graph | a constant graph outside the dataset can only match nothing, and an empty answer would look like an empty graph |
| `GRAPH ?g { … }` | its own graph, and nothing else | `?g` ranges over the dataset's named graphs, which is `north` alone |
| a sub-select naming `south` | refused, naming the graph | the check walks the whole query: sub-selects, `OPTIONAL`, `MINUS`, `UNION` and every `EXISTS` |
| a property path, order to price | no rows | the path names no graph; it walks the dataset, and the price is not in it |
| `FROM <south>` | refused | `graph=` is the dataset; a second way to name one is refused rather than silently overridden |
| `SERVICE <…>` | refused, in every build | a federated call would be a network request from inside a string, under no network grant |

Two kinds of answer are in that table, and the difference matters. The refusals are for things
the text **says**: a graph it names, a dataset it declares, a service it calls. The rest are for
things the text **does**, and those are not checked at all, because they do not need to be. `?g`
and the property path ran, and ran correctly, inside a dataset that holds one graph. The refusals
make mistakes legible (a misspelled graph IRI is the commonest of them), and the dataset is what
makes the boundary hold. If the store's walk of the query missed a shape tomorrow, that shape
would still match nothing outside the dataset, because the engine was never given anything
outside it.

That is "by construction". Nothing in the boundary depends on understanding SPARQL, which is why
it can be exact.

## The write side

Writing is the other half, through `urn:iki:store:graph-update graph=<iri>`, under the write grant
for that one graph. A query's dataset can be set through the engine's API; an update's effects
cannot, so the store confines updates differently. It runs the update against a private copy
holding only the part of the granted graph the update can observe, compares the result, and
applies the difference to the real graph in one transaction. **If anything landed anywhere
else, the whole update is refused** and nothing is applied:

```rust,ignore
{{#include ../../../../crates/graph-authority/src/lib.rs:writes}}
```

```bash
cargo run -p graph-authority -- 5
```

```text
north> sink urn:iki:store:graph-update graph=urn:iki:tutorial:graph:north   (an order of its own)
updated <urn:iki:tutorial:graph:north>: +3 -0 quads

north> sink urn:iki:store:graph-update graph=urn:iki:tutorial:graph:north   (a better price, in the supplier's graph)
error: denied: this write is scoped to graph <urn:iki:tutorial:graph:north> and the update would have written graph <urn:iki:tutorial:graph:south>. Nothing was applied. A statement with no `GRAPH` block goes to the default graph, which is not the scoped graph — wrap it in `GRAPH <urn:iki:tutorial:graph:north> { … }`, or hold `urn:cap:store:write` and use `urn:iki:store:update`

north> sink urn:iki:store:graph-update graph=urn:iki:tutorial:graph:north   (the same price, in the default graph)
error: denied: this write is scoped to graph <urn:iki:tutorial:graph:north> and the update would have written the DEFAULT graph. Nothing was applied. A statement with no `GRAPH` block goes to the default graph, which is not the scoped graph — wrap it in `GRAPH <urn:iki:tutorial:graph:north> { … }`, or hold `urn:cap:store:write` and use `urn:iki:store:update`

north> sink urn:iki:store:graph-update graph=urn:iki:tutorial:graph:north   (WITH <south>, priced from its own orders)
error: denied: this write is scoped to graph <urn:iki:tutorial:graph:north> and the update would have written graph <urn:iki:tutorial:graph:south>. Nothing was applied. A statement with no `GRAPH` block goes to the default graph, which is not the scoped graph — wrap it in `GRAPH <urn:iki:tutorial:graph:north> { … }`, or hold `urn:cap:store:write` and use `urn:iki:store:update`

north> sink urn:iki:store:graph-update graph=urn:iki:tutorial:graph:north   (WITH <south>, reading south)
updated <urn:iki:tutorial:graph:north>: +0 -0 quads

north> sink urn:iki:store:graph-update graph=urn:iki:tutorial:graph:north   (CREATE GRAPH)
error: denied: this write is scoped to graph <urn:iki:tutorial:graph:north> and the update created graph <urn:iki:tutorial:graph:elsewhere>. Nothing was applied. Creating or dropping another graph needs `urn:cap:store:write`

north> sink urn:iki:store:graph-update graph=urn:iki:tutorial:graph:north   (DROP ALL)
updated <urn:iki:tutorial:graph:north>: +0 -12 quads

feed> sink urn:iki:store:graph-update graph=urn:iki:tutorial:graph:south   (a new price)
updated <urn:iki:tutorial:graph:south>

feed> sink urn:iki:store:graph-update graph=urn:iki:tutorial:graph:south   (DELETE WHERE: every price)
error: denied: this update has a `WHERE` clause, which reads graph <urn:iki:tutorial:graph:south>, and reading it needs the grant `urn:cap:store:read:graph:urn:iki:tutorial:graph:south` (or `urn:cap:store:read`), which this capability does not hold. Nothing was evaluated. A write grant alone covers updates that read nothing: `INSERT DATA`, `DELETE DATA`, `CLEAR`, `DROP`, `CREATE`

job> sink urn:iki:store:graph-update graph=urn:iki:tutorial:graph:south   (CLEAR GRAPH, under the broad write key)
error: denied: capability does not grant `urn:cap:store:write:graph:*` (declared by `urn:iki:store:graph-update`; any grant under `urn:cap:store:write:graph:` satisfies it)

auditor> source urn:iki:store:graph-select graph="urn:iki:tutorial:graph:north urn:iki:tutorial:graph:south" as=text/tab-separated-values   (triples per graph)
?g	?triples
<urn:iki:tutorial:graph:south>	7
```

Taking them in order:

- An order of its own lands, three quads in the workshop's graph.
- A price in the supplier's graph is refused, and so is the same triple with no `GRAPH` block,
  which would have gone to the store's default graph: that is not the workshop's graph either.
- `WITH <south>` sets the graph an update's templates write to, and it is refused the moment the
  template would have written there. The check is on effects, so it does not care which of
  SPARQL's several spellings put the quad in the wrong place.
- `WITH <south>` whose `WHERE` reads only the supplier's graph is **not** refused. Its `WHERE` ran
  in the private copy, which holds the workshop's graph and nothing else, matched nothing, and so
  wrote nothing: `+0 -0`. Nothing escaped, so there was nothing to refuse. A scoped update cannot
  read outside its graph any more than a scoped query can, but this is the one place in the
  chapter where reaching for another graph gets an empty success instead of a refusal naming it,
  which is the trap the read side closed in 0.2.22. A `+0 -0` from an update that was meant to
  change something is worth a second look.
- `CREATE GRAPH` writes no quad, and it is still refused: it registers a graph, and that is an
  effect outside the grant.
- `DROP ALL` is applied, and empties **the workshop's graph alone**. The private copy held
  nothing else, and emptying a graph is within a grant to write it. Twelve quads: the nine it
  was seeded with and the three it just added.
- The feed holds the write grant for the supplier's graph and nothing to read with. It may add a
  price, and it is not told the `+N -M` counts, because a count is a read (`+0` after an `INSERT
  DATA` would say the quad was already there). It may not run `DELETE WHERE`, which reads the
  graph to decide what to delete: an update with a `WHERE` needs the read grant too, decided on
  the text before anything is evaluated, so the refusal itself says nothing about the graph.
- The old job holds `urn:cap:store:write`, the broad key, which means `DROP ALL` of everything.
  The narrow door refuses it. The broad key opens the broad door, `urn:iki:store:update`, which
  sees the whole dataset; letting it through the narrow door as well would make the narrow
  door's declared grant something other than the one it enforces.

The auditor's count at the end is the evidence: the workshop's graph is gone, and the
supplier's has its six seeded triples and the feed's one new price.

## Authority from the text

A query run through `graph-select` is told its dataset by whoever runs it. A query **published**
is not: it is run later, by other people, and the runner does not get to pick its graphs. So a
published query names them in its text, and [Who a script runs as](../code/authority.md) showed
what follows from that: its authority is derived from the names, `urn:cap:store:read:graph:<iri>`
for each graph the text reads.

```rust,ignore
{{#include ../../../../crates/graph-authority/src/lib.rs:costed_script}}
```

```bash
cargo run -p graph-authority -- 6
```

```text
host> sink urn:script:costed-orders language=sparql   (the text above)
urn:script:costed-orders:version:sha256:…

north> source urn:kernel:actions   (the lines naming urn:script:costed-orders)
(none)

south> source urn:kernel:actions   (the lines naming urn:script:costed-orders)
(none)

auditor> source urn:kernel:actions   (the lines naming urn:script:costed-orders)
urn:script:costed-orders:result
urn:script:costed-orders:runs

north> source urn:script:costed-orders:result as=text/tab-separated-values
error: denied: capability does not grant `urn:cap:store:read:graph:urn:iki:tutorial:graph:south` (declared by `urn:script:costed-orders:result`)

auditor> source urn:script:costed-orders:result as=text/tab-separated-values
?order	?quantity	?price
<https://north.example/order/1>	40	0.25
<https://north.example/order/2>	12	3.15
<https://north.example/order/3>	500
```

All three callers hold the grant to run the script, so the graphs are the only difference between
them. The published query's own entry declares both graph grants, and that declaration is what
the action manifold reads: `urn:kernel:actions` offers the query only to the caller who could
run it. An agent handed the workshop's tool list never sees `costed-orders`, and so never tries
it and fails. If the workshop asks anyway, the kernel's floor refuses it before the script host
runs anything, naming the grant, the same refusal as the join in "Refuse, never narrow", reached
by a different door.

## The cache keeps tenants apart

Every read in this chapter is cacheable, and sharing a cache between tenants is exactly where a
boundary like this would leak if it were going to. Two rules keep it shut.

```bash
cargo run -p graph-authority -- 7
```

```text
north> source urn:iki:store:graph-select graph=urn:iki:tutorial:graph:north as=text/tab-separated-values   (the query above)
3 orders, 0 priced   (computed)

north> source urn:iki:store:graph-select graph=urn:iki:tutorial:graph:north as=text/tab-separated-values   (the query above)
3 orders, 0 priced   (served from the cache)

auditor> source urn:iki:store:graph-select graph=urn:iki:tutorial:graph:north as=text/tab-separated-values   (the query above)
3 orders, 0 priced   (computed)

(a store whose price feed writes the supplier's graph through its own handle)

north> source urn:iki:store:graph-select graph=urn:iki:tutorial:graph:north as=text/tab-separated-values   (the query above)
3 orders, 0 priced   (computed)

north> source urn:iki:store:graph-select graph=urn:iki:tutorial:graph:north as=text/tab-separated-values   (the query above)
3 orders, 0 priced   (served from the cache)

auditor> source urn:iki:store:graph-select graph="urn:iki:tutorial:graph:north urn:iki:tutorial:graph:south" as=text/tab-separated-values   (the query above)
3 orders, 2 priced   (computed)

auditor> source urn:iki:store:graph-select graph="urn:iki:tutorial:graph:north urn:iki:tutorial:graph:south" as=text/tab-separated-values   (the query above)
3 orders, 2 priced   (computed)

feed> (loads the washer's price straight into the store)

auditor> source urn:iki:store:graph-select graph="urn:iki:tutorial:graph:north urn:iki:tutorial:graph:south" as=text/tab-separated-values   (the query above)
3 orders, 3 priced   (computed)
```

**The cache is keyed on the capability.** The workshop's second read is served from the cache;
the auditor's read of the same text over the same graph is computed, because the auditor's
capability is a different key. Two callers never share an entry, so an answer computed under one
tenant's authority is never served under another's, whatever the two answers happen to be. (Two
callers holding the same scopes do share, which is the point: the key is the authority, not the
identity.)

**A read the kernel cannot keep fresh is not cached at all.** The second half runs a store with a
second writer: the supplier's price feed loads straight into the store's handle, which the kernel
cannot see, so no write of the feed's cuts a golden thread.

```rust,ignore
{{#include ../../../../crates/graph-authority/src/lib.rs:shared}}
```

The host says where that writer writes (`SharerWrites`, naming the supplier's graph), and the
store uses the promise per read. A read of the workshop's graph alone cannot be changed by the
feed, so it is still cached. The auditor's join includes the supplier's graph, so **it is never
cached**, the second identical read is computed again, and the feed's price for the washer shows
up on the very next read, with nothing to invalidate. A read over several graphs is cached only
if every graph in it is one no invisible writer touches.

That second rule has a cost that is easy to misread as a bug. A join that includes a shared graph
is as slow on the hundredth read as on the first, while each of its graphs read alone is fast.
That is correct; the alternative is a cached answer that is silently out of date for as long as
the process lives. The promise is also only as good as the host that makes it: a `SharerWrites`
that leaves out a graph the sharer does write makes reads of that graph stale with no error at all,
which is why it names every graph and has no "trust me" form.

## Fair shares of the machine

Authority over a graph is authority to make the store work, and one tenant's expensive query is
every other tenant's latency. So the same doors bound the work as well as the reach:

```bash
cargo run -p graph-authority -- 8
```

```text
north> source urn:iki:store:graph-select graph=urn:iki:tutorial:graph:north max_rows=1 as=text/tab-separated-values   (the query above)
error: invalid argument `query`: the answer exceeds 1 rows; add LIMIT, narrow the query, or ask the host for more. It was refused, not truncated: no part of it was sent (ledger #970). A request's `max_rows=` can only lower this bound

north> source urn:iki:store:graph-select graph=urn:iki:tutorial:graph:north as=text/tab-separated-values   (a join of 33 triple patterns)
error: invalid argument `query`: this SPARQL text joins 33 patterns in one group, and this endpoint refuses more than 32 (MAX_JOIN_OPERANDS) before planning: the planner's join ordering grows about with the cube of that number and cannot be interrupted (ledger #964). A sequence path counts one pattern a step. Split the query, or move a list of values into VALUES
```

The size of an answer is bounded, and past the bound the answer is **refused, not truncated**,
for the same reason a join is refused rather than narrowed: the first thousand rows of a larger
answer look exactly like a complete answer of a thousand rows. A query the planner would choke on
is refused before planning, because planning is the one stage nothing can interrupt.

Inside those bounds, every evaluation has a **time budget**, and it is set the way everything
else in this chapter is: through the capability. A host gives every caller a base (five seconds
by default) and a ceiling (two minutes, which root gets); a door that means some callers to have
longer mints them a `urn:cap:store:budget:<milliseconds>` grant; and a request may carry
`budget=`, which can only lower what its capability gets:

```rust
# extern crate graph_authority;
# extern crate ikigai_core;
# extern crate ikigai_store;
use std::time::Duration;

use graph_authority::{auditor, north, READ_NORTH, READ_SOUTH};
use ikigai_core::Capability;
use ikigai_store::budget::{cap_budget, effective_budget, TimeBudget};

let host = TimeBudget::default();
assert_eq!(host.for_capability(&north()), Duration::from_secs(5));
assert_eq!(host.for_capability(&auditor()), Duration::from_secs(5));
assert_eq!(host.for_capability(&Capability::root()), Duration::from_secs(120));

// A door that mints its batch auditor half a minute.
let batch = Capability::scoped([READ_NORTH.to_string(), READ_SOUTH.to_string(), cap_budget(30_000)]);
assert_eq!(host.for_capability(&batch), Duration::from_secs(30));

// A grant past the ceiling gets the ceiling.
let greedy = Capability::scoped([READ_NORTH.to_string(), cap_budget(3_600_000)]);
assert_eq!(host.for_capability(&greedy), Duration::from_secs(120));

// A request's own `budget=` lowers what its capability gets, and can never raise it.
let five = host.for_capability(&north());
assert_eq!(effective_budget(Some("250"), five).unwrap(), Duration::from_millis(250));
assert_eq!(effective_budget(Some("60000"), five).unwrap(), five);
```

A caller cannot raise its own budget: attenuating a capability only removes grants, and removing
a budget grant only lowers the budget towards the base. At the budget the caller is answered with
a typed `Timeout`, never a partial answer, and an update that runs out of time writes nothing.

What a deadline cannot do is stop every evaluation. Some loops inside the SPARQL engine, an
aggregate over rows already in memory among them, do not check for cancellation, so the caller is
answered on time while a core stays busy until the loop ends. The store therefore caps how many
reads evaluate at once (`TimeBudget::with_max_overdue`; by default a quarter of the machine's
cores, at least one), and a read that finds every slot taken waits within its own budget or is
refused with a transient `Unavailable` that names the cap.

That cap is one pool for everyone in the published releases, so one caller able to start
evaluations the engine cannot stop can fill it, and then every other tenant, the host included,
is refused until those evaluations end. The next release of `ikigai-store` divides it: a reserve
only the host's own work may use, a pool for every other caller together, and a share no single
caller may exceed, where a caller is a principal (every capability that names none counts as one
caller). That change is merged and not yet published, so this book's crate cannot run it; until
it ships, a host that serves callers it does not trust keeps their budgets short.

## The rule, once more

| the question | the answer | what this chapter showed |
|---|---|---|
| what may a query read? | the dataset its caller named, set through the engine's API | one text, three answers |
| what if one graph is not granted? | refused, naming the grant, never answered over the rest | the workshop's join |
| can the text reach past the dataset? | no: what it names is refused, what it does stays inside | the escape table |
| what may an update write? | its one graph, judged on effects | `WITH`, `CREATE GRAPH`, `DROP ALL` |
| which graphs may I name? | `urn:iki:store:graphs`: exists and may read | three listings |
| who is offered a published query? | a caller holding every graph its text names | the manifold |
| can a cached answer cross tenants? | no: keyed on the capability, and never cached over a shared graph | computed, cached, computed |
| can one tenant starve another? | bounded by size, shape and time, through the capability | the bounds, and the budget |

None of this is a second permission system beside the kernel's. Every grant here is an ordinary
capability scope, checked by the kernel's floor where the description declares it and by the
store where the declaration cannot be exact (a family declared, one graph enforced). What the
store adds is the one idea in the title: it does not ask what a query means. It asks which
graphs the caller named, refuses if any one is not granted, and gives the query nothing else to
read.

The gonk Book applies this chapter to a server: [Two teams, one
gonk](../gonk/teams/two-teams.html) gives two teams a ledger each in one gonk, and runs the same
questions through its doors, with an agent per team, a manager over both, and a signed-out
browser.
